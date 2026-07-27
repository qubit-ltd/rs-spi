// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared registration, indexing, ordering, and selection algorithms.

use std::{
    cmp::Reverse,
    collections::HashSet,
    sync::Arc,
};

use parking_lot::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};

use crate::error::{
    ProviderResolutionError,
    RegistrationError,
};
use crate::selection::ProviderSelectionRepr;
use crate::{
    MissingProviderPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderMetadata,
    ProviderSelection,
    ProviderSelector,
};

use super::{
    RegistryEntry,
    RegistryInner,
    ResolvedCandidates,
};

/// Shared provider catalog independent of creation mode.
///
/// # Type Parameters
///
/// * `P` - Possibly unsized provider metadata contract stored by the catalog.
pub(crate) struct ProviderCatalog<P: ?Sized> {
    /// Synchronized provider entries and lookup indexes.
    inner: Arc<RwLock<RegistryInner<P>>>,
}

impl<P> ProviderCatalog<P>
where
    P: ProviderMetadata + ?Sized,
{
    /// Registers an already shared provider definition atomically.
    ///
    /// Provider-controlled metadata is obtained before the write lock is
    /// acquired.
    ///
    /// # Parameters
    ///
    /// * `provider` - Shared provider definition to register.
    ///
    /// # Returns
    ///
    /// `Ok(())` after successful registration.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] without mutation when any selector is
    /// already registered.
    pub(crate) fn register_shared(
        &self,
        provider: Arc<P>,
    ) -> Result<(), RegistrationError> {
        let descriptor = provider.descriptor();
        let canonical_selector = ProviderSelector::from(descriptor.id());
        let mut inner = self.write_inner();

        Self::validate_selector(
            &inner,
            &canonical_selector,
            descriptor.id().as_str(),
        )?;
        for alias in descriptor.aliases() {
            Self::validate_selector(&inner, alias, descriptor.id().as_str())?;
        }

        let provider_id = descriptor.id().clone();
        inner
            .selector_ids
            .insert(canonical_selector, provider_id.clone());
        for alias in descriptor.aliases() {
            inner
                .selector_ids
                .insert(alias.clone(), provider_id.clone());
        }
        let priority = descriptor.priority();
        let inserted = inner.entries.try_insert(
            provider_id.clone(),
            (Reverse(priority), provider_id.clone()),
            RegistryEntry {
                descriptor: Arc::new(descriptor),
                provider,
            },
        );
        assert!(inserted.is_ok(), "validated provider ID must be unique");
        inner.registration_ids.push(provider_id);
        Ok(())
    }

    /// Returns descriptors and the default selection from one catalog snapshot.
    ///
    /// # Returns
    ///
    /// Owned provider descriptors and default selection cloned while one read
    /// lock represents the catalog state.
    #[must_use]
    pub(crate) fn metadata_snapshot(
        &self,
    ) -> (Vec<ProviderDescriptor>, ProviderSelection) {
        let inner = self.read_inner();
        let descriptors = inner
            .registration_ids
            .iter()
            .map(|provider_id| {
                inner
                    .entries
                    .get(provider_id)
                    .expect("registered provider ID must have an entry")
                    .descriptor
                    .as_ref()
                    .clone()
            })
            .collect();
        let default_selection = inner.default_selection.clone();
        (descriptors, default_selection)
    }

    /// Returns the current default selection snapshot.
    ///
    /// # Returns
    ///
    /// An owned snapshot of the current default selection.
    #[inline(always)]
    #[must_use]
    pub(crate) fn default_selection(&self) -> ProviderSelection {
        self.read_inner().default_selection.clone()
    }

    /// Replaces the selection used for future default resolutions.
    ///
    /// # Parameters
    ///
    /// * `selection` - New default selection stored in the catalog.
    #[inline(always)]
    pub(crate) fn set_default_selection(&self, selection: ProviderSelection) {
        self.write_inner().default_selection = selection;
    }

    /// Resolves one explicit selection against a single catalog snapshot.
    ///
    /// # Parameters
    ///
    /// * `selection` - Validated selection to resolve.
    ///
    /// # Returns
    ///
    /// A nonempty candidate snapshot and its fallback policy.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] when the selection cannot be
    /// resolved to a nonempty candidate snapshot.
    pub(crate) fn resolve_selected(
        &self,
        selection: &ProviderSelection,
    ) -> Result<ResolvedCandidates<P>, ProviderResolutionError> {
        Self::resolve_from_inner(&self.read_inner(), selection)
    }

    /// Resolves the current default selection against one catalog snapshot.
    ///
    /// # Returns
    ///
    /// A nonempty candidate snapshot and its fallback policy.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::resolve_selected`].
    pub(crate) fn resolve(
        &self,
    ) -> Result<ResolvedCandidates<P>, ProviderResolutionError> {
        let inner = self.read_inner();
        Self::resolve_from_inner(&inner, &inner.default_selection)
    }

    /// Returns descriptors in successful registration order.
    ///
    /// # Returns
    ///
    /// Owned descriptor snapshots in registration order.
    #[must_use]
    pub(crate) fn descriptors(&self) -> Vec<ProviderDescriptor> {
        let inner = self.read_inner();
        inner
            .registration_ids
            .iter()
            .map(|provider_id| {
                inner
                    .entries
                    .get(provider_id)
                    .expect("registered provider ID must have an entry")
                    .descriptor
                    .as_ref()
                    .clone()
            })
            .collect()
    }

    /// Returns canonical provider IDs in successful registration order.
    ///
    /// # Returns
    ///
    /// Owned canonical IDs in registration order.
    #[must_use]
    pub(crate) fn provider_ids(&self) -> Vec<ProviderId> {
        self.read_inner().registration_ids.clone()
    }

    /// Returns the number of registered providers.
    ///
    /// # Returns
    ///
    /// The number of successful registrations.
    #[inline(always)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.read_inner().entries.len()
    }

    /// Returns whether no provider is registered.
    ///
    /// # Returns
    ///
    /// `true` when the catalog has no provider; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resolves a selection while one read lock represents the catalog
    /// snapshot.
    ///
    /// # Parameters
    ///
    /// * `inner` - Locked catalog state used as one consistent snapshot.
    /// * `selection` - Validated selection to resolve.
    ///
    /// # Returns
    ///
    /// A nonempty candidate snapshot and its fallback policy.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderResolutionError`] when required providers are absent,
    /// no lenient-chain candidate exists, or automatic selection is empty.
    fn resolve_from_inner(
        inner: &RegistryInner<P>,
        selection: &ProviderSelection,
    ) -> Result<ResolvedCandidates<P>, ProviderResolutionError> {
        let entries = match selection.repr() {
            ProviderSelectionRepr::Named(selector) => {
                let provider_id =
                    inner.selector_ids.get(selector).ok_or_else(|| {
                        ProviderResolutionError::unknown_providers(vec![
                            selector.clone(),
                        ])
                    })?;
                vec![
                    inner
                        .entries
                        .get(provider_id)
                        .expect("registered selector must have an entry")
                        .clone(),
                ]
            }
            ProviderSelectionRepr::Chain {
                selectors,
                missing_policy,
            } => {
                let mut seen = HashSet::with_capacity(selectors.len());
                let mut candidates = Vec::with_capacity(selectors.len());
                let mut missing = Vec::new();
                for selector in selectors {
                    let Some(provider_id) = inner.selector_ids.get(selector)
                    else {
                        missing.push(selector.clone());
                        continue;
                    };
                    if seen.insert(provider_id.clone()) {
                        candidates.push(
                            inner
                                .entries
                                .get(provider_id)
                                .expect(
                                    "registered selector must have an entry",
                                )
                                .clone(),
                        );
                    }
                }
                if *missing_policy == MissingProviderPolicy::Reject
                    && !missing.is_empty()
                {
                    return Err(ProviderResolutionError::unknown_providers(
                        missing,
                    ));
                }
                if candidates.is_empty() {
                    return Err(ProviderResolutionError::no_candidates(
                        selectors.to_vec(),
                    ));
                }
                candidates
            }
            ProviderSelectionRepr::Auto => {
                if inner.entries.is_empty() {
                    return Err(ProviderResolutionError::empty_registry());
                }
                inner.entries.values_ordered().cloned().collect()
            }
        };
        Ok(ResolvedCandidates {
            entries: entries.into_boxed_slice(),
            fallback_policy: selection.fallback_policy(),
        })
    }

    /// Ensures a normalized selector is not already claimed.
    ///
    /// # Parameters
    ///
    /// * `inner` - Catalog state whose selector index is checked.
    /// * `selector` - Normalized selector proposed by the new provider.
    /// * `provider` - Canonical ID of the provider being registered.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the selector is unclaimed.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] naming both providers when the selector
    /// is already claimed.
    fn validate_selector(
        inner: &RegistryInner<P>,
        selector: &ProviderSelector,
        provider: &str,
    ) -> Result<(), RegistrationError> {
        let Some(existing) = inner
            .selector_ids
            .get(selector)
            .and_then(|provider_id| inner.entries.get(provider_id))
        else {
            return Ok(());
        };
        Err(RegistrationError::duplicate_selector(
            selector.as_str(),
            existing.descriptor.id().as_str(),
            provider,
        ))
    }

    /// Acquires shared catalog state.
    ///
    /// # Returns
    ///
    /// A read guard held until the caller drops it.
    #[inline(always)]
    fn read_inner(&self) -> RwLockReadGuard<'_, RegistryInner<P>> {
        self.inner.read()
    }

    /// Acquires exclusive catalog state.
    ///
    /// # Returns
    ///
    /// A write guard held until the caller drops it.
    #[inline(always)]
    fn write_inner(&self) -> RwLockWriteGuard<'_, RegistryInner<P>> {
        self.inner.write()
    }
}

impl<P: ?Sized> Clone for ProviderCatalog<P> {
    /// Clones the catalog by sharing its synchronized state.
    ///
    /// # Returns
    ///
    /// A catalog handle observing the same synchronized state.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<P: ?Sized> Default for ProviderCatalog<P> {
    /// Creates an empty provider catalog.
    ///
    /// # Returns
    ///
    /// An empty catalog using automatic default selection.
    #[inline]
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner::default())),
        }
    }
}
