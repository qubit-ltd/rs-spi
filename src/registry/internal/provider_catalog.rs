// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared registration, indexing, ordering, and selection algorithms.

use std::{
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
        let mut automatic_indices =
            (0..inner.entries.len()).collect::<Vec<_>>();
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

    /// Returns the current default selection snapshot.
    #[inline(always)]
    #[must_use]
    pub(crate) fn default_selection(&self) -> ProviderSelection {
        self.read_inner().default_selection.clone()
    }

    /// Replaces the selection used for future default resolutions.
    #[inline(always)]
    pub(crate) fn set_default_selection(&self, selection: ProviderSelection) {
        self.write_inner().default_selection = selection;
    }

    /// Resolves one explicit selection against a single catalog snapshot.
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
    #[must_use]
    pub(crate) fn descriptors(&self) -> Vec<ProviderDescriptor> {
        self.read_inner()
            .entries
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect()
    }

    /// Returns canonical provider IDs in successful registration order.
    #[must_use]
    pub(crate) fn provider_ids(&self) -> Vec<ProviderId> {
        self.read_inner()
            .entries
            .iter()
            .map(|entry| entry.descriptor.id().clone())
            .collect()
    }

    /// Returns the number of registered providers.
    #[inline(always)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.read_inner().entries.len()
    }

    /// Returns whether no provider is registered.
    #[inline(always)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resolves a selection while one read lock represents the catalog
    /// snapshot.
    fn resolve_from_inner(
        inner: &RegistryInner<P>,
        selection: &ProviderSelection,
    ) -> Result<ResolvedCandidates<P>, ProviderResolutionError> {
        let entries = match selection.repr() {
            ProviderSelectionRepr::Named(selector) => {
                let index =
                    inner.selector_indices.get(selector).copied().ok_or_else(
                        || {
                            ProviderResolutionError::unknown_providers(vec![
                                selector.clone(),
                            ])
                        },
                    )?;
                vec![inner.entries[index].clone()]
            }
            ProviderSelectionRepr::Chain {
                selectors,
                missing_policy,
            } => {
                let mut seen = HashSet::with_capacity(selectors.len());
                let mut candidates = Vec::with_capacity(selectors.len());
                let mut missing = Vec::new();
                for selector in selectors {
                    let Some(index) =
                        inner.selector_indices.get(selector).copied()
                    else {
                        missing.push(selector.clone());
                        continue;
                    };
                    if seen.insert(index) {
                        candidates.push(inner.entries[index].clone());
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
        Ok(ResolvedCandidates {
            entries: entries.into_boxed_slice(),
            fallback_policy: selection.fallback_policy(),
        })
    }

    /// Ensures a normalized selector is not already claimed.
    fn validate_selector(
        inner: &RegistryInner<P>,
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

    /// Acquires shared catalog state.
    #[inline(always)]
    fn read_inner(&self) -> RwLockReadGuard<'_, RegistryInner<P>> {
        self.inner.read()
    }

    /// Acquires exclusive catalog state.
    #[inline(always)]
    fn write_inner(&self) -> RwLockWriteGuard<'_, RegistryInner<P>> {
        self.inner.write()
    }
}

impl<P: ?Sized> Clone for ProviderCatalog<P> {
    /// Clones the catalog by sharing its synchronized state.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<P: ?Sized> Default for ProviderCatalog<P> {
    /// Creates an empty provider catalog.
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner::default())),
        }
    }
}
