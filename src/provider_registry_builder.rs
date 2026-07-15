// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Startup-only assembly of immutable provider registries.

use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::internal::{
    BuilderEntry,
    RegistryEntry,
    RegistryInner,
};
use crate::{
    ProviderDescriptor,
    ProviderRegistry,
    ProviderSelector,
    RegistrationError,
    ServiceProvider,
    ServiceSpec,
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
    /// Registrations retained until they are transformed into immutable
    /// entries.
    registrations: Vec<BuilderEntry<S>>,
    /// Mapping from every claimed selector to its pending registration index.
    selector_indices: HashMap<ProviderSelector, usize>,
}

impl<S> ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Creates an empty provider-registry builder.
    ///
    /// # Returns
    ///
    /// A builder with no registrations or claimed selectors.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            registrations: Vec::new(),
            selector_indices: HashMap::new(),
        }
    }

    /// Registers an owned provider factory.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - Provider ID, aliases, and automatic priority.
    /// * `provider` - Owned factory moved into shared registry storage.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the registration and selector claims are recorded.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the descriptor conflicts with an
    /// earlier registration.
    #[inline(always)]
    pub fn register<P>(
        &mut self,
        descriptor: ProviderDescriptor,
        provider: P,
    ) -> Result<(), RegistrationError>
    where
        P: ServiceProvider<S>,
    {
        self.register_shared(descriptor, Arc::new(provider))
    }

    /// Registers an already shared provider factory.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - Provider ID, aliases, and automatic priority.
    /// * `provider` - Shared factory retained by the immutable registry.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the registration and selector claims are recorded.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the descriptor conflicts with an
    /// earlier registration.
    #[inline(always)]
    pub fn register_shared(
        &mut self,
        descriptor: ProviderDescriptor,
        provider: Arc<dyn ServiceProvider<S>>,
    ) -> Result<(), RegistrationError> {
        self.insert(descriptor, provider)
    }

    /// Builds the immutable provider catalog.
    ///
    /// # Returns
    ///
    /// A registry preserving registration order, the already validated
    /// selector index, and deterministic automatic-selection order.
    #[must_use]
    pub fn build(self) -> ProviderRegistry<S> {
        let Self {
            registrations,
            selector_indices,
        } = self;
        let mut entries = Vec::with_capacity(registrations.len());
        for BuilderEntry {
            descriptor,
            provider,
        } in registrations
        {
            entries.push(RegistryEntry {
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

    /// Validates and records one descriptor and factory.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - Provider metadata whose selectors must be unclaimed.
    /// * `provider` - Shared factory stored after validation succeeds.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the descriptor, factory, and selector indexes are stored.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without modifying the builder when any
    /// selector is already registered.
    ///
    /// # Panics
    ///
    /// Panics only if a previously validated canonical provider ID cannot be
    /// parsed as a selector, which safe construction cannot produce.
    fn insert(
        &mut self,
        descriptor: ProviderDescriptor,
        provider: Arc<dyn ServiceProvider<S>>,
    ) -> Result<(), RegistrationError> {
        let canonical_selector =
            ProviderSelector::parse(descriptor.id().as_str())
                .expect("canonical provider IDs are valid selectors");
        self.validate_selector(&canonical_selector, descriptor.id().as_str())?;
        for alias in descriptor.aliases() {
            self.validate_selector(alias, descriptor.id().as_str())?;
        }

        let registration_index = self.registrations.len();
        self.selector_indices
            .insert(canonical_selector, registration_index);
        for alias in descriptor.aliases() {
            self.selector_indices
                .insert(alias.clone(), registration_index);
        }
        self.registrations.push(BuilderEntry {
            descriptor,
            provider,
        });
        Ok(())
    }

    /// Ensures a normalized selector has not been claimed by another provider.
    ///
    /// # Arguments
    ///
    /// * `selector` - Candidate canonical ID or alias.
    /// * `provider` - Canonical ID attempting to claim the selector.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the selector is unclaimed.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] naming the existing owner when claimed.
    ///
    /// # Panics
    ///
    /// Panics only if the builder's private selector index refers outside its
    /// registration vector, which public builder operations cannot produce.
    #[inline]
    fn validate_selector(
        &self,
        selector: &ProviderSelector,
        provider: &str,
    ) -> Result<(), RegistrationError> {
        if let Some(existing_index) = self.selector_indices.get(selector) {
            let existing = self.registrations[*existing_index].descriptor.id();
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
    /// Creates an empty provider-registry builder.
    ///
    /// # Returns
    ///
    /// A builder with no registrations or claimed selectors.
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
