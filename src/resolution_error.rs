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

/// Classification of a failed provider resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// A named selector did not resolve to any registered provider.
    UnknownProvider,
    /// Every candidate failed or was skipped.
    NoProviderSucceeded,
}

/// One failed resolution attempt.
#[derive(Clone, Debug)]
pub struct AttemptFailure {
    requested_selector: Option<ProviderSelector>,
    provider_id: Option<ProviderId>,
    provider_error_kind: Option<ProviderErrorKind>,
    reason: Box<str>,
    source: Option<Arc<dyn Error + Send + Sync>>,
}

impl AttemptFailure {
    /// Creates an attempt failure for an unknown selector.
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

    /// Creates an attempt failure from a provider error.
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

    /// Gets the selector that requested this attempt.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        self.requested_selector.as_ref()
    }

    /// Gets the canonical provider ID when lookup succeeded.
    #[must_use]
    pub fn provider_id(&self) -> Option<&ProviderId> {
        self.provider_id.as_ref()
    }

    /// Gets the provider error kind when creation was attempted.
    #[must_use]
    pub const fn provider_error_kind(&self) -> Option<ProviderErrorKind> {
        self.provider_error_kind
    }

    /// Gets the failure reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Gets the underlying provider error when one is available.
    #[must_use]
    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

/// Aggregate error raised while resolving a provider selection.
#[derive(Clone, Debug)]
pub struct ResolutionError {
    kind: ResolutionErrorKind,
    requested_selector: Option<ProviderSelector>,
    attempts: Box<[AttemptFailure]>,
}

impl ResolutionError {
    /// Creates an unknown-provider error.
    #[must_use]
    pub fn unknown_provider(selector: impl AsRef<str>) -> Self {
        Self {
            kind: ResolutionErrorKind::UnknownProvider,
            requested_selector: ProviderSelector::parse(selector).ok(),
            attempts: Box::new([]),
        }
    }

    /// Creates an aggregate failure after provider attempts.
    #[must_use]
    pub fn no_provider_succeeded(attempts: impl Into<Box<[AttemptFailure]>>) -> Self {
        Self {
            kind: ResolutionErrorKind::NoProviderSucceeded,
            requested_selector: None,
            attempts: attempts.into(),
        }
    }

    /// Gets the error classification.
    #[must_use]
    pub const fn kind(&self) -> ResolutionErrorKind {
        self.kind
    }

    /// Gets the named selector when the error came from direct lookup.
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        self.requested_selector.as_ref()
    }

    /// Gets the recorded failed attempts.
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFailure] {
        &self.attempts
    }
}

impl fmt::Display for ResolutionError {
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
