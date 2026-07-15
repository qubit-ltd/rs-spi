// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while validating canonical provider IDs.

use thiserror::Error;

/// Classification of a canonical provider ID validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderIdErrorKind {
    /// The supplied provider ID is empty.
    Empty,
    /// The supplied provider ID is not in canonical form.
    NonCanonical,
}

/// Error returned when a canonical provider ID cannot be constructed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderIdError(ProviderIdErrorRepr);

/// Private representation of canonical provider ID validation failures.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
enum ProviderIdErrorRepr {
    /// The supplied provider ID was empty.
    #[error("provider ID must not be empty")]
    Empty {
        /// Verbatim empty input retained for diagnostics.
        input: Box<str>,
    },
    /// The supplied provider ID violated canonical syntax.
    #[error("provider ID is not canonical: {input}")]
    NonCanonical {
        /// Verbatim noncanonical input retained for diagnostics.
        input: Box<str>,
    },
}

impl ProviderIdError {
    /// Creates an error for an empty canonical provider ID.
    #[must_use]
    pub(crate) fn empty(input: impl AsRef<str>) -> Self {
        Self(ProviderIdErrorRepr::Empty {
            input: input.as_ref().into(),
        })
    }

    /// Creates an error for a noncanonical provider ID.
    #[must_use]
    pub(crate) fn noncanonical(input: impl AsRef<str>) -> Self {
        Self(ProviderIdErrorRepr::NonCanonical {
            input: input.as_ref().into(),
        })
    }

    /// Returns the provider ID validation rule that failed.
    #[must_use]
    pub const fn kind(&self) -> ProviderIdErrorKind {
        match self.0 {
            ProviderIdErrorRepr::Empty { .. } => ProviderIdErrorKind::Empty,
            ProviderIdErrorRepr::NonCanonical { .. } => ProviderIdErrorKind::NonCanonical,
        }
    }

    /// Returns the verbatim provider ID input retained by this error.
    #[must_use]
    pub fn input(&self) -> Option<&str> {
        match &self.0 {
            ProviderIdErrorRepr::Empty { input } | ProviderIdErrorRepr::NonCanonical { input } => {
                Some(input)
            }
        }
    }
}
