// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while resolving provider selections.

use std::{error::Error, fmt, sync::Arc};

use crate::{
    ProviderError, ProviderErrorKind, ProviderId, ProviderSelector, ProviderSelectorError,
};

/// Classification of a failed provider-selection resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// A raw selector does not satisfy selector syntax.
    InvalidSelector,
    /// A raw chained selection contains no selectors.
    EmptySelection,
    /// A named selector does not resolve to a registered provider.
    UnknownProvider,
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// At least one candidate was considered but no service was produced.
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

impl fmt::Display for AttemptFailure {
    /// Formats this failed attempt with selector or provider context.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { reason, .. } => formatter.write_str(reason),
            AttemptFailureRepr::ProviderError {
                requested_selector,
                provider_id,
                provider_error_kind,
                reason,
                ..
            } => {
                write!(
                    formatter,
                    "provider {provider_id} failed with {provider_error_kind:?}: {reason}"
                )?;
                if let Some(selector) = requested_selector {
                    write!(formatter, " (requested as {selector})")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for AttemptFailure {
    /// Returns the retained provider cause, when one exists.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Self::source(self)
    }
}

/// Aggregate error returned when provider resolution cannot create a service.
#[derive(Clone, Debug)]
pub struct ResolutionError(ResolutionErrorRepr);

/// Private representation of aggregate resolution failures.
#[derive(Clone, Debug)]
enum ResolutionErrorRepr {
    /// Raw selector input was invalid.
    InvalidSelector {
        /// Verbatim input supplied by the caller.
        input: Box<str>,
        /// Zero-based chain position, or `None` for named selection.
        selector_index: Option<usize>,
        /// Selector parsing failure.
        source: ProviderSelectorError,
    },
    /// A raw chained selection contains no inputs.
    EmptySelection,
    /// A valid normalized selector matched no provider.
    UnknownProvider {
        /// Normalized unknown selector.
        selector: ProviderSelector,
    },
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// No considered candidate produced a service.
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
            Err(source) => Self::invalid_selector(input, None, source),
        }
    }

    /// Creates an error for invalid raw selector input.
    #[must_use]
    pub(crate) fn invalid_selector(
        input: impl AsRef<str>,
        selector_index: Option<usize>,
        source: ProviderSelectorError,
    ) -> Self {
        Self(ResolutionErrorRepr::InvalidSelector {
            input: input.as_ref().into(),
            selector_index,
            source,
        })
    }

    /// Creates an error for an empty raw chained selection.
    #[must_use]
    pub(crate) const fn empty_selection() -> Self {
        Self(ResolutionErrorRepr::EmptySelection)
    }

    /// Creates an error for automatic resolution from an empty registry.
    #[must_use]
    pub(crate) const fn empty_registry() -> Self {
        Self(ResolutionErrorRepr::EmptyRegistry)
    }

    /// Creates an aggregate error when considered candidates produce no service.
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
            ResolutionErrorRepr::EmptySelection => ResolutionErrorKind::EmptySelection,
            ResolutionErrorRepr::UnknownProvider { .. } => ResolutionErrorKind::UnknownProvider,
            ResolutionErrorRepr::EmptyRegistry => ResolutionErrorKind::EmptyRegistry,
            ResolutionErrorRepr::NoProviderSucceeded { .. } => {
                ResolutionErrorKind::NoProviderSucceeded
            }
        }
    }

    /// Returns the original selector input, when one exists.
    #[must_use]
    pub fn selector_input(&self) -> Option<&str> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { input, .. } => Some(input),
            ResolutionErrorRepr::UnknownProvider { selector } => Some(selector.as_str()),
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the invalid selector's zero-based chain position, when present.
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        match self.0 {
            ResolutionErrorRepr::InvalidSelector { selector_index, .. } => selector_index,
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the selector parsing failure for invalid raw input.
    #[must_use]
    pub fn selector_error(&self) -> Option<&ProviderSelectorError> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { source, .. } => Some(source),
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the normalized selector retained for a direct lookup failure.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        match &self.0 {
            ResolutionErrorRepr::UnknownProvider { selector } => Some(selector),
            ResolutionErrorRepr::InvalidSelector { .. }
            | ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns failed attempts in the order candidates were considered.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFailure] {
        match &self.0 {
            ResolutionErrorRepr::NoProviderSucceeded { attempts } => attempts,
            ResolutionErrorRepr::InvalidSelector { .. }
            | ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry => &[],
        }
    }
}

impl fmt::Display for ResolutionError {
    /// Formats the resolution failure and ordered attempt diagnostics.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector {
                input,
                selector_index,
                ..
            } => match selector_index {
                Some(index) => write!(
                    formatter,
                    "invalid provider selector at chain index {index}: {input:?}"
                ),
                None => write!(formatter, "invalid provider selector {input:?}"),
            },
            ResolutionErrorRepr::EmptySelection => {
                formatter.write_str("provider selection chain must not be empty")
            }
            ResolutionErrorRepr::UnknownProvider { selector } => {
                write!(formatter, "unknown provider: {selector}")
            }
            ResolutionErrorRepr::EmptyRegistry => {
                formatter.write_str("cannot resolve a provider from an empty registry")
            }
            ResolutionErrorRepr::NoProviderSucceeded { attempts } => {
                write!(
                    formatter,
                    "no provider succeeded after {} attempt(s)",
                    attempts.len()
                )?;
                for (index, attempt) in attempts.iter().enumerate() {
                    write!(formatter, "; attempt {}: {attempt}", index + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ResolutionError {
    /// Returns the selector parsing source for invalid raw input.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { source, .. } => Some(source),
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }
}
