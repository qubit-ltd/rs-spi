// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider descriptors.

use thiserror::Error;

use super::ProviderSelectorError;

/// Error returned when provider descriptor aliases are invalid or ambiguous.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ProviderDescriptorError {
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

impl ProviderDescriptorError {
    /// Creates an error for an alias that cannot be parsed.
    ///
    /// # Arguments
    ///
    /// * `alias_index` - Zero-based alias position.
    /// * `alias` - Verbatim invalid alias.
    /// * `source` - Selector parsing error that rejected the alias.
    ///
    /// # Returns
    ///
    /// An invalid-alias descriptor error retaining the parse source.
    #[inline]
    #[must_use]
    pub(crate) fn invalid_alias(
        alias_index: usize,
        alias: &str,
        source: ProviderSelectorError,
    ) -> Self {
        Self::InvalidAlias {
            alias_index,
            alias: alias.into(),
            source,
        }
    }

    /// Creates an error for aliases that normalize to the same selector.
    ///
    /// # Arguments
    ///
    /// * `alias` - Normalized duplicate alias.
    ///
    /// # Returns
    ///
    /// A duplicate-alias descriptor error.
    #[inline]
    #[must_use]
    pub(crate) fn duplicate_alias(alias: &str) -> Self {
        Self::DuplicateAlias {
            alias: alias.into(),
        }
    }

    /// Creates an error for an alias matching the canonical provider ID.
    ///
    /// # Arguments
    ///
    /// * `alias` - Normalized alias matching the canonical ID.
    ///
    /// # Returns
    ///
    /// An alias-matches-ID descriptor error.
    #[inline]
    #[must_use]
    pub(crate) fn alias_matches_id(alias: &str) -> Self {
        Self::AliasMatchesId {
            alias: alias.into(),
        }
    }
}
