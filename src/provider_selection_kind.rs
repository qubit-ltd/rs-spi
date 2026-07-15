// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classifications for validated provider selections.

/// Classification of a validated provider selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionKind {
    /// Providers are tried in deterministic automatic order.
    Auto,
    /// Exactly one normalized selector is used.
    Named,
    /// Normalized selectors are tried in caller-supplied order.
    Chain,
}
