// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Error classifications for canonical provider ID validation failures.

/// Classification of a canonical provider ID validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderIdErrorKind {
    /// The supplied provider ID is empty.
    Empty,
    /// The supplied provider ID is not in canonical form.
    NonCanonical,
}
