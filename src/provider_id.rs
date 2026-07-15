// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict canonical provider identifiers.

use std::{fmt, str::FromStr};

use crate::ProviderIdError;

/// Stable canonical identifier of a provider.
///
/// Use this type to assign a provider its unique registry identity. Unlike a
/// selector, an ID must already be canonical and is never normalized.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderId(
    /// Canonical ASCII identifier text.
    Box<str>,
);

impl ProviderId {
    /// Creates a canonical provider identifier from already canonical text.
    ///
    /// `value` becomes the stable provider ID when it satisfies the documented
    /// canonical-token grammar. Returns the validated identifier.
    ///
    /// The value must already be lowercase ASCII and may contain alphanumeric
    /// characters plus hyphen, underscore, period, and plus between
    /// alphanumeric endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderIdError`] when `value` is empty or noncanonical.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ProviderIdError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ProviderIdError::empty(value));
        }
        if !is_canonical_token(value) {
            return Err(ProviderIdError::noncanonical(value));
        }
        Ok(Self(value.into()))
    }

    /// Returns the canonical identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderId {
    /// Forwards to [`ProviderId::as_str`].
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ProviderId {
    /// Writes the canonical identifier text to `formatter`.
    ///
    /// Returns a formatting error if `formatter` cannot accept the text.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    /// Error returned when the input is empty or violates canonical ID syntax.
    type Err = ProviderIdError;

    /// Parses an already canonical provider identifier.
    ///
    /// `value` is validated without trimming or case normalization. Returns the
    /// canonical provider ID when validation succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderIdError`] when `value` is empty or noncanonical.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Tests the shared canonical-token grammar for IDs and selectors.
///
/// `value` must be nonempty lowercase ASCII, start and end with an
/// alphanumeric byte, and use only the permitted separators. Returns `true`
/// exactly when all requirements are satisfied.
pub(crate) fn is_canonical_token(value: &str) -> bool {
    !value.is_empty()
        && value.is_ascii()
        && value == value.trim()
        && !value.bytes().any(|byte| byte.is_ascii_uppercase())
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && !value
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_' | b'.' | b'+'))
}
