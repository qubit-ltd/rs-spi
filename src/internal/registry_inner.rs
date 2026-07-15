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
use crate::{ProviderSelector, ServiceSpec};

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
