// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while parsing provider selectors.

use thiserror::Error;

use super::ProviderSelectorErrorKind;

/// Error returned when provider selector input cannot be parsed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectorError {
    /// Trimming the input produced an empty selector.
    #[error("provider selector must not be empty")]
    Empty {
        /// Verbatim selector input.
        input: Box<str>,
    },
    /// The normalized selector violated selector syntax.
    #[error(
        "invalid provider selector {input:?} (normalized as {normalized:?})"
    )]
    Invalid {
        /// Verbatim selector input.
        input: Box<str>,
        /// Trimmed and ASCII-lowercased selector input.
        normalized: Box<str>,
    },
}

impl ProviderSelectorError {
    /// Creates an error for selector input that becomes empty after trimming.
    ///
    /// # Arguments
    ///
    /// * `input` - Verbatim selector input.
    ///
    /// # Returns
    ///
    /// An empty selector error.
    #[inline]
    #[must_use]
    pub(crate) fn empty(input: &str) -> Self {
        Self::Empty {
            input: input.into(),
        }
    }

    /// Creates an error for invalid normalized selector input.
    ///
    /// # Arguments
    ///
    /// * `input` - Verbatim selector input.
    /// * `normalized` - Trimmed and ASCII-lowercased invalid value.
    ///
    /// # Returns
    ///
    /// An invalid selector error retaining both representations.
    #[inline]
    #[must_use]
    pub(crate) fn invalid(input: &str, normalized: &str) -> Self {
        Self::Invalid {
            input: input.into(),
            normalized: normalized.into(),
        }
    }

    /// Returns this selector error's stable classification.
    ///
    /// # Returns
    ///
    /// The empty or invalid classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectorErrorKind {
        match self {
            Self::Empty { .. } => ProviderSelectorErrorKind::Empty,
            Self::Invalid { .. } => ProviderSelectorErrorKind::Invalid,
        }
    }

    /// Returns the verbatim selector input.
    ///
    /// # Returns
    ///
    /// The input supplied to selector parsing.
    #[inline(always)]
    #[must_use]
    pub fn input(&self) -> &str {
        match self {
            Self::Empty { input } | Self::Invalid { input, .. } => input,
        }
    }

    /// Returns normalized invalid selector text.
    ///
    /// # Returns
    ///
    /// The invalid normalized value, or `None` when trimming produced an empty
    /// selector.
    #[inline(always)]
    #[must_use]
    pub fn normalized(&self) -> Option<&str> {
        match self {
            Self::Invalid { normalized, .. } => Some(normalized),
            Self::Empty { .. } => None,
        }
    }

    /// Reports whether trimming produced an empty selector.
    ///
    /// # Returns
    ///
    /// `true` only for [`Self::Empty`].
    #[inline(always)]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty { .. })
    }
}
