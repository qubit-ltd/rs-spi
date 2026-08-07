// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict canonical provider identifiers.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use crate::error::ProviderIdError;

/// Stable canonical identifier of a provider.
///
/// Use this type to assign a provider its unique registry identity. Unlike a
/// [`crate::ProviderSelector`], an ID must already be canonical and is never
/// trimmed, lowercased, or otherwise normalized.
///
/// # Canonical form
///
/// A legal [`ProviderId`] is a nonempty ASCII token that already satisfies all
/// of the following rules:
///
/// * **Nonempty** — the empty string is rejected.
/// * **ASCII only** — every byte must be an ASCII character; non-ASCII text
///   such as `"文件"` is rejected.
/// * **No surrounding whitespace** — leading or trailing spaces or tabs are
///   rejected; whitespace is not stripped.
/// * **Lowercase only** — ASCII uppercase letters (`A`–`Z`) are rejected.
/// * **Alphanumeric endpoints** — the first and last characters must each be an
///   ASCII letter (`a`–`z`) or digit (`0`–`9`).
/// * **Allowed body characters** — every other character must be an ASCII
///   letter, digit, or one of the separators `-`, `_`, `.`, and `+`.
///
/// Characters outside that set (for example `/`, spaces inside the token, or
/// control characters) are rejected. Consecutive separators are allowed, so
/// values such as `"a--b"`, `"a..b"`, `"git+ssh"`, and `"vendor.v2"` are valid.
///
/// # Examples
///
/// ```
/// use qubit_spi::ProviderId;
///
/// assert!(ProviderId::new("file-command").is_ok());
/// assert!(ProviderId::new("vendor.v2").is_ok());
/// assert!(ProviderId::new("File").is_err());
/// assert!(ProviderId::new("-file").is_err());
/// assert!(ProviderId::new("file-").is_err());
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderId(
    /// Shared canonical ASCII identifier text.
    Arc<str>,
);

impl ProviderId {
    /// Creates a canonical provider identifier from already canonical text.
    ///
    /// The input must already satisfy the [canonical
    /// form](ProviderId#canonical-form) rules. This constructor does not
    /// trim whitespace or change letter case.
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
        if !Self::is_canonical_token(value) {
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
        let Self(value) = self;
        value
    }

    /// Tests the shared canonical-token grammar for IDs and selectors.
    ///
    /// Returns whether `value` already satisfies the
    /// [canonical form](ProviderId#canonical-form) rules without normalizing
    /// the input.
    ///
    /// # Parameters
    ///
    /// * `value` - Candidate token to validate without normalization.
    ///
    /// # Returns
    ///
    /// `true` when the input is a nonempty lowercase ASCII token with
    /// alphanumeric endpoints and only permitted separators; otherwise,
    /// `false`.
    #[must_use]
    pub(crate) const fn is_canonical_token(value: &str) -> bool {
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return false;
        }
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            if !byte.is_ascii_lowercase()
                && !byte.is_ascii_digit()
                && !matches!(byte, b'-' | b'_' | b'.' | b'+')
            {
                return false;
            }
            if (index == 0 || index + 1 == bytes.len())
                && !byte.is_ascii_alphanumeric()
            {
                return false;
            }
            index += 1;
        }
        true
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
    /// Equivalent to [`ProviderId::new`]. The input must already satisfy the
    /// [canonical form](ProviderId#canonical-form) rules and is not normalized.
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
