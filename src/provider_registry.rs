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

use std::collections::HashSet;
use std::sync::Arc;

use crate::{
    ProviderAvailability,
    ProviderFailure,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
};

/// Registry of providers for one service trait and configuration type.
///
/// Provider ids and aliases are matched case-insensitively. Duplicate ids or
/// aliases are rejected during registration, including duplicates within one
/// provider's own id and aliases, so a selector resolves to at most one
/// provider.
#[derive(Debug)]
pub struct ProviderRegistry<S, C, E>
where
    S: ?Sized + 'static,
{
    /// Registered providers in insertion order.
    providers: Vec<Arc<dyn ServiceProvider<Config = C, Error = E, Service = S>>>,
}

impl<S, C, E> ProviderRegistry<S, C, E>
where
    S: ?Sized + 'static,
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
    /// Returns [`ProviderRegistryError::EmptyProviderName`] when the provider id
    /// or an alias is empty after trimming. Returns
    /// [`ProviderRegistryError::DuplicateProviderName`] when the provider id or
    /// an alias conflicts with an existing or same-provider name.
    pub fn register<P>(&mut self, provider: P) -> Result<(), ProviderRegistryError<E>>
    where
        P: ServiceProvider<Config = C, Error = E, Service = S> + 'static,
    {
        self.register_arc(Arc::new(provider))
    }

    /// Registers a shared provider.
    ///
    /// # Parameters
    /// - `provider`: Shared provider to register.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] when the provider id
    /// or an alias is empty after trimming. Returns
    /// [`ProviderRegistryError::DuplicateProviderName`] when the provider id or
    /// an alias conflicts with an existing or same-provider name.
    pub fn register_arc(
        &mut self,
        provider: Arc<dyn ServiceProvider<Config = C, Error = E, Service = S>>,
    ) -> Result<(), ProviderRegistryError<E>> {
        let names = provider_names(provider.as_ref())?;
        let mut local_names = HashSet::with_capacity(names.len());
        for name in names {
            let key = provider_name_key(&name);
            if !local_names.insert(key) || self.find_provider(&name).is_some() {
                return Err(ProviderRegistryError::DuplicateProviderName { name });
            }
        }
        self.providers.push(provider);
        Ok(())
    }

    /// Gets canonical provider ids in registration order.
    ///
    /// # Returns
    /// Registered provider ids.
    pub fn provider_names(&self) -> Vec<&'static str> {
        self.providers
            .iter()
            .map(|provider| provider.id())
            .collect()
    }

    /// Finds a provider by id or alias.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias. Matching is case-insensitive.
    ///
    /// # Returns
    /// Matching provider, or `None` when no provider matches.
    pub fn find_provider(
        &self,
        name: &str,
    ) -> Option<&dyn ServiceProvider<Config = C, Error = E, Service = S>> {
        let selector = name.trim();
        if selector.is_empty() {
            return None;
        }
        self.providers
            .iter()
            .map(Arc::as_ref)
            .find(|provider| provider_matches(*provider, selector))
    }

    /// Creates a service from one provider name.
    ///
    /// # Parameters
    /// - `name`: Provider id or alias.
    /// - `config`: Configuration passed to the provider.
    ///
    /// # Returns
    /// Boxed service created by the selected provider.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] when `name` is empty,
    /// [`ProviderRegistryError::UnknownProvider`] when no provider matches,
    /// [`ProviderRegistryError::ProviderUnavailable`] when the provider is not
    /// available, or [`ProviderRegistryError::ProviderCreate`] when the provider
    /// factory fails.
    pub fn create(&self, name: &str, config: &C) -> Result<Box<S>, ProviderRegistryError<E>> {
        let selector = normalize_provider_name(name)?;
        let provider = self.find_provider(&selector).ok_or_else(|| {
            ProviderRegistryError::UnknownProvider {
                name: selector.clone(),
            }
        })?;
        match provider.availability(config) {
            ProviderAvailability::Available => {
                provider
                    .create(config)
                    .map_err(|error| ProviderRegistryError::ProviderCreate {
                        name: selector,
                        error,
                    })
            }
            ProviderAvailability::Unavailable { reason } => {
                Err(ProviderRegistryError::ProviderUnavailable {
                    name: selector,
                    reason,
                })
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
    pub fn create_auto(&self, config: &C) -> Result<Box<S>, ProviderRegistryError<E>> {
        self.create_default(&ProviderSelection::auto(), config)
    }

    /// Creates a service from default and fallback provider selection.
    ///
    /// Automatic selection tries all registered providers ordered by descending
    /// priority and then by provider id. Explicit selection tries the configured
    /// default provider followed by fallbacks in order. Selection stops at the
    /// first provider that can create a service.
    ///
    /// # Parameters
    /// - `selection`: Default and fallback provider selection.
    /// - `config`: Configuration passed to candidate providers.
    ///
    /// # Returns
    /// Service created by the first successful provider candidate.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyRegistry`] when the registry has no
    /// providers, or [`ProviderRegistryError::NoAvailableProvider`] when every
    /// candidate is unknown, unavailable, or fails during creation.
    pub fn create_default(
        &self,
        selection: &ProviderSelection,
        config: &C,
    ) -> Result<Box<S>, ProviderRegistryError<E>> {
        if self.providers.is_empty() {
            return Err(ProviderRegistryError::EmptyRegistry);
        }
        let candidates = selection.candidates(self.auto_candidates());
        let mut failures = Vec::new();
        for candidate in candidates {
            let Some(provider) = self.find_provider(&candidate) else {
                failures.push(ProviderFailure::unknown(&candidate));
                continue;
            };
            match provider.availability(config) {
                ProviderAvailability::Available => match provider.create(config) {
                    Ok(service) => return Ok(service),
                    Err(error) => failures.push(ProviderFailure::create_failed(&candidate, error)),
                },
                ProviderAvailability::Unavailable { reason } => {
                    failures.push(ProviderFailure::unavailable(&candidate, &reason));
                }
            }
        }
        Err(ProviderRegistryError::NoAvailableProvider { failures })
    }

    /// Builds automatic provider candidates.
    ///
    /// # Returns
    /// Provider ids ordered by descending priority and then ascending id.
    fn auto_candidates(&self) -> Vec<String> {
        let mut providers: Vec<&dyn ServiceProvider<Config = C, Error = E, Service = S>> =
            self.providers.iter().map(Arc::as_ref).collect();
        providers.sort_by(|left, right| {
            right
                .priority()
                .cmp(&left.priority())
                .then_with(|| left.id().cmp(right.id()))
        });
        providers
            .into_iter()
            .map(|provider| provider.id().to_owned())
            .collect()
    }
}

impl<S, C, E> Clone for ProviderRegistry<S, C, E>
where
    S: ?Sized + 'static,
{
    /// Clones the provider list while sharing provider instances.
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
        }
    }
}

