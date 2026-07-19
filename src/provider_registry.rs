// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Synchronous provider registry facade.

use std::{
    fmt,
    sync::Arc,
};

use crate::error::{
    ProviderResolutionError,
    RegistrationError,
};
use crate::internal::ProviderCatalog;
use crate::{
    ProviderDefinition,
    ProviderDescriptor,
    ProviderId,
    ProviderSelection,
    ResolvingServiceProvider,
    SyncServiceSpec,
};

/// Shared catalog of synchronous providers for one service family.
///
/// Clones observe the same registrations and default selection. Catalog locks
/// are released before a resolver invokes any provider.
pub struct ProviderRegistry<S>
where
    S: SyncServiceSpec,
{
    /// Shared mode-independent provider catalog.
    providers: ProviderCatalog<dyn ProviderDefinition<S>>,
}

impl<S> ProviderRegistry<S>
where
    S: SyncServiceSpec,
{
    /// Registers an owned synchronous provider.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the provider's
    /// canonical ID or any alias is already registered.
    pub fn register<P>(&self, provider: P) -> Result<(), RegistrationError>
    where
        P: ProviderDefinition<S>,
    {
        let provider: Arc<dyn ProviderDefinition<S>> = Arc::new(provider);
        self.providers.register_shared(provider)
    }

    /// Registers an already shared synchronous provider.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the provider's
    /// canonical ID or any alias is already registered.
    #[inline(always)]
    pub fn register_shared(
        &self,
        provider: Arc<dyn ProviderDefinition<S>>,
    ) -> Result<(), RegistrationError> {
        self.providers.register_shared(provider)
    }

    /// Returns the selection used by [`Self::resolve`].
    #[inline(always)]
    #[must_use]
    pub fn default_selection(&self) -> ProviderSelection {
        self.providers.default_selection()
    }

    /// Replaces the selection used by future [`Self::resolve`] calls.
    #[inline(always)]
    pub fn set_default_selection(&self, selection: ProviderSelection) {
        self.providers.set_default_selection(selection);
    }

    /// Resolves an explicit selection into a synchronous candidate snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] when the selection cannot be
    /// resolved to a nonempty candidate snapshot.
    pub fn resolve_selected(
        &self,
        selection: &ProviderSelection,
    ) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = self.providers.resolve_selected(selection)?;
        Ok(ResolvingServiceProvider::new(
            candidates.entries,
            candidates.fallback_policy,
        ))
    }

    /// Resolves the current default selection.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::resolve_selected`].
    pub fn resolve(
        &self,
    ) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = self.providers.resolve()?;
        Ok(ResolvingServiceProvider::new(
            candidates.entries,
            candidates.fallback_policy,
        ))
    }

    /// Returns descriptors in successful registration order.
    #[inline(always)]
    #[must_use]
    pub fn descriptors(&self) -> Vec<ProviderDescriptor> {
        self.providers.descriptors()
    }

    /// Returns canonical provider IDs in successful registration order.
    #[inline(always)]
    #[must_use]
    pub fn provider_ids(&self) -> Vec<ProviderId> {
        self.providers.provider_ids()
    }

    /// Returns the number of registered providers.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Returns whether no provider is registered.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl<S> Clone for ProviderRegistry<S>
where
    S: SyncServiceSpec,
{
    /// Clones this facade by sharing its provider catalog.
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
        }
    }
}

impl<S> Default for ProviderRegistry<S>
where
    S: SyncServiceSpec,
{
    /// Creates an empty synchronous provider registry.
    fn default() -> Self {
        Self {
            providers: ProviderCatalog::default(),
        }
    }
}

impl<S> fmt::Debug for ProviderRegistry<S>
where
    S: SyncServiceSpec,
{
    /// Formats owned snapshots of registry metadata.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderRegistry")
            .field("descriptors", &self.descriptors())
            .field("default_selection", &self.default_selection())
            .finish()
    }
}
