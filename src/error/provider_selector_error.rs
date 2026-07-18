// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while parsing provider selectors.

use thiserror::Error;

/// Error returned when provider selector input cannot be parsed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectorError {
    /// Trimming the input produced an empty selector.
    #[non_exhaustive]
    #[error("provider selector must not be empty")]
    Empty {
        /// Verbatim selector input.
        input: Box<str>,
    },
    /// The normalized selector violated selector syntax.
    #[non_exhaustive]
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
    /// # Parameters
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
    /// # Parameters
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
}
