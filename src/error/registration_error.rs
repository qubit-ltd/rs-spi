// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Conflicts raised while registering providers.

use thiserror::Error;

use super::RegistrationErrorKind;

/// Error returned when a provider registration conflicts with registry state.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum RegistrationError {
    /// A selector is already owned by a registered provider.
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
    /// # Arguments
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

    /// Returns this registration error's stable classification.
    ///
    /// # Returns
    ///
    /// The registry-conflict classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> RegistrationErrorKind {
        match self {
            Self::DuplicateSelector { .. } => {
                RegistrationErrorKind::DuplicateSelector
            }
        }
    }

    /// Returns the conflicting selector.
    ///
    /// # Returns
    ///
    /// The canonical ID or alias claimed by both providers.
    #[inline(always)]
    #[must_use]
    pub fn selector(&self) -> &str {
        match self {
            Self::DuplicateSelector { selector, .. } => selector,
        }
    }

    /// Returns the provider that already owns the selector.
    ///
    /// # Returns
    ///
    /// The canonical ID of the existing provider.
    #[inline(always)]
    #[must_use]
    pub fn existing_provider(&self) -> &str {
        match self {
            Self::DuplicateSelector {
                existing_provider, ..
            } => existing_provider,
        }
    }

    /// Returns the provider whose registration was rejected.
    ///
    /// # Returns
    ///
    /// The canonical ID attempting to claim the selector.
    #[inline(always)]
    #[must_use]
    pub fn provider(&self) -> &str {
        match self {
            Self::DuplicateSelector { provider, .. } => provider,
        }
    }
}
