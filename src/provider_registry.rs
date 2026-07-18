// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Runtime-mutable provider catalog and typed provider lookup.

use std::{collections::HashSet, fmt, sync::Arc};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::error::{ProviderResolutionError, RegistrationError};
use crate::internal::{ProviderSelectionRepr, RegistryEntry, RegistryInner};
use crate::{
    ProviderDefinition, ProviderDescriptor, ProviderId, ProviderSelection, ProviderSelector,
    ResolvingServiceProvider, ServiceSpec,
};

/// Shared catalog of providers for one service family.
///
/// Clones refer to the same synchronized state. Registrations and default
/// selection updates are therefore visible through every existing clone.
/// Metadata and lookup methods return owned snapshots so no registry lock is
/// held while downstream code uses the result.
pub struct ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Shared synchronized provider entries and lookup indexes.
    inner: Arc<RwLock<RegistryInner<S>>>,
}

impl<S> ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Registers an owned self-described provider.
    ///
    /// # Parameters
    ///
    /// * `provider` - Provider definition moved into shared registry storage.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the provider and all selectors are registered atomically.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the canonical ID or
    /// any alias is already registered.
    #[inline(always)]
    pub fn register<P>(&self, provider: P) -> Result<(), RegistrationError>
    where
        P: ProviderDefinition<S>,
    {
        self.register_shared(Arc::new(provider))
    }

    /// Registers an already shared self-described provider.
    ///
    /// The descriptor is obtained before the registry write lock is acquired,
    /// so provider-controlled code never runs while shared registry state is
    /// locked.
    ///
    /// # Parameters
    ///
    /// * `provider` - Shared provider definition retained by the registry.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the provider and all selectors are registered atomically.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when the canonical ID or
    /// any alias is already registered.
    pub fn register_shared(
        &self,
        provider: Arc<dyn ProviderDefinition<S>>,
    ) -> Result<(), RegistrationError> {
        let descriptor = provider.descriptor();
        let canonical_selector = ProviderSelector::from(descriptor.id());
        let mut inner = self.write_inner();

        Self::validate_selector(&inner, &canonical_selector, descriptor.id().as_str())?;
        for alias in descriptor.aliases() {
            Self::validate_selector(&inner, alias, descriptor.id().as_str())?;
        }

        let registration_index = inner.entries.len();
        inner
            .selector_indices
            .insert(canonical_selector, registration_index);
        for alias in descriptor.aliases() {
            inner
                .selector_indices
                .insert(alias.clone(), registration_index);
        }
        inner.entries.push(RegistryEntry {
            descriptor,
            provider,
        });
        let mut automatic_indices = (0..inner.entries.len()).collect::<Vec<_>>();
        automatic_indices.sort_unstable_by(|left, right| {
            let left = &inner.entries[*left].descriptor;
            let right = &inner.entries[*right].descriptor;
            right
                .priority()
                .cmp(&left.priority())
                .then_with(|| left.id().cmp(right.id()))
        });
        inner.automatic_indices = automatic_indices;
        Ok(())
    }

    /// Returns the selection used when callers request the registry default.
    ///
    /// # Returns
    ///
    /// An owned snapshot of the current default selection.
    #[inline(always)]
    #[must_use]
    pub fn default_selection(&self) -> ProviderSelection {
        self.read_inner().default_selection.clone()
    }

    /// Replaces the selection used for future default resolutions.
    ///
    /// # Parameters
    ///
    /// * `selection` - Validated selection and fallback policy to store.
    #[inline(always)]
    pub fn set_default_selection(&self, selection: ProviderSelection) {
        self.write_inner().default_selection = selection;
    }

    /// Resolves a validated selection into a composing provider snapshot.
    ///
    /// # Parameters
    ///
    /// * `selection` - Candidate target and fallback policy to resolve.
    ///
    /// # Returns
    ///
    /// A composing provider owning the selected candidates in attempt order.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] before creation when a named
    /// selector is unknown, a chain matches no candidates, or automatic
    /// selection sees an empty Registry.
    #[inline]
    pub fn resolve_selected(
        &self,
        selection: &ProviderSelection,
    ) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let inner = self.read_inner();
        Self::resolve_from_inner(&inner, selection)
    }

    /// Resolves the registry's current default selection.
    ///
    /// # Returns
    ///
    /// A composing provider owning candidates selected from one current
    /// registry snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] under the same conditions as
    /// [`Self::resolve_selected`].
    #[inline]
    pub fn resolve(&self) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let inner = self.read_inner();
        Self::resolve_from_inner(&inner, &inner.default_selection)
    }

    /// Returns descriptors in successful registration order.
    ///
    /// # Returns
    ///
    /// An owned descriptor snapshot. Later registrations do not alter it.
    #[must_use]
    pub fn descriptors(&self) -> Vec<ProviderDescriptor> {
        self.read_inner()
            .entries
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect()
    }

    /// Returns canonical provider IDs in successful registration order.
    ///
    /// # Returns
    ///
    /// An owned provider-ID snapshot. Later registrations do not alter it.
    #[must_use]
    pub fn provider_ids(&self) -> Vec<ProviderId> {
        self.read_inner()
            .entries
            .iter()
            .map(|entry| entry.descriptor.id().clone())
            .collect()
    }

    /// Returns the number of registered providers.
    ///
    /// # Returns
    ///
    /// The number of entries visible when the read lock is acquired.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.read_inner().entries.len()
    }

    /// Returns whether this registry contains no registered providers.
    ///
    /// # Returns
    ///
    /// `true` when the synchronized catalog is empty; otherwise, `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resolves one selection against already locked registry state.
    ///
    /// # Parameters
    ///
    /// * `inner` - Registry snapshot containing selectors and providers.
    /// * `selection` - Selection interpreted against that same snapshot.
    ///
    /// # Returns
    ///
    /// A composing provider owning the selected candidates in attempt order.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] when the selection has no matching
    /// candidates in `inner`.
    fn resolve_from_inner(
        inner: &RegistryInner<S>,
        selection: &ProviderSelection,
    ) -> Result<ResolvingServiceProvider<S>, ProviderResolutionError> {
        let candidates = match selection.repr() {
            ProviderSelectionRepr::Named(selector) => {
                let index = inner
                    .selector_indices
                    .get(selector)
                    .copied()
                    .ok_or_else(|| ProviderResolutionError::unknown_provider(selector.clone()))?;
                vec![inner.entries[index].clone()]
            }
            ProviderSelectionRepr::Chain(selectors) => {
                let mut seen = HashSet::with_capacity(selectors.len());
                let mut candidates = Vec::with_capacity(selectors.len());
                for selector in selectors {
                    let Some(index) = inner.selector_indices.get(selector).copied() else {
                        continue;
                    };
                    if seen.insert(index) {
                        candidates.push(inner.entries[index].clone());
                    }
                }
                if candidates.is_empty() {
                    return Err(ProviderResolutionError::no_candidates(selectors.to_vec()));
                }
                candidates
            }
            ProviderSelectionRepr::Auto => {
                if inner.automatic_indices.is_empty() {
                    return Err(ProviderResolutionError::empty_registry());
                }
                inner
                    .automatic_indices
                    .iter()
                    .map(|index| inner.entries[*index].clone())
                    .collect()
            }
        };
        Ok(ResolvingServiceProvider::new(
            candidates,
            selection.fallback_policy(),
        ))
    }

    /// Ensures a normalized selector is not already claimed.
    ///
    /// # Parameters
    ///
    /// * `inner` - Locked registry state inspected without mutation.
    /// * `selector` - Candidate canonical ID or alias.
    /// * `provider` - Canonical ID attempting to claim the selector.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the selector is unclaimed.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] naming both owners when the selector is
    /// already claimed.
    fn validate_selector(
        inner: &RegistryInner<S>,
        selector: &ProviderSelector,
        provider: &str,
    ) -> Result<(), RegistrationError> {
        let Some(existing) = inner
            .selector_indices
            .get(selector)
            .and_then(|index| inner.entries.get(*index))
        else {
            return Ok(());
        };
        Err(RegistrationError::duplicate_selector(
            selector.as_str(),
            existing.descriptor.id().as_str(),
            provider,
        ))
    }

    /// Acquires shared registry state.
    ///
    /// # Returns
    ///
    /// A read guard protecting the current registry state.
    #[inline(always)]
    fn read_inner(&self) -> RwLockReadGuard<'_, RegistryInner<S>> {
        self.inner.read()
    }

    /// Acquires exclusive registry state.
    ///
    /// # Returns
    ///
    /// A write guard protecting the current registry state.
    #[inline(always)]
    fn write_inner(&self) -> RwLockWriteGuard<'_, RegistryInner<S>> {
        self.inner.write()
    }
}

impl<S> Clone for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Clones the registry by incrementing its shared-state reference count.
    ///
    /// # Returns
    ///
    /// Another handle observing the same registrations and default selection.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<S> Default for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Creates an empty runtime registry.
    ///
    /// # Returns
    ///
    /// A synchronized catalog with automatic default selection.
    #[inline]
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner::default())),
        }
    }
}

impl<S> fmt::Debug for ProviderRegistry<S>
where
    S: ServiceSpec,
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
        formatter
            .debug_struct("ProviderRegistry")
            .field("descriptors", &self.descriptors())
            .field("default_selection", &self.default_selection())
            .finish()
    }
}
