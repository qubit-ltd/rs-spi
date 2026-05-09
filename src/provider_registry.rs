/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Typed registry for pluggable service providers.

use std::collections::{
    HashMap,
    HashSet,
};
use std::marker::PhantomData;
use std::sync::Arc;

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
/// after registration.
#[derive(Debug)]
pub struct ProviderRegistry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Registered providers in insertion order.
    providers: Vec<ProviderEntry<Spec>>,
    /// Normalized provider id and alias lookup table.
    index: HashMap<ProviderName, usize>,
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the number of registered providers.
    ///
    /// # Returns
    /// Provider count.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Tells whether this registry contains no providers.
    ///
    /// # Returns
    /// `true` when no providers are registered.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Registers a provider owned by the registry.
    ///
    /// # Parameters
    /// - `provider`: Provider to register.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider descriptor is invalid
    /// or when its id or aliases conflict with an existing provider.
    pub fn register<P>(&mut self, provider: P) -> Result<(), ProviderRegistryError>
    where
        P: ServiceProvider<Spec> + 'static,
    {
        self.register_arc(Arc::new(provider))
    }

    /// Registers a shared provider.
    ///
    /// # Parameters
    /// - `provider`: Shared provider to register.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider descriptor is invalid
    /// or when its id or aliases conflict with an existing provider.
    pub fn register_arc(
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
        Ok(())
    }

    /// Gets canonical provider ids in registration order.
    ///
    /// # Returns
    /// Registered provider ids.
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers
            .iter()
            .map(|entry| entry.descriptor.id().as_str())
            .collect()
    }

    /// Gets captured provider descriptors in registration order.
    ///
    /// # Returns
    /// Provider descriptors captured during registration.
    pub fn provider_descriptors(&self) -> Vec<&ProviderDescriptor> {
        self.providers
            .iter()
            .map(|entry| &entry.descriptor)
            .collect()
    }

    /// Finds a provider by id or alias.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias. Names are normalized before lookup.
    ///
    /// # Returns
    /// Matching provider, or `None` when no provider matches or `name` is
    /// invalid.
    pub fn find_provider(&self, name: &str) -> Option<&dyn ServiceProvider<Spec>> {
        let name = ProviderName::new(name).ok()?;
        self.find_entry_by_name(&name)
            .map(|entry| entry.provider.as_ref())
    }

    /// Creates a service from one provider name.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias.
    /// - `config`: Configuration passed to the provider.
    ///
    /// # Returns
    /// Service value created by the selected provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] or
    /// [`ProviderRegistryError::InvalidProviderName`] when `name` is invalid,
    /// [`ProviderRegistryError::UnknownProvider`] when no provider matches,
    /// [`ProviderRegistryError::ProviderUnavailable`] when the provider is not
    /// available, or [`ProviderRegistryError::ProviderCreate`] when the provider
    /// factory fails.
    pub fn create(
        &self,
        name: &str,
        config: &Spec::Config,
    ) -> Result<Spec::Output, ProviderRegistryError> {
        let name = ProviderName::new(name)?;
        let entry = self
            .find_entry_by_name(&name)
            .ok_or_else(|| ProviderRegistryError::UnknownProvider { name: name.clone() })?;
        match entry.provider.availability(config) {
            ProviderAvailability::Available => entry
                .provider
                .create(config)
                .map_err(|error| registry_error_from_create_error(name, error)),
            ProviderAvailability::Unavailable { reason } => {
                Err(ProviderRegistryError::ProviderUnavailable { name, reason })
            }
        }
    }

    /// Creates a service using automatic provider selection.
    ///
    /// # Parameters
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Service created by the highest-priority usable provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when all
    /// automatic candidates fail.
    pub fn create_auto(
        &self,
        config: &Spec::Config,
    ) -> Result<Spec::Output, ProviderRegistryError> {
        self.create_selected(&ProviderSelection::Auto, config)
    }

    /// Creates a service from explicit provider selection.
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
    /// Service created by the first successful provider candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when every
    /// candidate is unknown, unavailable, or fails during creation.
    pub fn create_selected(
        &self,
        selection: &ProviderSelection,
        config: &Spec::Config,
    ) -> Result<Spec::Output, ProviderRegistryError> {
        if self.providers.is_empty() {
            return Err(ProviderRegistryError::EmptyRegistry);
        }
        let candidates = selection.candidates(self.auto_candidates());
        let mut failures = Vec::new();
        for candidate in candidates {
            let Some(entry) = self.find_entry_by_name(&candidate) else {
                failures.push(ProviderFailure::unknown_name(candidate));
                continue;
            };
            match entry.provider.availability(config) {
                ProviderAvailability::Available => match entry.provider.create(config) {
                    Ok(service) => return Ok(service),
                    Err(error) => failures.push(failure_from_create_error(candidate, error)),
                },
                ProviderAvailability::Unavailable { reason } => {
                    failures.push(ProviderFailure::unavailable_name(candidate, &reason));
                }
            }
        }
        Err(ProviderRegistryError::NoAvailableProvider { failures })
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
        let mut local_names = HashSet::with_capacity(descriptor.aliases().len() + 1);
        for name in descriptor.names() {
            if !local_names.insert(name.clone()) || self.index.contains_key(name) {
                return Err(ProviderRegistryError::DuplicateProviderName { name: name.clone() });
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
    fn find_entry_by_name(&self, name: &ProviderName) -> Option<&ProviderEntry<Spec>> {
        self.index
            .get(name)
            .and_then(|provider_index| self.providers.get(*provider_index))
    }

    /// Builds automatic provider candidates.
    ///
    /// # Returns
    /// Provider ids ordered by descending priority and then ascending id.
    fn auto_candidates(&self) -> Vec<ProviderName> {
        let mut providers: Vec<&ProviderEntry<Spec>> = self.providers.iter().collect();
        providers.sort_by(|left, right| {
            right
                .descriptor
                .priority()
                .cmp(&left.descriptor.priority())
                .then_with(|| left.descriptor.id().cmp(right.descriptor.id()))
        });
        providers
            .into_iter()
            .map(|entry| entry.descriptor.id().clone())
            .collect()
    }
}

impl<Spec> Clone for ProviderRegistry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Clones the provider list while sharing provider instances.
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
            index: self.index.clone(),
            marker: PhantomData,
        }
    }
}

impl<Spec> Clone for ProviderEntry<Spec>
where
    Spec: ServiceSpec + 'static,
{
    /// Clones one provider entry while sharing the provider instance.
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
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            index: HashMap::new(),
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
    match error {
        ProviderCreateError::Unavailable { reason } => {
            ProviderRegistryError::ProviderUnavailable { name, reason }
        }
        ProviderCreateError::Failed { reason } => {
            ProviderRegistryError::ProviderCreate { name, reason }
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
fn failure_from_create_error(name: ProviderName, error: ProviderCreateError) -> ProviderFailure {
    match error {
        ProviderCreateError::Unavailable { reason } => {
            ProviderFailure::unavailable_name(name, &reason)
        }
        ProviderCreateError::Failed { reason } => {
            ProviderFailure::create_failed_name(name, &reason)
        }
    }
}
