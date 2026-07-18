// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private invariant-safe provider selection storage.

use crate::ProviderSelector;

/// Validated provider selection representation consumed by the resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProviderSelectionRepr {
    /// Providers are tried in deterministic automatic order.
    Auto,
    /// Exactly one normalized selector is used.
    Named(
        /// Normalized selector naming the only candidate.
        ProviderSelector,
    ),
    /// A nonempty ordered selector chain is used.
    Chain(
        /// Normalized selectors retained in caller-supplied order.
        Box<[ProviderSelector]>,
    ),
}
