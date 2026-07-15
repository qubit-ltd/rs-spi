// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while parsing provider selectors.

use thiserror::Error;

use crate::ProviderSelectorErrorKind;
use crate::internal::ProviderSelectorErrorRepr;

/// Error returned when provider selector input cannot be parsed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderSelectorError(
    /// Variant-specific provider selector parsing failure.
    ProviderSelectorErrorRepr,
);

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
        Self(ProviderSelectorErrorRepr::Empty {
            input: input.into(),
        })
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
        Self(ProviderSelectorErrorRepr::Invalid {
            input: input.into(),
            normalized: normalized.into(),
        })
    }

    /// Returns the selector parsing rule that failed.
    ///
    /// # Returns
    ///
    /// The empty or invalid selector classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectorErrorKind {
        match self.0 {
            ProviderSelectorErrorRepr::Empty { .. } => {
                ProviderSelectorErrorKind::Empty
            }
            ProviderSelectorErrorRepr::Invalid { .. } => {
                ProviderSelectorErrorKind::Invalid
            }
        }
    }

    /// Returns the verbatim selector input.
    ///
    /// # Returns
    ///
    /// The original unnormalized selector text.
    #[inline(always)]
    #[must_use]
    pub fn input(&self) -> &str {
        match &self.0 {
            ProviderSelectorErrorRepr::Empty { input }
            | ProviderSelectorErrorRepr::Invalid { input, .. } => input,
        }
    }

    /// Returns the normalized invalid selector.
    ///
    /// # Returns
    ///
    /// `Some` for invalid normalized syntax, or `None` when trimming produced
    /// an empty selector.
    #[inline(always)]
    #[must_use]
    pub fn normalized(&self) -> Option<&str> {
        match &self.0 {
            ProviderSelectorErrorRepr::Empty { .. } => None,
            ProviderSelectorErrorRepr::Invalid { normalized, .. } => {
                Some(normalized)
            }
        }
    }
}
