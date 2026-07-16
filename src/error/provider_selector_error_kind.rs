// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stable classifications for provider selector failures.

/// Classification of a provider selector parsing failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectorErrorKind {
    /// Trimming produced an empty selector.
    Empty,
    /// The normalized selector violated selector syntax.
    Invalid,
}
