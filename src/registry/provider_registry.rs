// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Synchronous provider registry facade.

use std::fmt;
use std::sync::Arc;

use crate::ProviderDefinition;
use crate::ProviderDescriptor;
use crate::ProviderId;
use crate::ProviderSelection;
use crate::ResolvingServiceProvider;
use crate::SyncServiceSpec;
use crate::error::ProviderResolutionError;
use crate::error::RegistrationError;
use crate::registry::internal::ProviderCatalog;

/// Shared catalog of synchronous providers for one service family.
///
/// Clones observe the same registrations and default selection. Catalog locks
/// are released before a resolver invokes any provider.
///
/// # Type Parameters
///
/// * `S` - Synchronous service family whose providers are registered.
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
    /// # Type Parameters
    ///
    /// * `P` - Concrete provider definition transferred into the Registry.
    ///
    /// # Parameters
    ///
    /// * `provider` - Provider to register and share through the Registry.
    ///
    /// # Returns
    ///
    /// `Ok(())` after successful registration.
    ///
    /// Registration snapshots the descriptor returned by
    /// [`crate::ProviderMetadata::descriptor`] before acquiring the Registry's
    /// write lock and before validating selector conflicts.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the provider's
    /// canonical ID or any alias is already registered.
    ///
    /// # Panics
    ///
    /// Propagates a panic raised while obtaining the provider descriptor. The
    /// Registry remains unchanged because descriptor generation precedes all
    /// mutation.
    #[inline]
    pub fn register<P>(&self, provider: P) -> Result<(), RegistrationError>
    where
        P: ProviderDefinition<S>,
    {
        let provider: Arc<dyn ProviderDefinition<S>> = Arc::new(provider);
        self.providers.register_shared(provider)
    }

    /// Registers an already shared synchronous provider.
    ///
    /// # Parameters
    ///
    /// * `provider` - Shared provider definition to register.
    ///
    /// # Returns
    ///
    /// `Ok(())` after successful registration.
    ///
    /// Registration snapshots the descriptor returned by
    /// [`crate::ProviderMetadata::descriptor`] before acquiring the Registry's
    /// write lock and before validating selector conflicts.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the provider's
    /// canonical ID or any alias is already registered.
    ///
    /// # Panics
    ///
    /// Propagates a panic raised while obtaining the provider descriptor. The
    /// Registry remains unchanged because descriptor generation precedes all
    /// mutation.
    #[inline(always)]
    pub fn register_shared(&self, provider: Arc<dyn ProviderDefinition<S>>) -> Result<(), RegistrationError> {
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

    /// Resolves an explicit selection into a synchronous candidate snapshot.
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
    ) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = self.providers.resolve_selected(selection)?;
        Ok(ResolvingServiceProvider::new(
            candidates.entries,
            candidates.fallback_policy,
        ))
    }

    /// Resolves the current default selection.
    ///
    /// # Returns
    ///
    /// A resolver owning the selected provider snapshot.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::resolve_selected`].
    pub fn resolve(&self) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = self.providers.resolve()?;
        Ok(ResolvingServiceProvider::new(
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

impl<S> Clone for ProviderRegistry<S>
where
    S: SyncServiceSpec,
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

impl<S> Default for ProviderRegistry<S>
where
    S: SyncServiceSpec,
{
    /// Creates an empty synchronous provider registry.
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

impl<S> fmt::Debug for ProviderRegistry<S>
where
    S: SyncServiceSpec,
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
        let (descriptors, default_selection) = self.providers.metadata_snapshot();
        formatter
            .debug_struct("ProviderRegistry")
            .field("descriptors", &descriptors)
            .field("default_selection", &default_selection)
            .finish()
    }
}
