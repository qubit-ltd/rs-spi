// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Typed registry for pluggable service providers.

use std::collections::{
    HashMap,
    HashSet,
};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use log::{
    debug,
    trace,
};

use crate::{
    ProviderAvailability,
    ProviderCreateError,
    ProviderDescriptor,
    ProviderFailure,
    ProviderName,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};

/// Registry of providers for one service specification.
///
/// Provider descriptors are captured during registration. Provider ids and
/// aliases are normalized into [`ProviderName`] values and indexed
/// case-insensitively, so lookup does not depend on provider metadata changing
/// after registration. Automatic selection order is cached during registration
/// updates. Registries store providers behind shared trait objects, so
/// registered providers and service specifications must be `'static`.
#[derive(Debug)]
pub struct ProviderRegistry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Registered providers in insertion order.
    providers: Vec<ProviderEntry<Spec>>,
    /// Normalized provider id and alias lookup table.
    index: HashMap<ProviderName, usize>,
    /// Cached automatic-selection candidates ordered by priority and id.
    auto_candidates: Vec<ProviderName>,
    /// Keeps the service specification attached to this registry type.
    marker: PhantomData<fn() -> Spec>,
}

/// Registered provider and its captured descriptor.
#[derive(Debug)]
struct ProviderEntry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Captured provider descriptor.
    descriptor: ProviderDescriptor,
    /// Provider factory.
    provider: Arc<dyn ServiceProvider<Spec>>,
}

