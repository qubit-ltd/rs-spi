// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
// =============================================================================
//! Construction of synchronous registries from discovered provider factories.

use crate::ProviderRegistry;
use crate::SyncServiceSpec;
use crate::error::ProviderInventoryBuildError;
use crate::inventory::SyncProviderInventoryEntry;

/// Builds an unsealed registry from discovered synchronous provider entries.
///
/// Entries are stably ordered by their declaration source before their
/// factories run, making registration order independent of linker inventory
/// traversal order.
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
/// Returns [`ProviderInventoryBuildError`] with the failing submission source
/// when a provider violates registry mutation rules. The partially built
/// registry remains local and is not exposed to the caller.
///
/// # Panics
///
/// Propagates a panic raised by a submitted factory or by provider descriptor
/// evaluation during registration.
#[doc(hidden)]
pub fn build_sync_registry<'a, S>(
    entries: impl IntoIterator<Item = &'a SyncProviderInventoryEntry<S>>,
) -> Result<ProviderRegistry<S>, ProviderInventoryBuildError>
where
    S: SyncServiceSpec + 'a,
{
    let mut entries: Vec<_> = entries.into_iter().collect();
    entries.sort_by_key(|entry| entry.source());

    let registry = ProviderRegistry::default();
    for entry in entries {
        let provider = entry.create_provider();
        registry
            .register_shared(provider)
            .map_err(|error| ProviderInventoryBuildError::registration(entry.source(), error))?;
    }
    Ok(registry)
}
