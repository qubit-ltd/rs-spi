// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while resolving provider selections.

use std::{error::Error, sync::Arc};

use thiserror::Error;

use crate::{ProviderError, ProviderErrorKind, ProviderId, ProviderSelector, RegistrationError};

/// Classification of a failed provider-selection resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// A direct selector did not satisfy selector syntax.
    InvalidSelector,
    /// A named selector did not resolve to any registered provider.
    UnknownProvider,
    /// The provider selection produced no service.
    NoProviderSucceeded,
}

/// Classification of one failed resolution attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AttemptFailureKind {
    /// Selector lookup reached no provider.
    UnknownProvider,
    /// A resolved provider failed to create its service.
    ProviderError,
}

/// Diagnostic record for one candidate that could not produce a service.
#[derive(Clone, Debug)]
pub struct AttemptFailure {
    /// Variant-specific attempt diagnostics.
    repr: AttemptFailureRepr,
}

/// Private representation of a failed resolution attempt.
#[derive(Clone, Debug)]
enum AttemptFailureRepr {
    /// Selector lookup reached no provider.
    UnknownProvider {
        /// Selector retained from the request.
        requested_selector: ProviderSelector,
        /// Human-readable lookup failure.
        reason: Box<str>,
    },
    /// A provider factory returned a classified error.
    ProviderError {
        /// Explicit selector, or `None` for automatic selection.
        requested_selector: Option<ProviderSelector>,
        /// Canonical provider reached by lookup.
        provider_id: ProviderId,
        /// Provider-reported failure classification.
        provider_error_kind: ProviderErrorKind,
        /// Provider-supplied explanation.
        reason: Box<str>,
        /// Optional underlying provider cause.
        source: Option<Arc<dyn Error + Send + Sync>>,
    },
}

impl AttemptFailure {
    /// Creates a failed attempt for a selector that matched no provider.
    #[must_use]
    pub fn unknown_provider(selector: ProviderSelector) -> Self {
        Self {
            repr: AttemptFailureRepr::UnknownProvider {
                reason: format!("unknown provider: {selector}").into(),
                requested_selector: selector,
            },
        }
    }

    /// Creates a failed attempt from an error returned by a provider factory.
    #[must_use]
    pub fn provider_error(
        requested_selector: Option<ProviderSelector>,
        provider_id: ProviderId,
        error: &ProviderError,
    ) -> Self {
        Self {
            repr: AttemptFailureRepr::ProviderError {
                requested_selector,
                provider_id,
                provider_error_kind: error.kind(),
                reason: error.reason().into(),
                source: error.source_arc(),
            },
        }
    }

    /// Returns the explicit attempt classification.
    #[must_use]
    pub const fn kind(&self) -> AttemptFailureKind {
        match self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => AttemptFailureKind::UnknownProvider,
            AttemptFailureRepr::ProviderError { .. } => AttemptFailureKind::ProviderError,
        }
    }

    /// Returns the selector that requested this attempt.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider {
                requested_selector, ..
            } => Some(requested_selector),
            AttemptFailureRepr::ProviderError {
                requested_selector, ..
            } => requested_selector.as_ref(),
        }
    }

    /// Returns the canonical provider reached by selector lookup.
    #[must_use]
    pub fn provider_id(&self) -> Option<&ProviderId> {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => None,
            AttemptFailureRepr::ProviderError { provider_id, .. } => Some(provider_id),
        }
    }

    /// Returns the provider-reported creation failure classification.
    #[must_use]
    pub const fn provider_error_kind(&self) -> Option<ProviderErrorKind> {
        match self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => None,
            AttemptFailureRepr::ProviderError {
                provider_error_kind,
                ..
            } => Some(provider_error_kind),
        }
    }

    /// Returns the human-readable explanation for this failed attempt.
    #[must_use]
    pub fn reason(&self) -> &str {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { reason, .. }
            | AttemptFailureRepr::ProviderError { reason, .. } => reason,
        }
    }

    /// Returns the underlying cause retained from the provider error.
    #[must_use]
    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => None,
            AttemptFailureRepr::ProviderError { source, .. } => source
                .as_deref()
                .map(|source| source as &(dyn Error + 'static)),
        }
    }
}

/// Aggregate error returned when provider resolution cannot create a service.
#[derive(Clone, Debug, Error)]
#[error(transparent)]
pub struct ResolutionError(
    /// Private representation retaining variant-specific diagnostics.
    ResolutionErrorRepr,
);

/// Private representation of aggregate resolution failures.
#[derive(Clone, Debug, Error)]
enum ResolutionErrorRepr {
    /// Direct selector input was invalid.
    #[error("invalid provider selector {input:?}")]
    InvalidSelector {
        /// Verbatim input supplied by the caller.
        input: Box<str>,
        /// Selector grammar validation failure.
        #[source]
        source: RegistrationError,
    },
    /// A valid normalized selector matched no provider.
    #[error("unknown provider: {selector}")]
    UnknownProvider {
        /// Normalized unknown selector.
        selector: ProviderSelector,
    },
    /// No candidate produced a service.
    #[error("no provider succeeded after {} attempt(s)", attempts.len())]
    NoProviderSucceeded {
        /// Failures retained in encounter order.
        attempts: Box<[AttemptFailure]>,
    },
}

impl ResolutionError {
    /// Creates an error for a direct selector that matched no provider.
    #[must_use]
    pub fn unknown_provider(selector: impl AsRef<str>) -> Self {
        let input = selector.as_ref();
        match ProviderSelector::parse(input) {
            Ok(selector) => Self(ResolutionErrorRepr::UnknownProvider { selector }),
            Err(source) => Self::invalid_selector(input, source),
        }
    }

    /// Creates an error for invalid direct selector input.
    #[must_use]
    pub(crate) fn invalid_selector(input: impl AsRef<str>, source: RegistrationError) -> Self {
        Self(ResolutionErrorRepr::InvalidSelector {
            input: input.as_ref().into(),
            source,
        })
    }

    /// Creates an aggregate error when a selection produces no service.
    #[must_use]
    pub fn no_provider_succeeded(attempts: impl Into<Box<[AttemptFailure]>>) -> Self {
        Self(ResolutionErrorRepr::NoProviderSucceeded {
            attempts: attempts.into(),
        })
    }

    /// Returns the overall resolution failure classification.
    #[must_use]
    pub const fn kind(&self) -> ResolutionErrorKind {
        match self.0 {
            ResolutionErrorRepr::InvalidSelector { .. } => ResolutionErrorKind::InvalidSelector,
            ResolutionErrorRepr::UnknownProvider { .. } => ResolutionErrorKind::UnknownProvider,
            ResolutionErrorRepr::NoProviderSucceeded { .. } => {
                ResolutionErrorKind::NoProviderSucceeded
            }
        }
    }

    /// Returns the original direct selector input, when one exists.
    #[must_use]
    pub fn selector_input(&self) -> Option<&str> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { input, .. } => Some(input),
            ResolutionErrorRepr::UnknownProvider { selector } => Some(selector.as_str()),
            ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the normalized selector retained for a direct lookup failure.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        match &self.0 {
            ResolutionErrorRepr::UnknownProvider { selector } => Some(selector),
            ResolutionErrorRepr::InvalidSelector { .. }
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns failed attempts in the order candidates were considered.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFailure] {
        match &self.0 {
            ResolutionErrorRepr::NoProviderSucceeded { attempts } => attempts,
            ResolutionErrorRepr::InvalidSelector { .. }
            | ResolutionErrorRepr::UnknownProvider { .. } => &[],
        }
    }
}
