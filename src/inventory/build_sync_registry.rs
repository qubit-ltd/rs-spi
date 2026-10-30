// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Construction of synchronous registries from discovered provider factories.

use std::sync::Arc;

use crate::ProviderDefinition;
use crate::ProviderRegistry;
use crate::SyncServiceSpec;
use crate::error::ProviderInventoryBuildError;
use crate::inventory::SyncProviderInventoryEntry;

/// Builds an unsealed registry from discovered synchronous provider entries.
///
/// Entries are ordered by declaration source before their factories run. A
/// repeated source is rejected before any factory or transform runs; otherwise
/// registration order is independent of linker inventory traversal order.
///
/// # Type Parameters
///
/// * `S` - Synchronous service family implemented by the submitted providers.
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
pub fn build_sync_registry<'a, S>(
    entries: impl IntoIterator<Item = &'a SyncProviderInventoryEntry<S>>,
) -> Result<ProviderRegistry<S>, ProviderInventoryBuildError>
where
    S: SyncServiceSpec + 'a,
{
    build_sync_registry_with(entries, |provider| provider)
}

/// Builds an unsealed synchronous registry after transforming each provider.
///
/// Entries are ordered by declaration source before factories and transforms
/// run. Duplicate sources are rejected before either callback runs. This lets
/// service-family registries install adapters while retaining deterministic
/// registration and error reporting behavior.
///
/// # Type Parameters
///
/// * `S` - Synchronous service family implemented by the submitted providers.
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
pub fn build_sync_registry_with<'a, S>(
    entries: impl IntoIterator<Item = &'a SyncProviderInventoryEntry<S>>,
    mut transform: impl FnMut(Arc<dyn ProviderDefinition<S>>) -> Arc<dyn ProviderDefinition<S>>,
) -> Result<ProviderRegistry<S>, ProviderInventoryBuildError>
where
    S: SyncServiceSpec + 'a,
{
    let mut entries: Vec<_> = entries.into_iter().collect();
    entries.sort_by_key(|entry| entry.source());
    if let Some(pair) = entries.windows(2).find(|pair| pair[0].source() == pair[1].source()) {
        return Err(ProviderInventoryBuildError::ambiguous_source(pair[0].source()));
    }

    let registry = ProviderRegistry::default();
    for entry in entries {
        let provider = transform(entry.create_provider());
        registry
            .register_shared(provider)
            .map_err(|error| ProviderInventoryBuildError::registration(entry.source(), error))?;
    }
    Ok(registry)
}
