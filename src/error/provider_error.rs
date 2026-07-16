// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classified errors returned by provider construction.

use std::{
    error::Error,
    sync::Arc,
};

use thiserror::Error;

use super::ProviderErrorKind;

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
    /// # Arguments
    ///
    /// * `reason` - Unsupported capability or configuration explanation.
    ///
    /// # Returns
    ///
    /// An error classified as [`ProviderErrorKind::Unsupported`].
    #[inline(always)]
    #[must_use]
    pub fn unsupported(reason: impl Into<Box<str>>) -> Self {
        Self::new(ProviderErrorKind::Unsupported, reason)
    }

    /// Creates an unsupported-request error with an underlying cause.
    ///
    /// # Arguments
    ///
    /// * `reason` - Human-readable unsupported condition.
    /// * `source` - Causal error retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An unsupported error retaining its source.
    #[inline(always)]
    #[must_use]
    pub fn unsupported_with_source(
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(ProviderErrorKind::Unsupported, reason, source)
    }

    /// Creates an error indicating that the provider is unavailable.
    ///
    /// # Arguments
    ///
    /// * `reason` - Unavailable dependency or environment explanation.
    ///
    /// # Returns
    ///
    /// An error classified as [`ProviderErrorKind::Unavailable`].
    #[inline(always)]
    #[must_use]
    pub fn unavailable(reason: impl Into<Box<str>>) -> Self {
        Self::new(ProviderErrorKind::Unavailable, reason)
    }

    /// Creates an unavailable-provider error with an underlying cause.
    ///
    /// # Arguments
    ///
    /// * `reason` - Human-readable unavailable condition.
    /// * `source` - Causal error retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An unavailable error retaining its source.
    #[inline(always)]
    #[must_use]
    pub fn unavailable_with_source(
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(ProviderErrorKind::Unavailable, reason, source)
    }

    /// Creates an error for provider-specific invalid configuration.
    ///
    /// # Arguments
    ///
    /// * `reason` - Explanation identifying the invalid setting.
    ///
    /// # Returns
    ///
    /// An error classified as [`ProviderErrorKind::InvalidConfiguration`].
    #[inline(always)]
    #[must_use]
    pub fn invalid_configuration(reason: impl Into<Box<str>>) -> Self {
        Self::new(ProviderErrorKind::InvalidConfiguration, reason)
    }

    /// Creates an invalid-configuration error with an underlying cause.
    ///
    /// # Arguments
    ///
    /// * `reason` - Human-readable invalid configuration condition.
    /// * `source` - Causal error retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An invalid-configuration error retaining its source.
    #[inline(always)]
    #[must_use]
    pub fn invalid_configuration_with_source(
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(
            ProviderErrorKind::InvalidConfiguration,
            reason,
            source,
        )
    }

    /// Creates an unexpected provider-initialization failure.
    ///
    /// # Arguments
    ///
    /// * `reason` - Explanation of the initialization failure.
    ///
    /// # Returns
    ///
    /// An error classified as [`ProviderErrorKind::InitializationFailed`].
    #[inline(always)]
    #[must_use]
    pub fn initialization_failed(reason: impl Into<Box<str>>) -> Self {
        Self::new(ProviderErrorKind::InitializationFailed, reason)
    }

    /// Creates an initialization failure with an underlying cause.
    ///
    /// # Arguments
    ///
    /// * `reason` - Human-readable initialization failure.
    /// * `source` - Causal error retained for diagnostics.
    ///
    /// # Returns
    ///
    /// An initialization failure retaining its source.
    #[inline(always)]
    #[must_use]
    pub fn initialization_failed_with_source(
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(
            ProviderErrorKind::InitializationFailed,
            reason,
            source,
        )
    }

    /// Creates a classified error without an underlying source.
    ///
    /// # Arguments
    ///
    /// * `kind` - Classification controlling resolver fallback behavior.
    /// * `reason` - Human-readable diagnostic message.
    ///
    /// # Returns
    ///
    /// The classified provider error.
    #[inline]
    fn new(kind: ProviderErrorKind, reason: impl Into<Box<str>>) -> Self {
        Self {
            kind,
            reason: reason.into(),
            source: None,
        }
    }

    /// Creates a classified error retaining an underlying source.
    ///
    /// # Arguments
    ///
    /// * `kind` - Classification controlling resolver fallback behavior.
    /// * `reason` - Human-readable diagnostic message.
    /// * `source` - Causal error retained for error chaining.
    ///
    /// # Returns
    ///
    /// The classified provider error with its source.
    #[inline]
    fn with_source(
        kind: ProviderErrorKind,
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            reason: reason.into(),
            source: Some(Arc::new(source)),
        }
    }

    /// Returns the failure classification used by fallback policy evaluation.
    ///
    /// # Returns
    ///
    /// The provider-reported failure kind.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderErrorKind {
        self.kind
    }

    /// Returns the provider-supplied explanation of the failure.
    ///
    /// # Returns
    ///
    /// The human-readable provider diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}
