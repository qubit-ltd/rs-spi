// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider descriptors.

use thiserror::Error;

use crate::internal::ProviderDescriptorErrorRepr;
use crate::{ProviderDescriptorErrorKind, ProviderSelectorError};

/// Error returned when provider descriptor aliases are invalid or ambiguous.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderDescriptorError(
    /// Variant-specific descriptor construction failure.
    ProviderDescriptorErrorRepr,
);

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
        Self(ProviderDescriptorErrorRepr::InvalidAlias {
            alias_index,
            alias: alias.into(),
            source,
        })
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
        Self(ProviderDescriptorErrorRepr::DuplicateAlias {
            alias: alias.into(),
        })
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
        Self(ProviderDescriptorErrorRepr::AliasMatchesId {
            alias: alias.into(),
        })
    }

    /// Returns the descriptor construction rule that failed.
    ///
    /// # Returns
    ///
    /// The classification corresponding to the retained descriptor failure.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderDescriptorErrorKind {
        match self.0 {
            ProviderDescriptorErrorRepr::InvalidAlias { .. } => {
                ProviderDescriptorErrorKind::InvalidAlias
            }
            ProviderDescriptorErrorRepr::DuplicateAlias { .. } => {
                ProviderDescriptorErrorKind::DuplicateAlias
            }
            ProviderDescriptorErrorRepr::AliasMatchesId { .. } => {
                ProviderDescriptorErrorKind::AliasMatchesId
            }
        }
    }

    /// Returns the zero-based invalid alias position.
    ///
    /// # Returns
    ///
    /// `Some` for an invalid alias, or `None` for alias conflicts.
    #[inline(always)]
    #[must_use]
    pub const fn alias_index(&self) -> Option<usize> {
        match self.0 {
            ProviderDescriptorErrorRepr::InvalidAlias { alias_index, .. } => Some(alias_index),
            ProviderDescriptorErrorRepr::DuplicateAlias { .. }
            | ProviderDescriptorErrorRepr::AliasMatchesId { .. } => None,
        }
    }

    /// Returns the invalid or conflicting alias retained by this error.
    ///
    /// # Returns
    ///
    /// The alias associated with this descriptor failure.
    #[inline(always)]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        match &self.0 {
            ProviderDescriptorErrorRepr::InvalidAlias { alias, .. }
            | ProviderDescriptorErrorRepr::DuplicateAlias { alias }
            | ProviderDescriptorErrorRepr::AliasMatchesId { alias } => Some(alias),
        }
    }
}
