// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classifications for provider descriptor construction failures.

/// Classification of a provider descriptor construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderDescriptorErrorKind {
    /// An alias cannot be parsed as a provider selector.
    InvalidAlias,
    /// Two aliases normalize to the same selector.
    DuplicateAlias,
    /// An alias normalizes to the canonical provider ID.
    AliasMatchesId,
}