impl<Spec> ProviderRegistry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Creates an empty provider registry.
    ///
    /// # Returns
    /// Empty provider registry.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the number of registered providers.
    ///
    /// # Returns
    /// Provider count.
    #[inline]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Tells whether this registry contains no providers.
    ///
    /// # Returns
    /// `true` when no providers are registered.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Registers a provider owned by the registry.
    ///
    /// # Parameters
    /// - `provider`: Provider to register.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider descriptor is
    /// invalid or when its id or aliases conflict with an existing
    /// provider.
    #[inline]
    pub fn register<P>(
        &mut self,
        provider: P,
    ) -> Result<(), ProviderRegistryError>
    where
        P: ServiceProvider<Spec> + 'static,
    {
        self.register_provider(Arc::new(provider))
    }

    /// Registers a shared provider.
    ///
    /// # Parameters
    /// - `provider`: Shared provider to register.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider descriptor is
    /// invalid or when its id or aliases conflict with an existing
    /// provider.
    #[inline]
    pub fn register_shared(
        &mut self,
        provider: Arc<dyn ServiceProvider<Spec>>,
    ) -> Result<(), ProviderRegistryError> {
        self.register_provider(provider)
    }

    /// Registers a provider stored behind a trait object.
    ///
    /// # Parameters
    /// - `provider`: Shared provider trait object to register.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider descriptor is
    /// invalid or when its id or aliases conflict with an existing
    /// provider.
    fn register_provider(
        &mut self,
        provider: Arc<dyn ServiceProvider<Spec>>,
    ) -> Result<(), ProviderRegistryError> {
        let descriptor = provider.descriptor()?;
        self.validate_descriptor(&descriptor)?;
        let provider_index = self.providers.len();
        for name in descriptor.names() {
            self.index.insert(name.clone(), provider_index);
        }
        self.providers.push(ProviderEntry {
            descriptor,
            provider,
        });
        self.rebuild_auto_candidates();
        debug!(
            "registered provider '{}' with {} aliases and priority {}",
            self.providers[provider_index].descriptor.id(),
            self.providers[provider_index].descriptor.aliases().len(),
            self.providers[provider_index].descriptor.priority(),
        );
        Ok(())
    }

    /// Gets canonical provider ids in registration order.
    ///
    /// # Returns
    /// Registered provider ids.
    #[inline]
    pub fn provider_names(&self) -> Vec<&str> {
        self.iter_provider_names().collect()
    }

    /// Iterates over canonical provider ids in registration order.
    ///
    /// # Returns
    /// Iterator over registered provider ids.
    #[inline]
    pub fn iter_provider_names(&self) -> impl Iterator<Item = &str> + '_ {
        self.providers
            .iter()
            .map(|entry| entry.descriptor.id().as_str())
    }

    /// Gets captured provider descriptors in registration order.
    ///
    /// # Returns
    /// Provider descriptors captured during registration.
    #[inline]
    pub fn provider_descriptors(&self) -> Vec<&ProviderDescriptor> {
        self.iter_provider_descriptors().collect()
    }

    /// Iterates over captured provider descriptors in registration order.
    ///
    /// # Returns
    /// Iterator over provider descriptors captured during registration.
    #[inline]
    pub fn iter_provider_descriptors(
        &self,
    ) -> impl Iterator<Item = &ProviderDescriptor> + '_ {
        self.providers.iter().map(|entry| &entry.descriptor)
    }

    /// Finds a provider by id or alias.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias. Names are normalized before lookup.
    ///
    /// # Returns
    /// Matching provider, or `None` when no provider matches or `name` is
    /// invalid.
    #[inline]
    pub fn find_provider(
        &self,
        name: &str,
    ) -> Option<&dyn ServiceProvider<Spec>> {
        self.resolve_provider(name).ok()
    }

    /// Resolves a provider by id or alias.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias. Names are normalized before lookup.
    ///
    /// # Returns
    /// Matching provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] or
    /// [`ProviderRegistryError::InvalidProviderName`] when `name` is invalid,
    /// or [`ProviderRegistryError::UnknownProvider`] when no provider matches.
    pub fn resolve_provider(
        &self,
        name: &str,
    ) -> Result<&dyn ServiceProvider<Spec>, ProviderRegistryError> {
        let name = match ProviderName::new(name) {
            Ok(name) => name,
            Err(error) => {
                trace!("provider resolution rejected invalid name: {error}");
                return Err(error);
            }
        };
        let Some(entry) = self.find_entry_by_name(&name) else {
            trace!("provider resolution missed provider '{name}'");
            return Err(ProviderRegistryError::UnknownProvider { name });
        };
        trace!(
            "provider resolution matched '{}' to registered provider '{}'",
            name,
            entry.descriptor.id(),
        );
        Ok(entry.provider.as_ref())
    }

    /// Creates a boxed service from one provider name.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias.
    /// - `config`: Configuration passed to the provider.
    ///
    /// # Returns
    /// Boxed service value created by the selected provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] or
    /// [`ProviderRegistryError::InvalidProviderName`] when `name` is invalid,
    /// [`ProviderRegistryError::UnknownProvider`] when no provider matches,
    /// [`ProviderRegistryError::ProviderUnavailable`] when the provider is not
    /// available, or [`ProviderRegistryError::ProviderCreate`] when the
    /// provider factory fails.
    #[inline]
    pub fn create_box(
        &self,
        name: &str,
        config: &Spec::Config,
    ) -> Result<Box<Spec::Service>, ProviderRegistryError> {
        self.create_with(name, config, |provider, config| {
            provider.create_box(config)
        })
    }

    /// Creates an atomically shared service from one provider name.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias.
    /// - `config`: Configuration passed to the provider.
    ///
    /// # Returns
    /// Atomically shared service value created by the selected provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] or
    /// [`ProviderRegistryError::InvalidProviderName`] when `name` is invalid,
    /// [`ProviderRegistryError::UnknownProvider`] when no provider matches,
    /// [`ProviderRegistryError::ProviderUnavailable`] when the provider is not
    /// available, or [`ProviderRegistryError::ProviderCreate`] when the
    /// provider factory fails.
    #[inline]
    pub fn create_arc(
        &self,
        name: &str,
        config: &Spec::Config,
    ) -> Result<Arc<Spec::Service>, ProviderRegistryError> {
        self.create_with(name, config, |provider, config| {
            provider.create_arc(config)
        })
    }

    /// Creates a locally shared service from one provider name.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias.
    /// - `config`: Configuration passed to the provider.
    ///
    /// # Returns
    /// Locally shared service value created by the selected provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] or
    /// [`ProviderRegistryError::InvalidProviderName`] when `name` is invalid,
    /// [`ProviderRegistryError::UnknownProvider`] when no provider matches,
    /// [`ProviderRegistryError::ProviderUnavailable`] when the provider is not
    /// available, or [`ProviderRegistryError::ProviderCreate`] when the
    /// provider factory fails.
    #[inline]
    pub fn create_rc(
        &self,
        name: &str,
        config: &Spec::Config,
    ) -> Result<Rc<Spec::Service>, ProviderRegistryError> {
        self.create_with(name, config, |provider, config| {
            provider.create_rc(config)
        })
    }

    /// Creates a service handle from one provider name.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias.
    /// - `config`: Configuration passed to the provider.
    /// - `create`: Provider factory method used to create the handle.
    ///
    /// # Returns
    /// Service handle created by the selected provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider name is invalid,
    /// unknown, unavailable, or when the provider factory fails.
    fn create_with<Handle, Create>(
        &self,
        name: &str,
        config: &Spec::Config,
        create: Create,
    ) -> Result<Handle, ProviderRegistryError>
    where
        Create: Fn(
            &dyn ServiceProvider<Spec>,
            &Spec::Config,
        ) -> Result<Handle, ProviderCreateError>,
    {
        let name = ProviderName::new(name)?;
        let entry = self.find_entry_by_name(&name).ok_or_else(|| {
            ProviderRegistryError::UnknownProvider { name: name.clone() }
        })?;
        trace!("creating service from provider '{name}'");
        match entry.provider.availability(config) {
            ProviderAvailability::Available => {
                match create(entry.provider.as_ref(), config) {
                    Ok(service) => {
                        debug!("provider '{name}' created service");
                        Ok(service)
                    }
                    Err(error) => {
                        trace!(
                            "provider '{name}' failed to create service: {}",
                            error.reason(),
                        );
                        Err(registry_error_from_create_error(name, error))
                    }
                }
            }
            ProviderAvailability::Unavailable { reason } => {
                trace!("provider '{name}' is unavailable: {reason}");
                Err(ProviderRegistryError::ProviderUnavailable {
                    name,
                    source: ProviderCreateError::unavailable(&reason),
                })
            }
        }
    }

    /// Creates a boxed service using automatic provider selection.
    ///
    /// # Parameters
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Boxed service created by the highest-priority usable provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyRegistry`] when the registry has
    /// no providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// all automatic candidates fail.
    #[inline]
    pub fn create_auto_box(
        &self,
        config: &Spec::Config,
    ) -> Result<Box<Spec::Service>, ProviderRegistryError> {
        self.create_selected_box(&ProviderSelection::Auto, config)
    }

    /// Creates an atomically shared service using automatic provider selection.
    ///
    /// # Parameters
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Atomically shared service created by the highest-priority usable
    /// provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyRegistry`] when the registry has
    /// no providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// all automatic candidates fail.
    #[inline]
    pub fn create_auto_arc(
        &self,
        config: &Spec::Config,
    ) -> Result<Arc<Spec::Service>, ProviderRegistryError> {
        self.create_selected_arc(&ProviderSelection::Auto, config)
    }

    /// Creates a locally shared service using automatic provider selection.
    ///
    /// # Parameters
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Locally shared service created by the highest-priority usable provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyRegistry`] when the registry has
    /// no providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// all automatic candidates fail.
    #[inline]
    pub fn create_auto_rc(
        &self,
        config: &Spec::Config,
    ) -> Result<Rc<Spec::Service>, ProviderRegistryError> {
        self.create_selected_rc(&ProviderSelection::Auto, config)
    }

    /// Creates a boxed service from explicit provider selection.
    ///
    /// Automatic selection tries all registered providers ordered by descending
    /// priority and then by provider id. Named selection tries the primary
    /// provider followed by fallbacks in order. Selection stops at the first
    /// provider that can create a service.
    ///
    /// # Parameters
    /// - `selection`: Provider selection policy.
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Boxed service created by the first successful provider candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::DuplicateProviderCandidate`] when a
    /// named selection repeats a candidate name,
    /// [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// every candidate is unknown, unavailable, or fails during creation.
    #[inline]
    pub fn create_selected_box(
        &self,
        selection: &ProviderSelection,
        config: &Spec::Config,
    ) -> Result<Box<Spec::Service>, ProviderRegistryError> {
        self.create_selected_with(selection, config, |provider, config| {
            provider.create_box(config)
        })
    }

    /// Creates an atomically shared service from explicit provider selection.
    ///
    /// Automatic selection tries all registered providers ordered by descending
    /// priority and then by provider id. Named selection tries the primary
    /// provider followed by fallbacks in order. Selection stops at the first
    /// provider that can create a service.
    ///
    /// # Parameters
    /// - `selection`: Provider selection policy.
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Atomically shared service created by the first successful provider
    /// candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::DuplicateProviderCandidate`] when a
    /// named selection repeats a candidate name,
    /// [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// every candidate is unknown, unavailable, or fails during creation.
    #[inline]
    pub fn create_selected_arc(
        &self,
        selection: &ProviderSelection,
        config: &Spec::Config,
    ) -> Result<Arc<Spec::Service>, ProviderRegistryError> {
        self.create_selected_with(selection, config, |provider, config| {
            provider.create_arc(config)
        })
    }

    /// Creates a locally shared service from explicit provider selection.
    ///
    /// Automatic selection tries all registered providers ordered by descending
    /// priority and then by provider id. Named selection tries the primary
    /// provider followed by fallbacks in order. Selection stops at the first
    /// provider that can create a service.
    ///
    /// # Parameters
    /// - `selection`: Provider selection policy.
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Locally shared service created by the first successful provider
    /// candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::DuplicateProviderCandidate`] when a
    /// named selection repeats a candidate name,
    /// [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// every candidate is unknown, unavailable, or fails during creation.
    #[inline]
    pub fn create_selected_rc(
        &self,
        selection: &ProviderSelection,
        config: &Spec::Config,
    ) -> Result<Rc<Spec::Service>, ProviderRegistryError> {
        self.create_selected_with(selection, config, |provider, config| {
            provider.create_rc(config)
        })
    }

    /// Creates a service handle from explicit provider selection.
    ///
    /// # Parameters
    /// - `selection`: Provider selection policy.
    /// - `config`: Configuration passed to candidate providers.
    /// - `create`: Provider factory method used to create the handle.
    ///
    /// # Returns
    /// Service handle created by the first successful provider candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::DuplicateProviderCandidate`] when a
    /// named selection repeats a candidate name,
    /// [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when
    /// every candidate is unknown, unavailable, or fails during creation.
    fn create_selected_with<Handle, Create>(
        &self,
        selection: &ProviderSelection,
        config: &Spec::Config,
        create: Create,
    ) -> Result<Handle, ProviderRegistryError>
    where
        Create: Fn(
            &dyn ServiceProvider<Spec>,
            &Spec::Config,
        ) -> Result<Handle, ProviderCreateError>,
    {
        selection.validate_unique_names()?;
        if self.providers.is_empty() {
            trace!("provider selection failed because registry is empty");
            return Err(ProviderRegistryError::EmptyRegistry);
        }
        match selection {
            ProviderSelection::Auto => {
                trace!(
                    "automatic provider selection prepared {} candidate(s)",
                    self.auto_candidates().len(),
                );
                self.create_from_candidates_with(
                    self.auto_candidates().iter(),
                    config,
                    &create,
                )
            }
            ProviderSelection::Named { primary, fallbacks } => {
                trace!(
                    "named provider selection will try primary '{}' with {} fallback(s)",
                    primary,
                    fallbacks.len(),
                );
                self.create_from_candidates_with(
                    std::iter::once(primary).chain(fallbacks.iter()),
                    config,
                    &create,
                )
            }
        }
    }

    /// Creates a service handle by trying the supplied candidates in order.
    ///
    /// # Parameters
    /// - `candidates`: Candidate provider names to try.
    /// - `config`: Configuration passed to candidate providers.
    /// - `create`: Provider factory method used to create the handle.
    ///
    /// # Returns
    /// Service handle created by the first successful candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::NoAvailableProvider`] when every
    /// candidate is unknown, unavailable, or fails during creation. Candidate
    /// names that resolve to a provider already tried earlier in the same
    /// selection are skipped.
    fn create_from_candidates_with<'a, I, Handle, Create>(
        &self,
        candidates: I,
        config: &Spec::Config,
        create: &Create,
    ) -> Result<Handle, ProviderRegistryError>
    where
        I: IntoIterator<Item = &'a ProviderName>,
        Create: Fn(
            &dyn ServiceProvider<Spec>,
            &Spec::Config,
        ) -> Result<Handle, ProviderCreateError>,
    {
        let mut failures = Vec::new();
        let mut tried_provider_indices = HashSet::new();
        for candidate in candidates {
            if let Some(provider_index) = self.index.get(candidate).copied()
                && !tried_provider_indices.insert(provider_index)
            {
                trace!(
                    "provider candidate '{candidate}' resolves to an already tried provider; skipping",
                );
                continue;
            }
            match self.create_from_candidate_with(candidate, config, create) {
                Ok(service) => {
                    debug!("provider candidate '{candidate}' created service");
                    return Ok(service);
                }
                Err(failure) => {
                    trace!("provider candidate failed: {failure}");
                    failures.push(failure);
                }
            }
        }
        trace!(
            "provider selection exhausted all candidates with {} failure(s)",
            failures.len(),
        );
        Err(ProviderRegistryError::NoAvailableProvider { failures })
    }

    /// Creates a service handle from one normalized candidate name.
    ///
    /// # Parameters
    /// - `candidate`: Normalized provider id or alias.
    /// - `config`: Configuration passed to the provider.
    /// - `create`: Provider factory method used to create the handle.
    ///
    /// # Returns
    /// Service handle created by the provider.
    ///
    /// # Errors
    /// Returns [`ProviderFailure`] when the candidate is unknown, unavailable,
    /// or fails during service creation.
    fn create_from_candidate_with<Handle, Create>(
        &self,
        candidate: &ProviderName,
        config: &Spec::Config,
        create: &Create,
    ) -> Result<Handle, ProviderFailure>
    where
        Create: Fn(
            &dyn ServiceProvider<Spec>,
            &Spec::Config,
        ) -> Result<Handle, ProviderCreateError>,
    {
        let Some(entry) = self.find_entry_by_name(candidate) else {
            trace!("provider candidate '{candidate}' is unknown");
            return Err(ProviderFailure::unknown_name(candidate.clone()));
        };
        match entry.provider.availability(config) {
            ProviderAvailability::Available => {
                create(entry.provider.as_ref(), config).map_err(|error| {
                    failure_from_create_error(candidate.clone(), error)
                })
            }
            ProviderAvailability::Unavailable { reason } => {
                trace!(
                    "provider candidate '{candidate}' is unavailable: {reason}"
                );
                Err(ProviderFailure::unavailable_name(
                    candidate.clone(),
                    &reason,
                ))
            }
        }
    }

    /// Validates that a descriptor does not conflict with existing entries.
    ///
    /// # Parameters
    /// - `descriptor`: Descriptor to validate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::DuplicateProviderName`] when the
    /// descriptor contains duplicate names or conflicts with an existing name.
    fn validate_descriptor(
        &self,
        descriptor: &ProviderDescriptor,
    ) -> Result<(), ProviderRegistryError> {
        let mut local_names =
            HashSet::with_capacity(descriptor.aliases().len() + 1);
        for name in descriptor.names() {
            if !local_names.insert(name.clone())
                || self.index.contains_key(name)
            {
                return Err(ProviderRegistryError::DuplicateProviderName {
                    name: name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Finds a provider entry by a normalized provider name.
    ///
    /// # Parameters
    /// - `name`: Normalized provider id or alias.
    ///
    /// # Returns
    /// Matching provider entry, or `None` when no provider matches.
    fn find_entry_by_name(
        &self,
        name: &ProviderName,
    ) -> Option<&ProviderEntry<Spec>> {
        self.index
            .get(name)
            .and_then(|provider_index| self.providers.get(*provider_index))
    }

    /// Gets cached automatic provider candidates.
    ///
    /// # Returns
    /// Provider ids ordered by descending priority and then ascending id.
    fn auto_candidates(&self) -> &[ProviderName] {
        &self.auto_candidates
    }

    /// Rebuilds cached automatic provider candidates.
    fn rebuild_auto_candidates(&mut self) {
        let mut provider_indices: Vec<usize> =
            (0..self.providers.len()).collect();
        provider_indices.sort_by(|left, right| {
            let left_descriptor = &self.providers[*left].descriptor;
            let right_descriptor = &self.providers[*right].descriptor;
            right_descriptor
                .priority()
                .cmp(&left_descriptor.priority())
                .then_with(|| left_descriptor.id().cmp(right_descriptor.id()))
        });
        self.auto_candidates = provider_indices
            .into_iter()
            .map(|provider_index| {
                self.providers[provider_index].descriptor.id().clone()
            })
            .collect();
    }
}

impl<Spec> Clone for ProviderRegistry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Clones the provider list while sharing provider instances.
    #[inline]
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
            index: self.index.clone(),
            auto_candidates: self.auto_candidates.clone(),
            marker: PhantomData,
        }
    }
}

impl<Spec> Clone for ProviderEntry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Clones one provider entry while sharing the provider instance.
    #[inline]
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor.clone(),
            provider: self.provider.clone(),
        }
    }
}

impl<Spec> Default for ProviderRegistry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Creates an empty provider registry.
    #[inline]
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            index: HashMap::new(),
            auto_candidates: Vec::new(),
            marker: PhantomData,
        }
    }
}

/// Converts a provider creation error into a registry error.
///
/// # Parameters
/// - `name`: Provider name being created.
/// - `error`: Provider creation error.
///
/// # Returns
/// Registry error with provider-name context.
fn registry_error_from_create_error(
    name: ProviderName,
    error: ProviderCreateError,
) -> ProviderRegistryError {
    if error.is_unavailable() {
        ProviderRegistryError::ProviderUnavailable {
            name,
            source: error,
        }
    } else {
        ProviderRegistryError::ProviderCreate {
            name,
            source: error,
        }
    }
}

/// Converts a provider creation error into a candidate failure.
///
/// # Parameters
/// - `name`: Candidate provider name.
/// - `error`: Provider creation error.
///
/// # Returns
/// Candidate failure with provider-name context.
fn failure_from_create_error(
    name: ProviderName,
    error: ProviderCreateError,
) -> ProviderFailure {
    if error.is_unavailable() {
        ProviderFailure::unavailable_error(name, error)
    } else {
        ProviderFailure::create_failed_error(name, error)
    }
}
