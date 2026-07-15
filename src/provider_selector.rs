// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider selectors parsed from configuration or user input.

use std::{fmt, str::FromStr};

use crate::{RegistrationError, provider_id::validate_canonical_token};

/// Normalized provider lookup token.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderSelector(Box<str>);

impl ProviderSelector {
    /// Parses a provider selector from configuration input.
    ///
    /// Surrounding whitespace is removed and ASCII letters are lowercased
    /// before the canonical identifier grammar is validated.
    ///
    /// # Errors
    ///
    /// Returns RegistrationError when the normalized selector is empty or
    /// invalid.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, RegistrationError> {
        let normalized = value.as_ref().trim().to_ascii_lowercase();
        validate_canonical_token(&normalized)?;
        Ok(Self(normalized.into()))
    }

    /// Gets the normalized selector text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderSelector {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ProviderSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProviderSelector {
    type Err = RegistrationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}
