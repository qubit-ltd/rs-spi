// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stable classifications for provider descriptor failures.

/// Classification of a provider descriptor construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderDescriptorErrorKind {
    /// An alias could not be parsed.
    InvalidAlias,
    /// Two aliases normalized to the same selector.
    DuplicateAlias,
    /// An alias normalized to the canonical provider ID.
    AliasMatchesId,
}
