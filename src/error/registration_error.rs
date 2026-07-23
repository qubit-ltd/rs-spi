// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Conflicts raised while registering providers.

use thiserror::Error;

/// Error returned when a provider registration conflicts with registry state.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum RegistrationError {
    /// A selector is already owned by a registered provider.
    #[non_exhaustive]
    #[error(
        "provider selector {selector} claimed by {provider} is already owned by {existing_provider}"
    )]
    DuplicateSelector {
        /// Conflicting canonical ID or alias.
        selector: Box<str>,
        /// Canonical ID that already owns the selector.
        existing_provider: Box<str>,
        /// Canonical ID attempting the new claim.
        provider: Box<str>,
    },
}

impl RegistrationError {
    /// Creates an error for a selector claimed by another provider.
    ///
    /// # Parameters
    ///
    /// * `selector` - Conflicting canonical ID or alias.
    /// * `existing_provider` - Canonical provider currently owning the
    ///   selector.
    /// * `provider` - Canonical provider attempting the new claim.
    ///
    /// # Returns
    ///
    /// A registry-owned duplicate-selector error.
    #[inline]
    #[must_use]
    pub(crate) fn duplicate_selector(
        selector: &str,
        existing_provider: &str,
        provider: &str,
    ) -> Self {
        Self::DuplicateSelector {
            selector: selector.into(),
            existing_provider: existing_provider.into(),
            provider: provider.into(),
        }
    }

    /// Returns the canonical ID or alias that caused this conflict.
    ///
    /// # Returns
    ///
    /// The selector claimed by both providers.
    #[inline(always)]
    #[must_use]
    pub fn selector(&self) -> &str {
        match self {
            Self::DuplicateSelector { selector, .. } => selector,
        }
    }

    /// Returns the canonical ID of the provider that owns the selector.
    ///
    /// # Returns
    ///
    /// The registered provider that already owns the conflicting selector.
    #[inline(always)]
    #[must_use]
    pub fn existing_provider(&self) -> &str {
        match self {
            Self::DuplicateSelector {
                existing_provider, ..
            } => existing_provider,
        }
    }

    /// Returns the canonical ID of the provider attempting registration.
    ///
    /// # Returns
    ///
    /// The provider whose registration conflicts with existing registry state.
    #[inline(always)]
    #[must_use]
    pub fn provider(&self) -> &str {
        match self {
            Self::DuplicateSelector { provider, .. } => provider,
        }
    }
}
