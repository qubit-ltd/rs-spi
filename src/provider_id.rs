// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Strict canonical provider identifiers.

use std::{fmt, str::FromStr};

use crate::RegistrationError;

/// Stable canonical identifier of a provider.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderId(Box<str>);

impl ProviderId {
    /// Creates a canonical provider identifier.
    ///
    /// The value must already be lowercase ASCII and may contain alphanumeric
    /// characters plus hyphen, underscore, period, and plus between
    /// alphanumeric endpoints.
    ///
    /// # Errors
    ///
    /// Returns RegistrationError when the value is empty or noncanonical.
    pub fn new(value: impl AsRef<str>) -> Result<Self, RegistrationError> {
        let value = value.as_ref();
        validate_canonical_token(value)?;
        Ok(Self(value.into()))
    }

    /// Gets the canonical identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    type Err = RegistrationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

pub(crate) fn validate_canonical_token(value: &str) -> Result<(), RegistrationError> {
    if value.is_empty() {
        return Err(RegistrationError::empty_identifier());
    }
    if !value.is_ascii()
        || value != value.trim()
        || value.bytes().any(|byte| byte.is_ascii_uppercase())
        || !value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        || !value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_' | b'.' | b'+'))
    {
        return Err(RegistrationError::invalid_identifier(value));
    }
    Ok(())
}