impl<S, C, E> Default for ProviderRegistry<S, C, E>
where
    S: ?Sized + 'static,
{
    /// Creates an empty provider registry.
    fn default() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
}

/// Gets all names exposed by a provider.
///
/// # Parameters
/// - `provider`: Provider to inspect.
///
/// # Returns
/// Provider id followed by aliases.
///
/// # Errors
/// Returns [`ProviderRegistryError::EmptyProviderName`] when any exposed name is
/// empty after trimming.
fn provider_names<S, C, E>(
    provider: &dyn ServiceProvider<Config = C, Error = E, Service = S>,
) -> Result<Vec<String>, ProviderRegistryError<E>>
where
    S: ?Sized + 'static,
{
    let mut names = Vec::with_capacity(provider.aliases().len() + 1);
    names.push(normalize_provider_name(provider.id())?);
    for alias in provider.aliases() {
        names.push(normalize_provider_name(alias)?);
    }
    Ok(names)
}

/// Normalizes and validates one provider name.
///
/// # Parameters
/// - `name`: Raw provider name.
///
/// # Returns
/// Trimmed provider name.
///
/// # Errors
/// Returns [`ProviderRegistryError::EmptyProviderName`] when `name` is empty
/// after trimming.
fn normalize_provider_name<E>(name: &str) -> Result<String, ProviderRegistryError<E>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Err(ProviderRegistryError::EmptyProviderName)
    } else {
        Ok(trimmed.to_owned())
    }
}

/// Builds a case-insensitive comparison key for a provider name.
///
/// # Parameters
/// - `name`: Normalized provider name.
///
/// # Returns
/// Lowercase provider-name key.
fn provider_name_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// Tells whether a provider matches a requested selector.
///
/// # Parameters
/// - `provider`: Provider to inspect.
/// - `selector`: Requested provider id or alias.
///
/// # Returns
/// `true` when the selector matches the provider id or any alias.
fn provider_matches<S, C, E>(
    provider: &dyn ServiceProvider<Config = C, Error = E, Service = S>,
    selector: &str,
) -> bool
where
    S: ?Sized + 'static,
{
    provider.id().trim().eq_ignore_ascii_case(selector)
        || provider
            .aliases()
            .iter()
            .any(|alias| alias.trim().eq_ignore_ascii_case(selector))
}
