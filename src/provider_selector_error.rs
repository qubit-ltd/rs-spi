// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while parsing provider selectors.

use thiserror::Error;

/// Classification of a provider selector parsing failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectorErrorKind {
    /// Trimming the input produced an empty selector.
    Empty,
    /// The normalized selector violates selector syntax.
    Invalid,
}

/// Error returned when provider selector input cannot be parsed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderSelectorError(ProviderSelectorErrorRepr);

/// Private representation of provider selector parsing failures.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
enum ProviderSelectorErrorRepr {
    /// Trimming the input produced an empty selector.
    #[error("provider selector must not be empty")]
    Empty {
        /// Verbatim selector input.
        input: Box<str>,
    },
    /// The normalized selector violated selector syntax.
    #[error("invalid provider selector {input:?} (normalized as {normalized:?})")]
    Invalid {
        /// Verbatim selector input.
        input: Box<str>,
        /// Trimmed and ASCII-lowercased selector input.
        normalized: Box<str>,
    },
}

impl ProviderSelectorError {
    /// Creates an error for selector input that becomes empty after trimming.
    #[must_use]
    pub(crate) fn empty(input: impl AsRef<str>) -> Self {
        Self(ProviderSelectorErrorRepr::Empty {
            input: input.as_ref().into(),
        })
    }

    /// Creates an error for invalid normalized selector input.
    #[must_use]
    pub(crate) fn invalid(input: impl AsRef<str>, normalized: impl AsRef<str>) -> Self {
        Self(ProviderSelectorErrorRepr::Invalid {
            input: input.as_ref().into(),
            normalized: normalized.as_ref().into(),
        })
    }

    /// Returns the selector parsing rule that failed.
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectorErrorKind {
        match self.0 {
            ProviderSelectorErrorRepr::Empty { .. } => ProviderSelectorErrorKind::Empty,
            ProviderSelectorErrorRepr::Invalid { .. } => ProviderSelectorErrorKind::Invalid,
        }
    }

    /// Returns the verbatim selector input.
    #[must_use]
    pub fn input(&self) -> &str {
        match &self.0 {
            ProviderSelectorErrorRepr::Empty { input }
            | ProviderSelectorErrorRepr::Invalid { input, .. } => input,
        }
    }

    /// Returns the normalized invalid selector, when normalization produced one.
    #[must_use]
    pub fn normalized(&self) -> Option<&str> {
        match &self.0 {
            ProviderSelectorErrorRepr::Empty { .. } => None,
            ProviderSelectorErrorRepr::Invalid { normalized, .. } => Some(normalized),
        }
    }
}
