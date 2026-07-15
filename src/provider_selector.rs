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

/// Normalized token used to look up a provider by ID or alias.
///
/// This type is used at configuration and request boundaries, where inputs are
/// trimmed and ASCII-lowercased before registry lookup.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProviderSelector(
    /// Normalized selector text accepted by registry lookup.
    Box<str>,
);

impl ProviderSelector {
    /// Parses and normalizes a provider selector from configuration input.
    ///
    /// `value` is trimmed and ASCII-lowercased before validation. Returns the
    /// normalized selector used for registry lookup.
    ///
    /// Surrounding whitespace is removed and ASCII letters are lowercased
    /// before the canonical identifier grammar is validated.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the normalized selector is empty or
    /// invalid.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, RegistrationError> {
        let normalized = value.as_ref().trim().to_ascii_lowercase();
        validate_canonical_token(&normalized)?;
        Ok(Self(normalized.into()))
    }

    /// Returns the normalized selector text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderSelector {
    /// Forwards to [`ProviderSelector::as_str`].
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ProviderSelector {
    /// Writes the normalized selector text to `formatter`.
    ///
    /// Returns a formatting error if `formatter` cannot accept the text.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ProviderSelector {
    /// Error returned when the normalized input is empty or invalid.
    type Err = RegistrationError;

    /// Parses a provider selector from configuration-style input.
    ///
    /// `value` is trimmed and ASCII-lowercased before validation. Returns the
    /// normalized selector used for registry lookup.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the normalized input is empty or
    /// violates selector syntax.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}
