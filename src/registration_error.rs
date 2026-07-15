// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while building a provider registry.

use std::fmt;

/// Classification of a registry-registration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistrationErrorKind {
    /// An identifier was empty after applying its input rules.
    EmptyIdentifier,
    /// An identifier used unsupported characters or structure.
    InvalidIdentifier,
    /// A canonical identifier or alias was already owned by another provider.
    DuplicateSelector,
}

/// Error raised while validating a provider registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationError {
    kind: RegistrationErrorKind,
    identifier: Option<Box<str>>,
    existing_provider: Option<Box<str>>,
    provider: Option<Box<str>>,
}

impl RegistrationError {
    /// Creates an error for an empty identifier.
    #[must_use]
    pub fn empty_identifier() -> Self {
        Self {
            kind: RegistrationErrorKind::EmptyIdentifier,
            identifier: None,
            existing_provider: None,
            provider: None,
        }
    }

    /// Creates an error for an invalid identifier.
    #[must_use]
    pub fn invalid_identifier(identifier: impl AsRef<str>) -> Self {
        Self {
            kind: RegistrationErrorKind::InvalidIdentifier,
            identifier: Some(identifier.as_ref().into()),
            existing_provider: None,
            provider: None,
        }
    }

    /// Creates an error for a selector conflict.
    #[must_use]
    pub fn duplicate_selector(
        identifier: impl AsRef<str>,
        existing_provider: impl AsRef<str>,
        provider: impl AsRef<str>,
    ) -> Self {
        Self {
            kind: RegistrationErrorKind::DuplicateSelector,
            identifier: Some(identifier.as_ref().into()),
            existing_provider: Some(existing_provider.as_ref().into()),
            provider: Some(provider.as_ref().into()),
        }
    }

    /// Gets the error classification.
    #[must_use]
    pub const fn kind(&self) -> RegistrationErrorKind {
        self.kind
    }

    /// Gets the invalid or conflicting identifier when one is available.
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Gets the canonical provider that already owns a conflicting selector.
    #[must_use]
    pub fn existing_provider(&self) -> Option<&str> {
        self.existing_provider.as_deref()
    }

    /// Gets the canonical provider that attempted to claim a selector.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            RegistrationErrorKind::EmptyIdentifier => {
                formatter.write_str("provider identifier must not be empty")
            }
            RegistrationErrorKind::InvalidIdentifier => write!(
                formatter,
                "invalid provider identifier: {}",
                self.identifier().unwrap_or("<missing>"),
            ),
            RegistrationErrorKind::DuplicateSelector => write!(
                formatter,
                "provider selector {} is already owned by {}",
                self.identifier().unwrap_or("<missing>"),
                self.existing_provider().unwrap_or("<missing>"),
            ),
        }
    }
}

impl std::error::Error for RegistrationError {}
