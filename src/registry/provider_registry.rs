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
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, SyncServiceSpec};
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = String;
///     type Error = std::io::Error;
/// }
/// impl SyncServiceSpec for Spec { type Output = String; }
/// use qubit_spi::{ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider};
/// use qubit_spi::error::ProviderFailure;
/// struct Echo;
/// impl ProviderMetadata for Echo {
///     fn descriptor(&self) -> ProviderDescriptor {
///         ProviderDescriptor::new(ProviderId::new("echo").expect("valid static ID"))
///     }
/// }
/// impl ServiceProvider<Spec> for Echo {
///     fn create_configured(&self, config: &String) -> Result<String, ProviderFailure<std::io::Error>> {
///         Ok(config.clone())
///     }
/// }
/// let registry = qubit_spi::ProviderRegistry::<Spec>::default();
/// registry.register(Echo)?;
/// let resolver = registry.resolve()?;
/// assert_eq!("hello", resolver.create_configured(&"hello".to_owned())?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// Returns [`RegistrationError`] without inserting this provider when its
    /// canonical ID or any alias is already registered. Reentrant changes
    /// made by the metadata callback are not rolled back.
    ///
    /// # Panics
    ///
    /// Propagates a panic raised while obtaining the provider descriptor. The
    /// attempted registration is not applied. Changes made by reentrant
    /// metadata callbacks are not rolled back.
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
    /// Returns [`RegistrationError`] without inserting this provider when its
    /// canonical ID or any alias is already registered. Reentrant changes
    /// made by the metadata callback are not rolled back.
    ///
    /// # Panics
    ///
    /// Propagates a panic raised while obtaining the provider descriptor. The
    /// attempted registration is not applied. Changes made by reentrant
    /// metadata callbacks are not rolled back.
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

    /// Resolves the current default selection and returns the captured
    /// selection together with its provider snapshot.
    ///
    /// Both values are captured from one catalog read snapshot. This is useful
    /// to callers that must validate configuration against the exact default
    /// selection whose candidates they will create from.
    ///
    /// # Returns
    ///
    /// The captured default selection and an owned resolver for its candidates.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::resolve_selected`].
    pub fn resolve_default_snapshot(
        &self,
    ) -> (
        ProviderSelection,
        Result<ResolvingServiceProvider<S>, ProviderResolutionError>,
    ) {
        let (selection, candidates) = self.providers.resolve_default_snapshot();
        let resolver =
            candidates.map(|candidates| ResolvingServiceProvider::new(candidates.entries, candidates.fallback_policy));
        (selection, resolver)
    }

    /// Resolves the current default selection.
    ///
    /// Captures the current default and returns its candidate resolver.
    ///
    /// # Returns
    ///
    /// An owned candidate snapshot governed by the captured default policy.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] if the captured default selects no
    /// candidates or requires an unknown selector.
    pub fn resolve(&self) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let (_, result) = self.resolve_default_snapshot();
        result
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
