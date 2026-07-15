// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classified errors returned by provider construction.

use std::{error::Error, sync::Arc};

use thiserror::Error;

/// Classification of a failure reported while a provider creates a service.
///
/// Providers return these variants so [`crate::ProviderResolver`] can decide
/// whether its fallback policy permits another provider to be tried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderErrorKind {
    /// The provider does not support this request or configuration.
    Unsupported,
    /// The provider cannot run in the current environment.
    Unavailable,
    /// The provider-specific configuration is invalid.
    InvalidConfiguration,
    /// Provider initialization failed unexpectedly.
    InitializationFailed,
}

/// Error returned by one provider while creating a service.
///
/// Use the constructors to report a classified failure from a
/// [`crate::ServiceProvider`] implementation. The resolver preserves this
/// information in its attempt diagnostics.
#[derive(Clone, Debug, Error)]
#[error("provider {kind:?}: {reason}")]
pub struct ProviderError {
    /// Classification consumed by the resolver's fallback policy.
    kind: ProviderErrorKind,
    /// Human-readable explanation supplied by the provider.
    reason: Box<str>,
    /// Optional underlying error retained for diagnostics and error chaining.
    #[source]
    source: Option<Arc<dyn Error + Send + Sync>>,
}

impl ProviderError {
    /// Creates an error indicating that the request is unsupported.
    ///
    /// `reason` explains the unsupported capability or configuration. Returns
    /// an error classified as [`ProviderErrorKind::Unsupported`].
    #[must_use]
    pub fn unsupported(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::Unsupported, reason)
    }

    /// Creates an error indicating that the provider is currently unavailable.
    ///
    /// `reason` explains the unavailable dependency or environment. Returns
    /// an error classified as [`ProviderErrorKind::Unavailable`].
    #[must_use]
    pub fn unavailable(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::Unavailable, reason)
    }

    /// Creates an unavailable-provider error with an underlying cause.
    ///
    /// `reason` describes the unavailable condition and `source` retains the
    /// causal error for diagnostics. Returns an error classified as
    /// [`ProviderErrorKind::Unavailable`].
    #[must_use]
    pub fn unavailable_with_source(
        reason: impl AsRef<str>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(ProviderErrorKind::Unavailable, reason, source)
    }

    /// Creates an error for provider-specific invalid configuration.
    ///
    /// `reason` identifies the invalid setting. Returns an error classified as
    /// [`ProviderErrorKind::InvalidConfiguration`].
    #[must_use]
    pub fn invalid_configuration(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::InvalidConfiguration, reason)
    }

    /// Creates an unexpected provider-initialization failure.
    ///
    /// `reason` describes the initialization failure. Returns an error
    /// classified as [`ProviderErrorKind::InitializationFailed`].
    #[must_use]
    pub fn initialization_failed(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::InitializationFailed, reason)
    }

    /// Creates an initialization failure with an underlying cause.
    ///
    /// `reason` describes the failure and `source` retains its causal error.
    /// Returns an error classified as [`ProviderErrorKind::InitializationFailed`].
    #[must_use]
    pub fn initialization_failed_with_source(
        reason: impl AsRef<str>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(ProviderErrorKind::InitializationFailed, reason, source)
    }

    /// Returns the failure classification used by fallback policy evaluation.
    #[must_use]
    pub const fn kind(&self) -> ProviderErrorKind {
        self.kind
    }

    /// Returns the provider-supplied explanation of the failure.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Clones the optional source error for internal diagnostic aggregation.
    ///
    /// Returns `Some` with a new [`Arc`] reference when a source was supplied,
    /// and `None` otherwise.
    pub(crate) fn source_arc(&self) -> Option<Arc<dyn Error + Send + Sync>> {
        self.source.clone()
    }

    /// Creates a classified error without an underlying source.
    ///
    /// `kind` controls fallback behavior and `reason` becomes the diagnostic
    /// message. Returns the constructed provider error.
    fn new(kind: ProviderErrorKind, reason: impl AsRef<str>) -> Self {
        Self {
            kind,
            reason: reason.as_ref().into(),
            source: None,
        }
    }

    /// Creates a classified error that retains an underlying source.
    ///
    /// `kind` controls fallback behavior, `reason` describes the failure, and
    /// `source` is stored for later diagnostic chaining. Returns the error.
    fn with_source(
        kind: ProviderErrorKind,
        reason: impl AsRef<str>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            reason: reason.as_ref().into(),
            source: Some(Arc::new(source)),
        }
    }
}
