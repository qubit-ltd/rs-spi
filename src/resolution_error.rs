// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while resolving provider selections.

use std::{error::Error, fmt, sync::Arc};

use crate::{ProviderError, ProviderErrorKind, ProviderId, ProviderSelector};

/// Classification of a failed provider-selection resolution.
///
/// Inspect this value to distinguish a failed direct lookup from an exhausted
/// automatic or explicit fallback sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// A named selector did not resolve to any registered provider.
    UnknownProvider,
    /// The provider selection produced no service.
    NoProviderSucceeded,
}

/// Diagnostic record for one candidate that could not produce a service.
///
/// Resolvers collect these values in attempt order so callers can determine
/// which selector was requested, which canonical provider was reached, and why
/// lookup or service creation failed.
#[derive(Clone, Debug)]
pub struct AttemptFailure {
    /// Selector that caused this attempt, or `None` for automatic selection.
    requested_selector: Option<ProviderSelector>,
    /// Canonical provider reached by lookup, or `None` when lookup failed.
    provider_id: Option<ProviderId>,
    /// Provider-reported classification, or `None` when creation was not run.
    provider_error_kind: Option<ProviderErrorKind>,
    /// Human-readable explanation of the lookup or creation failure.
    reason: Box<str>,
    /// Optional underlying provider cause retained for error chaining.
    source: Option<Arc<dyn Error + Send + Sync>>,
}

impl AttemptFailure {
    /// Creates a failed attempt for a selector that matched no provider.
    ///
    /// `selector` is retained as the requested selector and used to construct the
    /// reason. Returns an attempt without a provider ID, provider error kind, or
    /// source because service creation was not reached.
    #[must_use]
    pub fn unknown_provider(selector: ProviderSelector) -> Self {
        Self {
            reason: format!("unknown provider: {}", selector).into(),
            requested_selector: Some(selector),
            provider_id: None,
            provider_error_kind: None,
            source: None,
        }
    }

    /// Creates a failed attempt from an error returned by a provider factory.
    ///
    /// `requested_selector` identifies the explicit selector, or is `None` for
    /// automatic selection; `provider_id` identifies the reached provider; and
    /// `error` supplies the classification, reason, and optional source. Returns a
    /// diagnostic record retaining those values.
    #[must_use]
    pub fn provider_error(
        requested_selector: Option<ProviderSelector>,
        provider_id: ProviderId,
        error: &ProviderError,
    ) -> Self {
        Self {
            requested_selector,
            provider_id: Some(provider_id),
            provider_error_kind: Some(error.kind()),
            reason: error.reason().into(),
            source: error.source_arc(),
        }
    }

    /// Returns the selector that requested this attempt.
    ///
    /// Returns `Some` for named and chained selection attempts and `None` for an
    /// automatically selected candidate.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        self.requested_selector.as_ref()
    }

    /// Returns the canonical provider reached by selector lookup.
    ///
    /// Returns `Some` when service creation was attempted and `None` when lookup
    /// failed before a provider was identified.
    #[must_use]
    pub fn provider_id(&self) -> Option<&ProviderId> {
        self.provider_id.as_ref()
    }

    /// Returns the provider-reported creation failure classification.
    ///
    /// Returns `Some` when a provider factory returned an error and `None` when
    /// service creation was not attempted.
    #[must_use]
    pub const fn provider_error_kind(&self) -> Option<ProviderErrorKind> {
        self.provider_error_kind
    }

    /// Returns the human-readable explanation recorded for this failed attempt.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the underlying cause retained from the provider error.
    ///
    /// Returns `Some` when the provider supplied a source error and `None` for an
    /// unknown selector or a provider error without a source.
    #[must_use]
    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

/// Aggregate error returned when a provider selection cannot create a service.
///
/// Direct lookup failures retain the requested selector when it can be parsed.
/// Failed named, automatic, and chained resolution retains ordered
/// [`AttemptFailure`] records for unknown selectors and provider errors observed
/// before resolution stopped. Duplicate selectors for an already attempted
/// provider and candidates not reached after an early stop are not recorded.
#[derive(Clone, Debug)]
pub struct ResolutionError {
    /// Classification of the overall resolution failure.
    kind: ResolutionErrorKind,
    /// Normalized direct-lookup selector, when one could be retained.
    requested_selector: Option<ProviderSelector>,
    /// Ordered failures recorded while candidates were considered.
    attempts: Box<[AttemptFailure]>,
}

impl ResolutionError {
    /// Creates an error for a direct selector that matched no provider.
    ///
    /// `selector` is normalized and retained when it is valid. Returns an
    /// [`ResolutionErrorKind::UnknownProvider`] error with no attempt records; an
    /// invalid selector is represented by `None` in [`Self::requested_selector`].
    #[must_use]
    pub fn unknown_provider(selector: impl AsRef<str>) -> Self {
        Self {
            kind: ResolutionErrorKind::UnknownProvider,
            requested_selector: ProviderSelector::parse(selector).ok(),
            attempts: Box::new([]),
        }
    }

    /// Creates an aggregate error when a selection produces no service.
    ///
    /// `attempts` contains the unknown selectors and provider creation failures
    /// recorded before resolution stopped, in encounter order. It may be empty
    /// when no candidate exists. Returns a
    /// [`ResolutionErrorKind::NoProviderSucceeded`] error without a direct
    /// requested selector.
    #[must_use]
    pub fn no_provider_succeeded(attempts: impl Into<Box<[AttemptFailure]>>) -> Self {
        Self {
            kind: ResolutionErrorKind::NoProviderSucceeded,
            requested_selector: None,
            attempts: attempts.into(),
        }
    }

    /// Returns the overall resolution failure classification.
    #[must_use]
    pub const fn kind(&self) -> ResolutionErrorKind {
        self.kind
    }

    /// Returns the normalized selector retained for a direct lookup failure.
    ///
    /// Returns `Some` for a valid unknown selector and `None` for invalid input or
    /// an aggregate candidate failure.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        self.requested_selector.as_ref()
    }

    /// Returns failed attempts in the order candidates were considered.
    ///
    /// The slice contains only unknown selectors and actual provider failures;
    /// duplicate selectors and candidates not reached after an early stop are
    /// omitted. It is empty for a direct unknown-provider error or when no
    /// candidate exists.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFailure] {
        &self.attempts
    }
}

impl fmt::Display for ResolutionError {
    /// Formats the overall resolution failure for human-readable diagnostics.
    ///
    /// `formatter` receives either the unknown selector or the number of recorded
    /// attempts. Returns a formatting error if the formatter rejects the message.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ResolutionErrorKind::UnknownProvider => write!(
                formatter,
                "unknown provider: {}",
                self.requested_selector
                    .as_ref()
                    .map_or("<invalid>", ProviderSelector::as_str),
            ),
            ResolutionErrorKind::NoProviderSucceeded => {
                write!(
                    formatter,
                    "no provider succeeded after {} attempt(s)",
                    self.attempts.len()
                )
            }
        }
    }
}

impl Error for ResolutionError {}
