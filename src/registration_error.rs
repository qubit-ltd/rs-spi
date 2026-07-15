// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while building a provider registry.

use thiserror::Error;

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
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct RegistrationError(
    /// Private representation retaining variant-specific diagnostics.
    RegistrationErrorRepr,
);

/// Private representation of registration validation failures.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
enum RegistrationErrorRepr {
    /// An identifier was empty after input processing.
    #[error("provider identifier must not be empty")]
    EmptyIdentifier,
    /// An identifier violated the canonical grammar.
    #[error("invalid provider identifier: {identifier}")]
    InvalidIdentifier {
        /// Verbatim invalid identifier.
        identifier: Box<str>,
    },
    /// A selector was already owned by a registered provider.
    #[error(
        "provider selector {identifier} claimed by {provider} is already owned by {existing_provider}"
    )]
    DuplicateSelector {
        /// Conflicting canonical ID or alias.
        identifier: Box<str>,
        /// Canonical ID that already owns the selector.
        existing_provider: Box<str>,
        /// Canonical ID attempting the new claim.
        provider: Box<str>,
    },
}

impl RegistrationError {
    /// Creates an error for an identifier that is empty after input processing.
    ///
    /// Returns a [`RegistrationErrorKind::EmptyIdentifier`] error without an
    /// identifier or provider ownership details.
    #[must_use]
    pub fn empty_identifier() -> Self {
        Self(RegistrationErrorRepr::EmptyIdentifier)
    }

    /// Creates an error for an identifier that violates the canonical grammar.
    ///
    /// `identifier` is retained verbatim for diagnostics. Returns an
    /// [`RegistrationErrorKind::InvalidIdentifier`] error containing that value.
    #[must_use]
    pub fn invalid_identifier(identifier: impl AsRef<str>) -> Self {
        Self(RegistrationErrorRepr::InvalidIdentifier {
            identifier: identifier.as_ref().into(),
        })
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
        Self(RegistrationErrorRepr::DuplicateSelector {
            identifier: identifier.as_ref().into(),
            existing_provider: existing_provider.as_ref().into(),
            provider: provider.as_ref().into(),
        })
    }

    /// Returns the registration rule that failed.
    #[must_use]
    pub const fn kind(&self) -> RegistrationErrorKind {
        match self.0 {
            RegistrationErrorRepr::EmptyIdentifier => RegistrationErrorKind::EmptyIdentifier,
            RegistrationErrorRepr::InvalidIdentifier { .. } => {
                RegistrationErrorKind::InvalidIdentifier
            }
            RegistrationErrorRepr::DuplicateSelector { .. } => {
                RegistrationErrorKind::DuplicateSelector
            }
        }
    }

    /// Returns the invalid or conflicting identifier, when one was recorded.
    ///
    /// Returns `Some` for invalid-identifier and duplicate-selector errors, and
    /// `None` for an empty-identifier error.
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        match &self.0 {
            RegistrationErrorRepr::EmptyIdentifier => None,
            RegistrationErrorRepr::InvalidIdentifier { identifier }
            | RegistrationErrorRepr::DuplicateSelector { identifier, .. } => Some(identifier),
        }
    }

    /// Returns the canonical provider that already owns a conflicting selector.
    ///
    /// Returns `Some` for a duplicate-selector error and `None` for other kinds.
    #[must_use]
    pub fn existing_provider(&self) -> Option<&str> {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector {
                existing_provider, ..
            } => Some(existing_provider),
            RegistrationErrorRepr::EmptyIdentifier
            | RegistrationErrorRepr::InvalidIdentifier { .. } => None,
        }
    }

    /// Returns the canonical provider that attempted to claim a selector.
    ///
    /// Returns `Some` for a duplicate-selector error and `None` for other kinds.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector { provider, .. } => Some(provider),
            RegistrationErrorRepr::EmptyIdentifier
            | RegistrationErrorRepr::InvalidIdentifier { .. } => None,
        }
    }
}
