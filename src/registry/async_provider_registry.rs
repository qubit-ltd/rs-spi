// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous-provider registry with synchronous catalog operations.

use std::{
    fmt,
    sync::Arc,
};

use crate::error::{
    ProviderResolutionError,
    RegistrationError,
};
use crate::registry::internal::ProviderCatalog;
use crate::{
    AsyncProviderDefinition,
    AsyncResolvingServiceProvider,
    AsyncServiceSpec,
    ProviderDescriptor,
    ProviderId,
    ProviderSelection,
};

/// Shared catalog of asynchronous providers for one service family.
///
/// Registration, metadata lookup, default selection, and resolution are all
/// synchronous. Only service creation through the returned resolver is
/// asynchronous.
///
/// # Type Parameters
///
/// * `S` - Asynchronous service family whose providers are registered.
pub struct AsyncProviderRegistry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Shared mode-independent provider catalog.
    providers: ProviderCatalog<dyn AsyncProviderDefinition<S>>,
}

impl<S> AsyncProviderRegistry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Registers an owned asynchronous provider.
    ///
    /// # Type Parameters
    ///
    /// * `P` - Concrete asynchronous provider transferred into the Registry.
    ///
    /// # Parameters
    ///
    /// * `provider` - Provider to register and share through the Registry.
    ///
    /// # Returns
    ///
    /// `Ok(())` after successful registration.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the provider's
    /// canonical ID or any alias is already registered.
    #[inline]
    pub fn register<P>(&self, provider: P) -> Result<(), RegistrationError>
    where
        P: AsyncProviderDefinition<S>,
    {
        let provider: Arc<dyn AsyncProviderDefinition<S>> = Arc::new(provider);
        self.providers.register_shared(provider)
    }

    /// Registers an already shared asynchronous provider.
    ///
    /// # Parameters
    ///
    /// * `provider` - Shared asynchronous provider definition to register.
    ///
    /// # Returns
    ///
    /// `Ok(())` after successful registration.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the provider's
    /// canonical ID or any alias is already registered.
    #[inline(always)]
    pub fn register_shared(
        &self,
        provider: Arc<dyn AsyncProviderDefinition<S>>,
    ) -> Result<(), RegistrationError> {
        self.providers.register_shared(provider)
    }

    /// Returns the selection used by [`Self::resolve`].
    ///
    /// # Returns
    ///
    /// A snapshot of the Registry's current default selection.
    #[inline(always)]
    #[must_use]
    pub fn default_selection(&self) -> ProviderSelection {
        self.providers.default_selection()
    }

    /// Replaces the selection used by future [`Self::resolve`] calls.
    ///
    /// # Parameters
    ///
    /// * `selection` - New default selection stored by the Registry.
    #[inline(always)]
    pub fn set_default_selection(&self, selection: ProviderSelection) {
        self.providers.set_default_selection(selection);
    }

    /// Resolves an explicit selection into an asynchronous candidate snapshot.
    ///
    /// This function performs no asynchronous work.
    ///
    /// # Parameters
    ///
    /// * `selection` - Validated selection to resolve.
    ///
    /// # Returns
    ///
    /// A resolver owning the selected provider snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] when the selection cannot be
    /// resolved to a nonempty candidate snapshot.
    pub fn resolve_selected(
        &self,
        selection: &ProviderSelection,
    ) -> Result<AsyncResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = self.providers.resolve_selected(selection)?;
        Ok(AsyncResolvingServiceProvider::new(
            candidates.entries,
            candidates.fallback_policy,
        ))
    }

    /// Resolves the current default selection without asynchronous work.
    ///
    /// # Returns
    ///
    /// A resolver owning the selected provider snapshot.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::resolve_selected`].
    pub fn resolve(
        &self,
    ) -> Result<AsyncResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = self.providers.resolve()?;
        Ok(AsyncResolvingServiceProvider::new(
            candidates.entries,
            candidates.fallback_policy,
        ))
    }

    /// Returns descriptors in successful registration order.
    ///
    /// # Returns
    ///
    /// Owned descriptor snapshots in registration order.
    #[inline(always)]
    #[must_use]
    pub fn descriptors(&self) -> Vec<ProviderDescriptor> {
        self.providers.descriptors()
    }

    /// Returns canonical provider IDs in successful registration order.
    ///
    /// # Returns
    ///
    /// Owned canonical IDs in registration order.
    #[inline(always)]
    #[must_use]
    pub fn provider_ids(&self) -> Vec<ProviderId> {
        self.providers.provider_ids()
    }

    /// Returns the number of registered providers.
    ///
    /// # Returns
    ///
    /// The number of successful registrations.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Returns whether no provider is registered.
    ///
    /// # Returns
    ///
    /// `true` when the Registry contains no provider; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl<S> Clone for AsyncProviderRegistry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Clones this facade by sharing its provider catalog.
    ///
    /// # Returns
    ///
    /// A Registry facade observing the same catalog state.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
        }
    }
}

impl<S> Default for AsyncProviderRegistry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Creates an empty asynchronous-provider registry.
    ///
    /// # Returns
    ///
    /// An empty Registry using automatic default selection.
    #[inline]
    fn default() -> Self {
        Self {
            providers: ProviderCatalog::default(),
        }
    }
}

impl<S> fmt::Debug for AsyncProviderRegistry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Formats owned snapshots of registry metadata.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects debug output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (descriptors, default_selection) =
            self.providers.metadata_snapshot();
        formatter
            .debug_struct("AsyncProviderRegistry")
            .field("descriptors", &descriptors)
            .field("default_selection", &default_selection)
            .finish()
    }
}
