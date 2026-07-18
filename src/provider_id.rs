// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict canonical provider identifiers.

use std::{
    fmt,
    str::FromStr,
    sync::Arc,
};

use crate::error::ProviderIdError;

/// Stable canonical identifier of a provider.
///
/// Use this type to assign a provider its unique registry identity. Unlike a
/// selector, an ID must already be canonical and is never normalized.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderId(
    /// Shared canonical ASCII identifier text.
    Arc<str>,
);

impl ProviderId {
    /// Creates a canonical provider identifier from already canonical text.
    ///
    /// The value must already be lowercase ASCII and may contain alphanumeric
    /// characters plus hyphen, underscore, period, and plus between
    /// alphanumeric endpoints.
    ///
    /// # Parameters
    ///
    /// * `value` - Candidate canonical identifier text.
    ///
    /// # Returns
    ///
    /// The validated stable provider identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderIdError`] when `value` is empty or noncanonical.
    #[inline]
    pub fn new(value: &str) -> Result<Self, ProviderIdError> {
        if value.is_empty() {
            return Err(ProviderIdError::empty(value));
        }
        if !is_canonical_token(value) {
            return Err(ProviderIdError::noncanonical(value));
        }
        Ok(Self(Arc::from(value)))
    }

    /// Returns the canonical identifier text.
    ///
    /// # Returns
    ///
    /// The validated canonical token.
    #[inline(always)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderId {
    /// Forwards to [`ProviderId::as_str`].
    ///
    /// # Returns
    ///
    /// The canonical provider ID text.
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ProviderId {
    /// Writes the canonical identifier text to `formatter`.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the destination formatter rejects the text.
    #[inline(always)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    /// Error returned when the input is empty or violates canonical ID syntax.
    type Err = ProviderIdError;

    /// Parses an already canonical provider identifier.
    ///
    /// # Parameters
    ///
    /// * `value` - Input validated without trimming or case normalization.
    ///
    /// # Returns
    ///
    /// The canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderIdError`] when `value` is empty or noncanonical.
    #[inline(always)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Tests the shared canonical-token grammar for IDs and selectors.
///
/// # Parameters
///
/// * `value` - Candidate token to validate without normalization.
///
/// # Returns
///
/// `true` when the input is nonempty lowercase ASCII, has alphanumeric
/// endpoints, and contains only permitted separators; otherwise, `false`.
#[must_use]
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
        && !value.bytes().any(|byte| {
            !byte.is_ascii_alphanumeric()
                && !matches!(byte, b'-' | b'_' | b'.' | b'+')
        })
}
