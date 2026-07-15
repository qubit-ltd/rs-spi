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

pub(crate) struct RegistryEntry<S>
where
    S: ServiceSpec,
{
    pub(crate) descriptor: ProviderDescriptor,
    pub(crate) provider: Arc<dyn ServiceProvider<S>>,
}

pub(crate) struct RegistryInner<S>
where
    S: ServiceSpec,
{
    pub(crate) entries: Box<[RegistryEntry<S>]>,
    pub(crate) selector_indices: HashMap<ProviderSelector, usize>,
    pub(crate) automatic_indices: Box<[usize]>,
}

/// Immutable catalog of explicitly registered providers.
pub struct ProviderRegistry<S>
where
    S: ServiceSpec,
{
    inner: Arc<RegistryInner<S>>,
}

impl<S> ProviderRegistry<S>
where
    S: ServiceSpec,
{
    /// Creates an empty builder for this service specification.
    #[must_use]
    pub fn builder() -> ProviderRegistryBuilder<S> {
        ProviderRegistryBuilder::new()
    }

    /// Resolves a canonical ID or alias.
    ///
    /// # Errors
    ///
    /// Returns ResolutionError when the selector is invalid or unknown.
    pub fn resolve(
        &self,
        selector: impl AsRef<str>,
    ) -> Result<ResolvedProvider<'_, S>, ResolutionError> {
        let selector = ProviderSelector::parse(selector)
            .map_err(|_| ResolutionError::unknown_provider("<invalid>"))?;
        self.resolve_selector(&selector)
            .ok_or_else(|| ResolutionError::unknown_provider(selector.as_str()))
    }

    /// Finds a provider by canonical ID or alias.
    #[must_use]
    pub fn find(&self, selector: impl AsRef<str>) -> Option<ResolvedProvider<'_, S>> {
        ProviderSelector::parse(selector)
            .ok()
            .and_then(|selector| self.resolve_selector(&selector))
    }

    /// Iterates over descriptors in registration order.
    pub fn descriptors(&self) -> impl ExactSizeIterator<Item = &ProviderDescriptor> {
        self.inner.entries.iter().map(|entry| &entry.descriptor)
    }

    /// Iterates over canonical IDs in registration order.
    pub fn provider_ids(&self) -> impl ExactSizeIterator<Item = &ProviderId> {
        self.inner.entries.iter().map(|entry| entry.descriptor.id())
    }

    /// Returns whether this registry has no providers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.entries.is_empty()
    }

    pub(crate) fn from_inner(inner: Arc<RegistryInner<S>>) -> Self {
        Self { inner }
    }

    pub(crate) fn index_for(&self, selector: &ProviderSelector) -> Option<usize> {
        self.inner.selector_indices.get(selector).copied()
    }

    pub(crate) fn resolved_at(&self, index: usize) -> ResolvedProvider<'_, S> {
        ResolvedProvider {
            entry: &self.inner.entries[index],
        }
    }

    pub(crate) fn automatic_indices(&self) -> &[usize] {
        &self.inner.automatic_indices
    }

    fn resolve_selector(&self, selector: &ProviderSelector) -> Option<ResolvedProvider<'_, S>> {
        self.index_for(selector)
            .map(|index| self.resolved_at(index))
    }
}

impl<S> Clone for ProviderRegistry<S>
where
    S: ServiceSpec,
{
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
    fn default() -> Self {
        Self::builder().build()
    }
}

impl<S> fmt::Debug for ProviderRegistry<S>
where
    S: ServiceSpec,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderRegistry")
            .field("descriptors", &self.descriptors().collect::<Vec<_>>())
            .finish()
    }
}

/// Opaque provider lookup result retaining descriptor metadata.
pub struct ResolvedProvider<'a, S>
where
    S: ServiceSpec,
{
    entry: &'a RegistryEntry<S>,
}

impl<S> ResolvedProvider<'_, S>
where
    S: ServiceSpec,
{
    /// Gets provider registration metadata.
    #[must_use]
    pub fn descriptor(&self) -> &ProviderDescriptor {
        &self.entry.descriptor
    }

    /// Creates a service through the resolved provider.
    pub fn create(&self, config: &S::Config) -> Result<S::Output, crate::ProviderError> {
        self.entry.provider.create(config)
    }
}
