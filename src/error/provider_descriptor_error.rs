// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider descriptors.

use std::error::Error;
use std::fmt;

use super::ProviderSelectorError;

/// Error returned when provider descriptor aliases are invalid or ambiguous.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderDescriptorError {
    /// An alias cannot be parsed as a selector.
    #[non_exhaustive]
    InvalidAlias {
        /// Zero-based position of the invalid alias.
        alias_index: usize,
        /// Selector parsing failure owning the verbatim alias.
        source: ProviderSelectorError,
    },
    /// Two aliases normalize to the same selector.
    #[non_exhaustive]
    DuplicateAlias {
        /// Normalized duplicate alias.
        alias: Box<str>,
    },
    /// An alias normalizes to the canonical provider ID.
    #[non_exhaustive]
    AliasMatchesId {
        /// Normalized alias matching the canonical ID.
        alias: Box<str>,
    },
}

impl ProviderDescriptorError {
    /// Creates an error for an alias that cannot be parsed.
    ///
    /// # Parameters
    ///
    /// * `alias_index` - Zero-based alias position.
    /// * `source` - Selector parsing error that rejected the alias.
    ///
    /// # Returns
    ///
    /// An invalid-alias descriptor error retaining the parse source.
    #[inline]
    #[must_use]
    pub(crate) fn invalid_alias(
        alias_index: usize,
        source: ProviderSelectorError,
    ) -> Self {
        Self::InvalidAlias {
            alias_index,
            source,
        }
    }

    /// Creates an error for aliases that normalize to the same selector.
    ///
    /// # Parameters
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
    /// # Parameters
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

    /// Returns the alias retained by this error.
    ///
    /// # Returns
    ///
    /// The verbatim invalid alias or normalized conflicting alias.
    #[inline(always)]
    #[must_use]
    pub fn alias(&self) -> &str {
        match self {
            Self::InvalidAlias { source, .. } => source.input(),
            Self::DuplicateAlias { alias } | Self::AliasMatchesId { alias } => {
                alias
            }
        }
    }
}

impl fmt::Display for ProviderDescriptorError {
    /// Formats the descriptor construction failure.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects diagnostic output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAlias {
                alias_index,
                source,
            } => write!(
                formatter,
                "invalid provider alias at index {alias_index}: {:?}",
                source.input(),
            ),
            Self::DuplicateAlias { alias } => {
                write!(formatter, "duplicate provider alias: {alias}")
            }
            Self::AliasMatchesId { alias } => {
                write!(
                    formatter,
                    "provider alias matches canonical ID: {alias}"
                )
            }
        }
    }
}

impl Error for ProviderDescriptorError {
    /// Returns the selector parsing failure when one is available.
    ///
    /// # Returns
    ///
    /// The selector parsing source for an invalid alias, or `None` for alias
    /// conflicts.
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidAlias { source, .. } => Some(source),
            Self::DuplicateAlias { .. } | Self::AliasMatchesId { .. } => None,
        }
    }
}
