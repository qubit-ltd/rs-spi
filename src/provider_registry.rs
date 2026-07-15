// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable provider catalog and typed provider lookup.

use std::{collections::HashMap, fmt, sync::Arc};

use crate::{
    ProviderDescriptor, ProviderId, ProviderRegistryBuilder, ProviderSelector, ResolutionError,
    ServiceProvider, ServiceSpec,
};

/// Internal pairing of one descriptor and the factory it represents.
///
/// Registry construction creates entries once; lookup results borrow them for
/// the lifetime of the immutable registry.
pub(crate) struct RegistryEntry<S>
where
    S: ServiceSpec,
{
    /// Metadata used to identify and order this provider.
    pub(crate) descriptor: ProviderDescriptor,
    /// Shared factory used to create this provider's service.
    pub(crate) provider: Arc<dyn ServiceProvider<S>>,
}

/// Immutable lookup indexes and entries shared by registry clones.
pub(crate) struct RegistryInner<S>
where
    S: ServiceSpec,
{
    /// Registrations retained in their original registration order.
    pub(crate) entries: Box<[RegistryEntry<S>]>,
    /// Mapping from canonical IDs and aliases to positions in `entries`.
    pub(crate) selector_indices: HashMap<ProviderSelector, usize>,
    /// Positions in the deterministic automatic-selection order.
    pub(crate) automatic_indices: Box<[usize]>,
}

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
    /// Returns a mutable [`ProviderRegistryBuilder`] used during startup to
    /// register providers before producing this immutable registry.
    #[must_use]
    pub fn builder() -> ProviderRegistryBuilder<S> {
        ProviderRegistryBuilder::new()
    }

    /// Resolves a canonical provider ID or alias.
    ///
    /// `selector` is parsed using selector normalization. Returns the matching
    /// provider together with its descriptor on success.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when `selector` is invalid or does not name
    /// a registered provider.
    pub fn resolve(
        &self,
        selector: impl AsRef<str>,
    ) -> Result<ResolvedProvider<'_, S>, ResolutionError> {
        let selector = ProviderSelector::parse(selector)
            .map_err(|_| ResolutionError::unknown_provider("<invalid>"))?;
        self.resolve_selector(&selector)
            .ok_or_else(|| ResolutionError::unknown_provider(selector.as_str()))
    }

    /// Finds a provider by canonical ID or alias without constructing an error.
    ///
    /// `selector` is parsed using selector normalization. Returns `Some` for a
    /// matching provider and `None` for invalid or unknown input.
    #[must_use]
    pub fn find(&self, selector: impl AsRef<str>) -> Option<ResolvedProvider<'_, S>> {
        ProviderSelector::parse(selector)
            .ok()
            .and_then(|selector| self.resolve_selector(&selector))
    }

    /// Iterates over descriptors in registration order.
    ///
    /// The returned exact-size iterator borrows this registry and yields each
    /// provider's immutable metadata exactly once.
    pub fn descriptors(&self) -> impl ExactSizeIterator<Item = &ProviderDescriptor> {
        self.inner.entries.iter().map(|entry| &entry.descriptor)
    }

    /// Iterates over canonical provider IDs in registration order.
    ///
    /// The returned exact-size iterator borrows this registry and yields each
    /// registered canonical ID exactly once.
    pub fn provider_ids(&self) -> impl ExactSizeIterator<Item = &ProviderId> {
        self.inner.entries.iter().map(|entry| entry.descriptor.id())
    }

    /// Returns whether this registry contains no registered providers.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.entries.is_empty()
    }

    /// Creates a registry from prepared immutable internal storage.
    ///
    /// `inner` must contain mutually consistent entries and indexes. Returns a
    /// registry sharing that storage; callers are responsible for the invariant.
    #[inline]
    pub(crate) fn from_inner(inner: Arc<RegistryInner<S>>) -> Self {
        Self { inner }
    }

    /// Looks up the entry position for a normalized selector.
    ///
    /// Returns `Some` with the internal entry position when found, and `None`
    /// otherwise.
    #[inline]
    pub(crate) fn index_for(&self, selector: &ProviderSelector) -> Option<usize> {
        self.inner.selector_indices.get(selector).copied()
    }

    /// Borrows the resolved provider at a valid internal entry position.
    ///
    /// `index` must refer to an existing entry. Returns its lookup wrapper and
    /// panics if the caller violates that internal invariant.
    #[inline]
    pub(crate) fn resolved_at(&self, index: usize) -> ResolvedProvider<'_, S> {
        ResolvedProvider {
            entry: &self.inner.entries[index],
        }
    }

    /// Returns provider entry positions in automatic-selection order.
    ///
    /// Each position is valid for this registry's internal entry array.
    #[inline]
    pub(crate) fn automatic_indices(&self) -> &[usize] {
        &self.inner.automatic_indices
    }

    /// Resolves a normalized selector by forwarding through the selector index.
    ///
    /// Returns `Some` for a registered selector and `None` otherwise.
    #[inline(always)]
    fn resolve_selector(&self, selector: &ProviderSelector) -> Option<ResolvedProvider<'_, S>> {
        self.index_for(selector)
            .map(|index| self.resolved_at(index))
    }
}

impl<S> Clone for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Clones the registry by incrementing its shared immutable-storage count.
    ///
    /// Returns another registry handle; entries and indexes are not copied.
    #[inline]
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
    /// Creates an empty registry by forwarding to the builder's default build.
    #[inline(always)]
    fn default() -> Self {
        Self::builder().build()
    }
}

impl<S> fmt::Debug for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Formats the registry using its descriptors in registration order.
    ///
    /// Returns a formatting error if `formatter` cannot accept the debug data.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderRegistry")
            .field("descriptors", &self.descriptors().collect::<Vec<_>>())
            .finish()
    }
}

/// Borrowed provider lookup result retaining descriptor metadata.
///
/// This value is returned by registry lookup and exposes the selected
/// provider's metadata and typed creation operation without exposing internals.
pub struct ResolvedProvider<'a, S>
where
    S: ServiceSpec,
{
    /// Internal entry borrowed from the immutable registry.
    entry: &'a RegistryEntry<S>,
}

impl<S> ResolvedProvider<'_, S>
where
    S: ServiceSpec,
{
    /// Returns the selected provider's registration metadata.
    #[must_use]
    #[inline]
    pub fn descriptor(&self) -> &ProviderDescriptor {
        &self.entry.descriptor
    }

    /// Creates a service through the resolved provider factory.
    ///
    /// `config` is the service-family configuration. Returns the provider's
    /// complete output on success.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ProviderError`] when the provider cannot create the
    /// service; its classification is preserved for resolver fallback logic.
    pub fn create(&self, config: &S::Config) -> Result<S::Output, crate::ProviderError> {
        self.entry.provider.create(config)
    }
}
