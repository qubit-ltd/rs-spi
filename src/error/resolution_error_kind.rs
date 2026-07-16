// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stable classifications for provider resolution failures.

/// Classification of a provider resolution failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// Raw selector input was invalid.
    InvalidSelector,
    /// A raw chained selection was empty.
    EmptySelection,
    /// A valid selector matched no provider.
    UnknownProvider,
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// Candidate traversal ended without a successful provider.
    NoProviderSucceeded,
}
