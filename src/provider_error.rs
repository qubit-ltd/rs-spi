// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classified errors returned by provider construction.

use std::{error::Error, fmt, sync::Arc};

/// Classification of a provider creation failure.
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
#[derive(Clone, Debug)]
pub struct ProviderError {
    kind: ProviderErrorKind,
    reason: Box<str>,
    source: Option<Arc<dyn Error + Send + Sync>>,
}

impl ProviderError {
    /// Creates an unsupported-request error.
    #[must_use]
    pub fn unsupported(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::Unsupported, reason)
    }

    /// Creates an unavailable-provider error.
    #[must_use]
    pub fn unavailable(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::Unavailable, reason)
    }

    /// Creates an unavailable-provider error with a source.
    #[must_use]
    pub fn unavailable_with_source(
        reason: impl AsRef<str>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(ProviderErrorKind::Unavailable, reason, source)
    }

    /// Creates an invalid-provider-configuration error.
    #[must_use]
    pub fn invalid_configuration(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::InvalidConfiguration, reason)
    }

    /// Creates a provider-initialization failure.
    #[must_use]
    pub fn initialization_failed(reason: impl AsRef<str>) -> Self {
        Self::new(ProviderErrorKind::InitializationFailed, reason)
    }

    /// Creates a provider-initialization failure with a source.
    #[must_use]
    pub fn initialization_failed_with_source(
        reason: impl AsRef<str>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::with_source(ProviderErrorKind::InitializationFailed, reason, source)
    }

    /// Gets the failure classification.
    #[must_use]
    pub const fn kind(&self) -> ProviderErrorKind {
        self.kind
    }

    /// Gets the provider-supplied reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub(crate) fn source_arc(&self) -> Option<Arc<dyn Error + Send + Sync>> {
        self.source.clone()
    }

    fn new(kind: ProviderErrorKind, reason: impl AsRef<str>) -> Self {
        Self {
            kind,
            reason: reason.as_ref().into(),
            source: None,
        }
    }

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

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "provider {:?}: {}", self.kind, self.reason)
    }
}

impl Error for ProviderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}
