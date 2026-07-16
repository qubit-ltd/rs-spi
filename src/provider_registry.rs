// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable provider catalog and typed provider lookup.

use std::{
    fmt,
    sync::Arc,
};

use crate::error::ResolutionError;
use crate::internal::RegistryInner;
use crate::{
    ProviderDescriptor,
    ProviderId,
    ProviderRegistryBuilder,
    ProviderSelector,
    ResolvedProvider,
    ServiceSpec,
};

/// Immutable catalog of explicitly registered providers.
///
/// Build a registry during application startup, then share or clone it freely
/// for read-only ID/alias lookup and service resolution.
pub struct ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Shared immutable storage for all provider entries and lookup indexes.
    inner: Arc<RegistryInner<S>>,
}

impl<S> ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Creates an empty builder for this service specification.
    ///
    /// # Returns
    ///
    /// A mutable builder used during startup to register providers.
    #[inline(always)]
    #[must_use]
    pub fn builder() -> ProviderRegistryBuilder<S> {
        ProviderRegistryBuilder::new()
    }

    /// Creates a registry from prepared immutable internal storage.
    ///
    /// # Arguments
    ///
    /// * `inner` - Mutually consistent entries and registry-owned indexes.
    ///
    /// # Returns
    ///
    /// A registry sharing the supplied immutable storage.
    #[inline]
    #[must_use]
    pub(crate) fn from_inner(inner: Arc<RegistryInner<S>>) -> Self {
        Self { inner }
    }

    /// Resolves a canonical provider ID or alias.
    ///
    /// # Arguments
    ///
    /// * `selector` - Raw selector normalized and validated before lookup.
    ///
    /// # Returns
    ///
    /// The matching provider together with its descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when `selector` is invalid or does not name
    /// a registered provider.
    #[inline]
    pub fn resolve(
        &self,
        selector: impl AsRef<str>,
    ) -> Result<ResolvedProvider<'_, S>, ResolutionError> {
        let selector = ProviderSelector::parse(selector).map_err(|source| {
            ResolutionError::invalid_selector(None, source)
        })?;
        self.resolve_selector(&selector)
            .ok_or_else(|| ResolutionError::unknown_provider(selector))
    }

    /// Finds a provider by canonical ID or alias without constructing an error.
    ///
    /// # Arguments
    ///
    /// * `selector` - Raw selector normalized and validated before lookup.
    ///
    /// # Returns
    ///
    /// `Some` with the matching provider, or `None` for invalid or unknown
    /// input.
    #[inline]
    #[must_use]
    pub fn find(
        &self,
        selector: impl AsRef<str>,
    ) -> Option<ResolvedProvider<'_, S>> {
        ProviderSelector::parse(selector)
            .ok()
            .and_then(|selector| self.resolve_selector(&selector))
    }

    /// Iterates over descriptors in registration order.
    ///
    /// # Returns
    ///
    /// An exact-size iterator borrowing each immutable descriptor once.
    #[inline(always)]
    pub fn descriptors(
        &self,
    ) -> impl ExactSizeIterator<Item = &ProviderDescriptor> {
        self.inner.entries.iter().map(|entry| &entry.descriptor)
    }

    /// Iterates over canonical provider IDs in registration order.
    ///
    /// # Returns
    ///
    /// An exact-size iterator borrowing every canonical provider ID once.
    #[inline(always)]
    pub fn provider_ids(&self) -> impl ExactSizeIterator<Item = &ProviderId> {
        self.inner.entries.iter().map(|entry| entry.descriptor.id())
    }

    /// Returns the number of registered providers.
    ///
    /// # Returns
    ///
    /// The number of immutable registry entries.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.entries.len()
    }

    /// Returns whether this registry contains no registered providers.
    ///
    /// # Returns
    ///
    /// `true` when the registry has no entries; otherwise, `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Looks up the entry position for a normalized selector.
    ///
    /// # Arguments
    ///
    /// * `selector` - Valid normalized selector used as an index key.
    ///
    /// # Returns
    ///
    /// `Some` with the internal entry position, or `None` when unregistered.
    #[inline(always)]
    pub(crate) fn index_for(
        &self,
        selector: &ProviderSelector,
    ) -> Option<usize> {
        self.inner.selector_indices.get(selector).copied()
    }

    /// Borrows the resolved provider at an internal entry position.
    ///
    /// # Arguments
    ///
    /// * `index` - Registry-owned entry position.
    ///
    /// # Returns
    ///
    /// A lookup wrapper borrowing the indexed entry.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the registry entry array. Registry-owned
    /// indexes satisfy this invariant.
    #[inline(always)]
    pub(crate) fn resolved_at(&self, index: usize) -> ResolvedProvider<'_, S> {
        ResolvedProvider::new(&self.inner.entries[index])
    }

    /// Returns provider positions in automatic-selection order.
    ///
    /// # Returns
    ///
    /// A slice of valid registry-owned entry positions.
    #[inline(always)]
    pub(crate) fn automatic_indices(&self) -> &[usize] {
        &self.inner.automatic_indices
    }

    /// Resolves a normalized selector through the selector index.
    ///
    /// # Arguments
    ///
    /// * `selector` - Valid normalized selector used as an index key.
    ///
    /// # Returns
    ///
    /// `Some` for a registered selector, or `None` otherwise.
    #[inline(always)]
    fn resolve_selector(
        &self,
        selector: &ProviderSelector,
    ) -> Option<ResolvedProvider<'_, S>> {
        self.index_for(selector)
            .map(|index| self.resolved_at(index))
    }
}

impl<S> Clone for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Clones the registry by incrementing its shared storage count.
    ///
    /// # Returns
    ///
    /// Another handle to the same immutable registry storage.
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
    /// Creates an empty registry through its default builder.
    ///
    /// # Returns
    ///
    /// An immutable registry with no providers.
    #[inline(always)]
    fn default() -> Self {
        Self::builder().build()
    }
}

impl<S> fmt::Debug for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Formats the registry using descriptors in registration order.
    ///
    /// # Arguments
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects any debug output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProviderRegistry { descriptors: ")?;
        formatter
            .debug_list()
            .entries(self.descriptors())
            .finish()?;
        formatter.write_str(" }")
    }
}
