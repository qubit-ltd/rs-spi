// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Construction of asynchronous registries from discovered provider factories.

use std::sync::Arc;

use crate::AsyncProviderDefinition;
use crate::AsyncProviderRegistry;
use crate::AsyncServiceSpec;
use crate::error::ProviderInventoryBuildError;
use crate::inventory::AsyncProviderInventoryEntry;

/// Builds an unsealed registry from discovered asynchronous provider entries.
///
/// Entries are ordered by declaration source before their factories run. A
/// repeated source is rejected before any factory or transform runs; otherwise
/// registration order is independent of linker inventory traversal order. This
/// function only registers provider definitions; it never invokes or polls
/// asynchronous service creation.
///
/// # Type Parameters
///
/// * `S` - Asynchronous service family implemented by the submitted providers.
///
/// # Parameters
///
/// * `entries` - Discovered entries collected for one concrete service family.
///
/// # Returns
///
/// An unsealed registry containing every successfully registered provider.
///
/// # Errors
///
/// Returns [`ProviderInventoryBuildError`] when sources are ambiguous or a
/// provider violates registry mutation rules. Source ambiguity is checked
/// before any factory runs and takes precedence over registration errors. The
/// partially built registry remains local and is not exposed to the caller.
///
/// # Panics
///
/// Propagates a panic raised by a submitted factory or by provider descriptor
/// evaluation during registration.
#[doc(hidden)]
#[inline]
pub fn build_async_registry<'a, S>(
    entries: impl IntoIterator<Item = &'a AsyncProviderInventoryEntry<S>>,
) -> Result<AsyncProviderRegistry<S>, ProviderInventoryBuildError>
where
    S: AsyncServiceSpec + 'a,
    S::Config: Sync,
{
    build_async_registry_with(entries, |provider| provider)
}

/// Builds an unsealed asynchronous registry after transforming each provider.
///
/// Entries are ordered by declaration source before factories and transforms
/// run. Duplicate sources are rejected before either callback runs. This lets
/// service-family registries install adapters while retaining deterministic
/// registration and source-rich errors. This function only registers
/// definitions; it never creates asynchronous services.
///
/// # Type Parameters
///
/// * `S` - Asynchronous service family implemented by the submitted providers.
///
/// # Parameters
///
/// * `entries` - Discovered entries collected for one concrete service family.
/// * `transform` - Adapter applied to each created provider before
///   registration.
///
/// # Returns
///
/// An unsealed registry containing every transformed provider.
///
/// # Errors
///
/// Returns [`ProviderInventoryBuildError`] when sources are ambiguous or a
/// transformed provider violates registry mutation rules. Source ambiguity is
/// checked first and takes precedence over registration errors.
///
/// # Panics
///
/// Propagates panics raised by provider factories, the transform, or provider
/// descriptor evaluation during registration.
#[doc(hidden)]
pub fn build_async_registry_with<'a, S>(
    entries: impl IntoIterator<Item = &'a AsyncProviderInventoryEntry<S>>,
    mut transform: impl FnMut(Arc<dyn AsyncProviderDefinition<S>>) -> Arc<dyn AsyncProviderDefinition<S>>,
) -> Result<AsyncProviderRegistry<S>, ProviderInventoryBuildError>
where
    S: AsyncServiceSpec + 'a,
    S::Config: Sync,
{
    let mut entries: Vec<_> = entries.into_iter().collect();
    entries.sort_by_key(|entry| entry.source());
    if let Some(pair) = entries.windows(2).find(|pair| pair[0].source() == pair[1].source()) {
        return Err(ProviderInventoryBuildError::ambiguous_source(pair[0].source()));
    }

    let registry = AsyncProviderRegistry::default();
    for entry in entries {
        let provider = transform(entry.create_provider());
        registry
            .register_shared(provider)
            .map_err(|error| ProviderInventoryBuildError::registration(entry.source(), error))?;
    }
    Ok(registry)
}
