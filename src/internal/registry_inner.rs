// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private immutable storage shared by provider registry clones.

use std::collections::HashMap;

use crate::internal::RegistryEntry;
use crate::{
    ProviderSelection,
    ProviderSelector,
    ServiceSpec,
};

/// Mutable provider catalog protected by the registry's synchronization lock.
pub(crate) struct RegistryInner<S>
where
    S: ServiceSpec,
{
    /// Registrations retained in their original registration order.
    pub(crate) entries: Vec<RegistryEntry<S>>,
    /// Mapping from canonical IDs and aliases to positions in `entries`.
    pub(crate) selector_indices: HashMap<ProviderSelector, usize>,
    /// Positions in the deterministic automatic-selection order.
    pub(crate) automatic_indices: Vec<usize>,
    /// Selection used by callers that do not supply an explicit preference.
    pub(crate) default_selection: ProviderSelection,
}

impl<S> Default for RegistryInner<S>
where
    S: ServiceSpec,
{
    /// Creates empty mutable registry state.
    ///
    /// # Returns
    ///
    /// Empty indexes and automatic default provider selection.
    #[inline]
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            selector_indices: HashMap::new(),
            automatic_indices: Vec::new(),
            default_selection: ProviderSelection::auto(),
        }
    }
}
