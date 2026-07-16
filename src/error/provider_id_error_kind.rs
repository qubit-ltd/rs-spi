// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stable classifications for canonical provider ID failures.

/// Classification of a canonical provider ID validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderIdErrorKind {
    /// The provider ID input was empty.
    Empty,
    /// The provider ID input violated canonical syntax.
    NonCanonical,
}
