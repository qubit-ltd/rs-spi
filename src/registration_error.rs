// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while building a provider registry.

use std::fmt;

/// Classification of a failure detected while validating registry metadata.
///
/// Inspect this value when callers need to distinguish malformed identifiers
/// from conflicts with registrations already accepted by a registry builder.
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

/// Error raised while validating provider identifiers, aliases, or ownership.
///
/// Provider IDs, selectors, selections, and descriptor aliases return this
/// error while their input is being constructed or normalized. A registry
/// builder also returns it before mutation when a canonical ID or alias is
/// already owned. The optional diagnostic fields expose the values relevant to
/// the specific [`Self::kind`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationError {
    /// Classification identifying the registration rule that failed.
    kind: RegistrationErrorKind,
    /// Invalid or conflicting identifier, when the error concerns one.
    identifier: Option<Box<str>>,
    /// Canonical ID of the provider that already owns a conflicting selector.
    existing_provider: Option<Box<str>>,
    /// Canonical ID of the provider that attempted to claim the selector.
    provider: Option<Box<str>>,
}

impl RegistrationError {
    /// Creates an error for an identifier that is empty after input processing.
    ///
    /// Returns a [`RegistrationErrorKind::EmptyIdentifier`] error without an
    /// identifier or provider ownership details.
    #[must_use]
    pub fn empty_identifier() -> Self {
        Self {
            kind: RegistrationErrorKind::EmptyIdentifier,
            identifier: None,
            existing_provider: None,
            provider: None,
        }
    }

    /// Creates an error for an identifier that violates the canonical grammar.
    ///
    /// `identifier` is retained verbatim for diagnostics. Returns an
    /// [`RegistrationErrorKind::InvalidIdentifier`] error containing that value.
    #[must_use]
    pub fn invalid_identifier(identifier: impl AsRef<str>) -> Self {
        Self {
            kind: RegistrationErrorKind::InvalidIdentifier,
            identifier: Some(identifier.as_ref().into()),
            existing_provider: None,
            provider: None,
        }
    }

    /// Creates an error for a selector already claimed by another provider.
    ///
    /// `identifier` is the conflicting canonical ID or alias,
    /// `existing_provider` is its current canonical owner, and `provider` is the
    /// canonical ID attempting the new claim. Returns a
    /// [`RegistrationErrorKind::DuplicateSelector`] error retaining all three.
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

    /// Returns the registration rule that failed.
    #[must_use]
    pub const fn kind(&self) -> RegistrationErrorKind {
        self.kind
    }

    /// Returns the invalid or conflicting identifier, when one was recorded.
    ///
    /// Returns `Some` for invalid-identifier and duplicate-selector errors, and
    /// `None` for an empty-identifier error.
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Returns the canonical provider that already owns a conflicting selector.
    ///
    /// Returns `Some` for a duplicate-selector error and `None` for other kinds.
    #[must_use]
    pub fn existing_provider(&self) -> Option<&str> {
        self.existing_provider.as_deref()
    }

    /// Returns the canonical provider that attempted to claim a selector.
    ///
    /// Returns `Some` for a duplicate-selector error and `None` for other kinds.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        self.provider.as_deref()
    }
}

impl fmt::Display for RegistrationError {
    /// Formats the failed registration rule and its available diagnostic values.
    ///
    /// `formatter` receives a human-readable message selected from [`Self::kind`].
    /// Returns a formatting error if the formatter cannot accept the message.
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
