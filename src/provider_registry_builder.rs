// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Startup-only assembly of immutable provider registries.

use std::{collections::HashMap, sync::Arc};

use crate::{
    ProviderDescriptor, ProviderRegistration, ProviderRegistry, ProviderSelector,
    RegistrationError, ServiceProvider, ServiceSpec, provider_registry::RegistryInner,
};

/// Mutable startup-only builder for an immutable provider registry.
///
/// Use this type to register all providers once during application assembly,
/// validate selector conflicts, and then call [`Self::build`] for a shared
/// read-only [`ProviderRegistry`].
pub struct ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Registrations retained until they are transformed into immutable entries.
    registrations: Vec<ProviderRegistration<S>>,
    /// Current canonical owner of every claimed canonical ID and alias.
    selector_owners: HashMap<ProviderSelector, crate::ProviderId>,
}

impl<S> ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Creates an empty provider-registry builder.
    ///
    /// Returns a builder with no registrations or claimed selectors.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registrations: Vec::new(),
            selector_owners: HashMap::new(),
        }
    }

    /// Registers an owned provider factory.
    ///
    /// `descriptor` supplies IDs, aliases, and priority; `provider` is moved
    /// into shared storage. Returns `Ok(())` after recording the registration.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the descriptor conflicts with an
    /// earlier registration.
    pub fn register<P>(
        &mut self,
        descriptor: ProviderDescriptor,
        provider: P,
    ) -> Result<(), RegistrationError>
    where
        P: ServiceProvider<S>,
    {
        self.register_registration(ProviderRegistration::new(descriptor, provider))
    }

    /// Registers an already shared provider factory.
    ///
    /// `descriptor` supplies IDs, aliases, and priority; `provider` is retained
    /// as the shared factory. Returns `Ok(())` after recording the registration.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the descriptor conflicts with an
    /// earlier registration.
    pub fn register_shared(
        &mut self,
        descriptor: ProviderDescriptor,
        provider: Arc<dyn ServiceProvider<S>>,
    ) -> Result<(), RegistrationError> {
        self.register_registration(ProviderRegistration::shared(descriptor, provider))
    }

    /// Builds the immutable provider catalog.
    ///
    /// Consumes this builder, preserves registration order for iteration, and
    /// computes descending-priority automatic order. Returns the immutable
    /// registry; no further registrations can be made through this builder.
    #[must_use]
    pub fn build(self) -> ProviderRegistry<S> {
        let mut entries = Vec::with_capacity(self.registrations.len());
        let mut selector_indices = HashMap::with_capacity(self.selector_owners.len());
        for (index, registration) in self.registrations.into_iter().enumerate() {
            let (descriptor, provider) = registration.into_parts();
            let canonical_selector = ProviderSelector::parse(descriptor.id().as_str())
                .expect("canonical provider IDs are valid selectors");
            selector_indices.insert(canonical_selector, index);
            for alias in descriptor.aliases() {
                selector_indices.insert(alias.clone(), index);
            }
            entries.push(crate::provider_registry::RegistryEntry {
                descriptor,
                provider,
            });
        }
        let mut automatic_indices = (0..entries.len()).collect::<Vec<_>>();
        automatic_indices.sort_unstable_by(|left, right| {
            let left = &entries[*left].descriptor;
            let right = &entries[*right].descriptor;
            right
                .priority()
                .cmp(&left.priority())
                .then_with(|| left.id().cmp(right.id()))
        });
        ProviderRegistry::from_inner(Arc::new(RegistryInner {
            entries: entries.into_boxed_slice(),
            selector_indices,
            automatic_indices: automatic_indices.into_boxed_slice(),
        }))
    }

    /// Validates and records one complete provider registration.
    ///
    /// `registration` supplies metadata and its factory. Returns `Ok(())` when
    /// all canonical IDs and aliases are unclaimed, otherwise returns the
    /// corresponding [`RegistrationError`] without modifying the builder.
    fn register_registration(
        &mut self,
        registration: ProviderRegistration<S>,
    ) -> Result<(), RegistrationError> {
        let descriptor = registration.descriptor();
        let canonical_selector = ProviderSelector::parse(descriptor.id().as_str())
            .expect("canonical provider IDs are valid selectors");
        self.validate_selector(&canonical_selector, descriptor.id().as_str())?;
        for alias in descriptor.aliases() {
            self.validate_selector(alias, descriptor.id().as_str())?;
        }
        self.selector_owners
            .insert(canonical_selector, descriptor.id().clone());
        for alias in descriptor.aliases() {
            self.selector_owners
                .insert(alias.clone(), descriptor.id().clone());
        }
        self.registrations.push(registration);
        Ok(())
    }

    /// Ensures a normalized selector has not been claimed by another provider.
    ///
    /// `selector` is the candidate ID or alias and `provider` identifies the
    /// prospective owner. Returns `Ok(())` when unclaimed, otherwise returns a
    /// [`RegistrationError`] naming the conflicting owner.
    fn validate_selector(
        &self,
        selector: &ProviderSelector,
        provider: &str,
    ) -> Result<(), RegistrationError> {
        if let Some(existing) = self.selector_owners.get(selector) {
            return Err(RegistrationError::duplicate_selector(
                selector.as_str(),
                existing.as_str(),
                provider,
            ));
        }
        Ok(())
    }
}

impl<S> Default for ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Creates an empty builder by forwarding to [`ProviderRegistryBuilder::new`].
    fn default() -> Self {
        Self::new()
    }
}
