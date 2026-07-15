// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classifications for aggregate provider resolution failures.

/// Classification of a failed provider-selection resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// A raw selector does not satisfy selector syntax.
    InvalidSelector,
    /// A raw chained selection contains no selectors.
    EmptySelection,
    /// A named selector does not resolve to a registered provider.
    UnknownProvider,
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// At least one candidate was considered but no service was produced.
    NoProviderSucceeded,
}
