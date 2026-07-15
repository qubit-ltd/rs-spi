// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider descriptors.

use thiserror::Error;

use crate::ProviderSelectorError;

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

/// Error returned when provider descriptor aliases are invalid or ambiguous.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderDescriptorError(ProviderDescriptorErrorRepr);

/// Private representation of provider descriptor construction failures.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
enum ProviderDescriptorErrorRepr {
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
    #[must_use]
    pub(crate) fn invalid_alias(
        alias_index: usize,
        alias: impl AsRef<str>,
        source: ProviderSelectorError,
    ) -> Self {
        Self(ProviderDescriptorErrorRepr::InvalidAlias {
            alias_index,
            alias: alias.as_ref().into(),
            source,
        })
    }

    /// Creates an error for aliases that normalize to the same selector.
    #[must_use]
    pub(crate) fn duplicate_alias(alias: impl AsRef<str>) -> Self {
        Self(ProviderDescriptorErrorRepr::DuplicateAlias {
            alias: alias.as_ref().into(),
        })
    }

    /// Creates an error for an alias that matches the canonical provider ID.
    #[must_use]
    pub(crate) fn alias_matches_id(alias: impl AsRef<str>) -> Self {
        Self(ProviderDescriptorErrorRepr::AliasMatchesId {
            alias: alias.as_ref().into(),
        })
    }

    /// Returns the descriptor construction rule that failed.
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

    /// Returns the zero-based invalid alias position, when applicable.
    #[must_use]
    pub const fn alias_index(&self) -> Option<usize> {
        match self.0 {
            ProviderDescriptorErrorRepr::InvalidAlias { alias_index, .. } => Some(alias_index),
            ProviderDescriptorErrorRepr::DuplicateAlias { .. }
            | ProviderDescriptorErrorRepr::AliasMatchesId { .. } => None,
        }
    }

    /// Returns the invalid or conflicting alias retained by this error.
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        match &self.0 {
            ProviderDescriptorErrorRepr::InvalidAlias { alias, .. }
            | ProviderDescriptorErrorRepr::DuplicateAlias { alias }
            | ProviderDescriptorErrorRepr::AliasMatchesId { alias } => Some(alias),
        }
    }
}
