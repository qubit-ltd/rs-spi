// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for provider descriptor construction failures.

use thiserror::Error;

use crate::error::ProviderSelectorError;

/// Variant-specific provider descriptor construction failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum ProviderDescriptorErrorRepr {
    /// An alias cannot be parsed as a selector.
    #[error("invalid provider alias at index {alias_index}: {alias:?}")]
    InvalidAlias {
        /// Zero-based position of the invalid alias.
        alias_index: usize,
        /// Verbatim invalid alias.
        alias: Box<str>,
        /// Selector parsing failure.
        #[source]
        source: ProviderSelectorError,
    },
    /// Two aliases normalize to the same selector.
    #[error("duplicate provider alias: {alias}")]
    DuplicateAlias {
        /// Normalized duplicate alias.
        alias: Box<str>,
    },
    /// An alias normalizes to the canonical provider ID.
    #[error("provider alias matches canonical ID: {alias}")]
    AliasMatchesId {
        /// Normalized alias matching the canonical ID.
        alias: Box<str>,
    },
}
