// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while validating canonical provider IDs.

use thiserror::Error;

/// Error returned when a canonical provider ID cannot be constructed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ProviderIdError {
    /// The supplied provider ID was empty.
    #[non_exhaustive]
    #[error("provider ID must not be empty")]
    Empty {
        /// Verbatim empty input retained for diagnostics.
        input: Box<str>,
    },
    /// The supplied provider ID violated canonical syntax.
    #[non_exhaustive]
    #[error("provider ID is not canonical: {input}")]
    NonCanonical {
        /// Verbatim noncanonical input retained for diagnostics.
        input: Box<str>,
    },
}

impl ProviderIdError {
    /// Creates an error for an empty canonical provider ID.
    ///
    /// # Arguments
    ///
    /// * `input` - Verbatim empty input retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An empty provider ID error.
    #[inline]
    #[must_use]
    pub(crate) fn empty(input: &str) -> Self {
        Self::Empty {
            input: input.into(),
        }
    }

    /// Creates an error for a noncanonical provider ID.
    ///
    /// # Arguments
    ///
    /// * `input` - Verbatim noncanonical input retained for diagnostics.
    ///
    /// # Returns
    ///
    /// A noncanonical provider ID error.
    #[inline]
    #[must_use]
    pub(crate) fn noncanonical(input: &str) -> Self {
        Self::NonCanonical {
            input: input.into(),
        }
    }

    /// Returns the verbatim provider ID input.
    ///
    /// # Returns
    ///
    /// The input rejected by canonical ID validation.
    #[inline(always)]
    #[must_use]
    pub fn input(&self) -> &str {
        match self {
            Self::Empty { input } | Self::NonCanonical { input } => input,
        }
    }
}
