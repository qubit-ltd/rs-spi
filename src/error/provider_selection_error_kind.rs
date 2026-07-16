// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Error classifications for provider selection construction failures.

/// Classification of a provider selection construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionErrorKind {
    /// One selector input cannot be parsed.
    InvalidSelector,
    /// A chained selection contains no selector inputs.
    EmptyChain,
}
