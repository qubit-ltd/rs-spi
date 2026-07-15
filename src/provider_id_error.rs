// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while validating canonical provider IDs.

use thiserror::Error;

use crate::internal::ProviderIdErrorRepr;
use crate::ProviderIdErrorKind;

/// Error returned when a canonical provider ID cannot be constructed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderIdError(
    /// Variant-specific provider ID validation failure.
    ProviderIdErrorRepr,
);

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
        Self(ProviderIdErrorRepr::Empty {
            input: input.into(),
        })
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
        Self(ProviderIdErrorRepr::NonCanonical {
            input: input.into(),
        })
    }

    /// Returns the provider ID validation rule that failed.
    ///
    /// # Returns
    ///
    /// The empty or noncanonical classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderIdErrorKind {
        match self.0 {
            ProviderIdErrorRepr::Empty { .. } => ProviderIdErrorKind::Empty,
            ProviderIdErrorRepr::NonCanonical { .. } => ProviderIdErrorKind::NonCanonical,
        }
    }

    /// Returns the verbatim provider ID input retained by this error.
    ///
    /// # Returns
    ///
    /// The original invalid input.
    #[inline(always)]
    #[must_use]
    pub fn input(&self) -> Option<&str> {
        match &self.0 {
            ProviderIdErrorRepr::Empty { input }
            | ProviderIdErrorRepr::NonCanonical { input } => Some(input),
        }
    }
}
