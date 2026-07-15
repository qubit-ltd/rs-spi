// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Conflicts raised while registering providers.

use thiserror::Error;

/// Classification of a provider registration conflict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistrationErrorKind {
    /// A canonical ID or alias is already owned by another registration.
    DuplicateSelector,
}

/// Error returned when a provider registration conflicts with registry state.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct RegistrationError(RegistrationErrorRepr);

/// Private representation of provider registration conflicts.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
enum RegistrationErrorRepr {
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
    /// Creates an error for a selector already claimed by another provider.
    ///
    /// `selector` is the conflicting canonical ID or alias,
    /// `existing_provider` is its current canonical owner, and `provider` is the
    /// canonical ID attempting the new claim.
    #[must_use]
    pub fn duplicate_selector(
        selector: impl AsRef<str>,
        existing_provider: impl AsRef<str>,
        provider: impl AsRef<str>,
    ) -> Self {
        Self(RegistrationErrorRepr::DuplicateSelector {
            selector: selector.as_ref().into(),
            existing_provider: existing_provider.as_ref().into(),
            provider: provider.as_ref().into(),
        })
    }

    /// Returns the registration conflict classification.
    #[must_use]
    pub const fn kind(&self) -> RegistrationErrorKind {
        RegistrationErrorKind::DuplicateSelector
    }

    /// Returns the canonical ID or alias that conflicts.
    #[must_use]
    pub fn selector(&self) -> &str {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector { selector, .. } => selector,
        }
    }

    /// Returns the canonical provider that already owns the selector.
    #[must_use]
    pub fn existing_provider(&self) -> &str {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector {
                existing_provider, ..
            } => existing_provider,
        }
    }

    /// Returns the canonical provider that attempted to claim the selector.
    #[must_use]
    pub fn provider(&self) -> &str {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector { provider, .. } => provider,
        }
    }
}
