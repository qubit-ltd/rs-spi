// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider selectors parsed from configuration or user input.

use std::{
    fmt,
    str::FromStr,
};

use crate::ProviderId;
use crate::error::ProviderSelectorError;
use crate::provider_id::is_canonical_token;

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
    /// Surrounding whitespace is removed and ASCII letters are lowercased
    /// before the canonical identifier grammar is validated.
    ///
    /// # Arguments
    ///
    /// * `value` - Raw configuration or user input.
    ///
    /// # Returns
    ///
    /// The normalized selector used for registry lookup.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderSelectorError`] when the normalized selector is empty
    /// or invalid.
    ///
    /// # Performance
    ///
    /// Successful parsing stores an owned normalized selector and therefore
    /// allocates. Cache a [`ProviderSelector`] or
    /// [`crate::ProviderSelection`] when the same configured input is reused.
    #[inline]
    pub fn parse(value: &str) -> Result<Self, ProviderSelectorError> {
        let input = value;
        let normalized = input.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(ProviderSelectorError::empty(input));
        }
        if !is_canonical_token(&normalized) {
            return Err(ProviderSelectorError::invalid(input, &normalized));
        }
        Ok(Self(normalized.into()))
    }

    /// Returns the normalized selector text.
    ///
    /// # Returns
    ///
    /// The validated lowercase selector token.
    #[inline(always)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderSelector {
    /// Forwards to [`ProviderSelector::as_str`].
    ///
    /// # Returns
    ///
    /// The normalized selector text.
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&ProviderId> for ProviderSelector {
    /// Converts a validated canonical provider ID into a selector.
    ///
    /// # Arguments
    ///
    /// * `id` - Canonical provider ID whose invariant already satisfies the
    ///   selector grammar.
    ///
    /// # Returns
    ///
    /// A selector containing the same canonical text without reparsing.
    #[inline]
    fn from(id: &ProviderId) -> Self {
        Self(id.as_str().into())
    }
}

impl fmt::Display for ProviderSelector {
    /// Writes the normalized selector text to `formatter`.
    ///
    /// # Arguments
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

impl FromStr for ProviderSelector {
    /// Error returned when the normalized input is empty or invalid.
    type Err = ProviderSelectorError;

    /// Parses a provider selector from configuration-style input.
    ///
    /// # Arguments
    ///
    /// * `value` - Input trimmed and ASCII-lowercased before validation.
    ///
    /// # Returns
    ///
    /// The normalized selector used for registry lookup.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderSelectorError`] when the normalized input is empty or
    /// violates selector syntax.
    #[inline(always)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}
