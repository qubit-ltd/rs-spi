// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private immutable storage shared by provider registry clones.

use std::collections::HashMap;

use qubit_collections::map::OrderedIndexMap;

use crate::ProviderId;
use crate::ProviderSelection;
use crate::ProviderSelector;
use crate::registry::internal::RegistryEntry;

/// Mutable provider catalog protected by the registry's synchronization lock.
///
/// # Type Parameters
///
/// * `P` - Possibly unsized provider definition stored by the catalog.
pub(crate) struct RegistryInner<P: ?Sized> {
    /// Entries indexed by canonical ID and automatic-selection order.
    pub(crate) entries: OrderedIndexMap<
        ProviderId,
        (std::cmp::Reverse<i32>, ProviderId),
        RegistryEntry<P>,
    >,
    /// Mapping from canonical IDs and aliases to canonical provider IDs.
    pub(crate) selector_ids: HashMap<ProviderSelector, ProviderId>,
    /// Canonical IDs retained in successful registration order.
    pub(crate) registration_ids: Vec<ProviderId>,
    /// Selection used by callers that do not supply an explicit preference.
    pub(crate) default_selection: ProviderSelection,
}

impl<P: ?Sized> Default for RegistryInner<P> {
    /// Creates empty mutable registry state.
    ///
    /// # Returns
    ///
    /// Empty indexes and automatic default provider selection.
    #[inline]
    fn default() -> Self {
        Self {
            entries: OrderedIndexMap::new(),
            selector_ids: HashMap::new(),
            registration_ids: Vec::new(),
            default_selection: ProviderSelection::auto(),
        }
    }
}
