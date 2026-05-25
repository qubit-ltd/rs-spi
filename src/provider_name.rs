/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Strongly typed provider names.

use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};
use std::str::FromStr;

use crate::ProviderRegistryError;

/// Stable provider id or alias accepted by a registry.
///
/// Provider names are normalized by trimming surrounding whitespace and folding
/// ASCII letters to lowercase. Valid names may contain ASCII letters, digits,
/// `_`, and `-`, must start and end with an ASCII letter or digit, and must not
/// contain consecutive separators.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProviderName(String);

impl ProviderName {
    /// Creates a normalized provider name.
    ///
    /// # Parameters
    /// - `name`: Raw provider id, alias, or selector.
    ///
    /// # Returns
    /// Normalized provider name.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::EmptyProviderName`] when `name` is empty
    /// after trimming. Returns [`ProviderRegistryError::InvalidProviderName`]
    /// when `name` is non-ASCII or contains unsupported characters.
    #[inline]
    pub fn new(name: &str) -> Result<Self, ProviderRegistryError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(ProviderRegistryError::EmptyProviderName);
        }
        if !trimmed.is_ascii() {
            return Err(invalid_provider_name(trimmed, "provider names must be ASCII"));
        }
        if !trimmed.bytes().all(is_allowed_provider_name_byte) {
            return Err(invalid_provider_name(
                trimmed,
                "provider names may contain only ASCII letters, digits, '_' or '-'",
            ));
        }
        let bytes = trimmed.as_bytes();
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if !first.is_ascii_alphanumeric() {
            return Err(invalid_provider_name(
                trimmed,
                "provider names must start with an ASCII letter or digit",
            ));
        }
        if !last.is_ascii_alphanumeric() {
            return Err(invalid_provider_name(
                trimmed,
                "provider names must end with an ASCII letter or digit",
            ));
        }
        if has_consecutive_separators(trimmed) {
            return Err(invalid_provider_name(
                trimmed,
                "provider names must not contain consecutive separators",
            ));
        }
        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    /// Gets the normalized provider name.
    ///
    /// # Returns
    /// Normalized provider name string.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for ProviderName {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProviderName {
    type Err = ProviderRegistryError;

    #[inline]
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::new(name)
    }
}

/// Tells whether one ASCII byte is allowed in a provider name.
///
/// # Parameters
/// - `byte`: Byte to validate.
///
/// # Returns
/// `true` when the byte is accepted by the provider-name grammar.
fn is_allowed_provider_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || is_separator_provider_name_byte(byte)
}

/// Tells whether one ASCII byte is a provider-name separator.
///
/// # Parameters
/// - `byte`: Byte to validate.
///
/// # Returns
/// `true` when the byte is a provider-name separator.
fn is_separator_provider_name_byte(byte: u8) -> bool {
    matches!(byte, b'_' | b'-')
}

/// Tells whether a provider name contains consecutive separators.
///
/// # Parameters
/// - `name`: Provider name after trimming and character validation.
///
/// # Returns
/// `true` when any two adjacent bytes are separators.
fn has_consecutive_separators(name: &str) -> bool {
    name.as_bytes()
        .windows(2)
        .any(|bytes| is_separator_provider_name_byte(bytes[0]) && is_separator_provider_name_byte(bytes[1]))
}

/// Builds an invalid provider-name error.
///
/// # Parameters
/// - `name`: Invalid provider name after trimming.
/// - `reason`: Human-readable validation failure reason.
///
/// # Returns
/// Invalid provider-name error.
fn invalid_provider_name(name: &str, reason: &str) -> ProviderRegistryError {
    ProviderRegistryError::InvalidProviderName {
        name: name.to_owned(),
        reason: reason.to_owned(),
    }
}
