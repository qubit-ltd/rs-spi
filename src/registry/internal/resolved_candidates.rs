// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Owned candidate snapshots returned by the shared provider catalog.

use crate::FallbackPolicy;

use super::RegistryEntry;

/// Resolved provider entries and the fallback policy applied to them.
pub(crate) struct ResolvedCandidates<P: ?Sized> {
    /// Nonempty provider entries in deterministic attempt order.
    pub(crate) entries: Box<[RegistryEntry<P>]>,
    /// Policy applied after each provider creation failure.
    pub(crate) fallback_policy: FallbackPolicy,
}
