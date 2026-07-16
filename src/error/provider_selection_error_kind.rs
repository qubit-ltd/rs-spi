// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stable classifications for provider selection failures.

/// Classification of a provider selection construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionErrorKind {
    /// A selector input was invalid.
    InvalidSelector,
    /// A chained selection contained no selectors.
    EmptyChain,
}
