// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Typed domain errors and failure constructors shared by SPI tests.

use std::{
    error::Error,
    fmt,
    sync::Arc,
};

use qubit_spi::error::{
    ProviderFailure,
    ProviderFailureKind,
};

/// Cloneable domain error used by reusable provider fixtures.
#[derive(Clone, Debug)]
pub(crate) struct TestError {
    /// Human-readable test diagnostic.
    reason: Box<str>,
    /// Optional source retained for source-chain assertions.
    source: Option<Arc<dyn Error + Send + Sync>>,
}

impl TestError {
    /// Returns the test diagnostic text.
    ///
    /// # Returns
    ///
    /// The human-readable reason supplied by the fixture.
    #[must_use]
    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for TestError {
    /// Formats the test diagnostic.
    ///
    /// # Parameters
    ///
    /// * formatter - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl Error for TestError {
    /// Returns the optional fixture source.
    ///
    /// # Returns
    ///
    /// The causal error configured by the fixture, when present.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

/// Test-only constructors for typed provider failures.
pub(crate) struct TestProviderFailure;

impl TestProviderFailure {
    /// Creates a typed failure retaining an underlying source.
    ///
    /// # Parameters
    ///
    /// * kind - Fallback classification for the fixture.
    /// * reason - Diagnostic text retained by the test domain error.
    /// * source - Causal source used by source-chain assertions.
    ///
    /// # Returns
    ///
    /// A typed provider failure with its configured domain source.
    #[must_use]
    pub(crate) fn with_source(
        kind: ProviderFailureKind,
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> ProviderFailure<TestError> {
        Self::with_optional_source(kind, reason.into(), Some(Arc::new(source)))
    }

    /// Creates an unsupported typed failure.
    ///
    /// # Parameters
    ///
    /// * reason - Diagnostic text retained by the test domain error.
    ///
    /// # Returns
    ///
    /// A failure classified as unsupported.
    #[must_use]
    pub(crate) fn unsupported(
        reason: impl Into<Box<str>>,
    ) -> ProviderFailure<TestError> {
        ProviderFailure::unsupported(TestError {
            reason: reason.into(),
            source: None,
        })
    }

    /// Creates an unavailable typed failure.
    ///
    /// # Parameters
    ///
    /// * reason - Diagnostic text retained by the test domain error.
    ///
    /// # Returns
    ///
    /// A failure classified as unavailable.
    #[must_use]
    pub(crate) fn unavailable(
        reason: impl Into<Box<str>>,
    ) -> ProviderFailure<TestError> {
        ProviderFailure::unavailable(TestError {
            reason: reason.into(),
            source: None,
        })
    }

    /// Creates an invalid-configuration typed failure.
    ///
    /// # Parameters
    ///
    /// * reason - Diagnostic text retained by the test domain error.
    ///
    /// # Returns
    ///
    /// A failure classified as invalid configuration.
    #[must_use]
    pub(crate) fn invalid_configuration(
        reason: impl Into<Box<str>>,
    ) -> ProviderFailure<TestError> {
        ProviderFailure::invalid_configuration(TestError {
            reason: reason.into(),
            source: None,
        })
    }

    /// Creates an initialization typed failure.
    ///
    /// # Parameters
    ///
    /// * reason - Diagnostic text retained by the test domain error.
    ///
    /// # Returns
    ///
    /// A failure classified as initialization failed.
    #[must_use]
    pub(crate) fn initialization_failed(
        reason: impl Into<Box<str>>,
    ) -> ProviderFailure<TestError> {
        ProviderFailure::initialization_failed(TestError {
            reason: reason.into(),
            source: None,
        })
    }

    /// Creates an unavailable typed failure with a source.
    ///
    /// # Parameters
    ///
    /// * reason - Diagnostic text retained by the test domain error.
    /// * source - Causal source used by source-chain assertions.
    ///
    /// # Returns
    ///
    /// An unavailable failure retaining the configured source.
    #[must_use]
    pub(crate) fn unavailable_with_source(
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> ProviderFailure<TestError> {
        Self::with_source(ProviderFailureKind::Unavailable, reason, source)
    }

    /// Creates an initialization typed failure with a source.
    ///
    /// # Parameters
    ///
    /// * reason - Diagnostic text retained by the test domain error.
    /// * source - Causal source used by source-chain assertions.
    ///
    /// # Returns
    ///
    /// An initialization failure retaining the configured source.
    #[must_use]
    pub(crate) fn initialization_failed_with_source(
        reason: impl Into<Box<str>>,
        source: impl Error + Send + Sync + 'static,
    ) -> ProviderFailure<TestError> {
        Self::with_source(
            ProviderFailureKind::InitializationFailed,
            reason,
            source,
        )
    }

    /// Builds a typed failure with optional source storage.
    fn with_optional_source(
        kind: ProviderFailureKind,
        reason: Box<str>,
        source: Option<Arc<dyn Error + Send + Sync>>,
    ) -> ProviderFailure<TestError> {
        let error = TestError { reason, source };
        match kind {
            ProviderFailureKind::Unsupported => {
                ProviderFailure::unsupported(error)
            }
            ProviderFailureKind::Unavailable => {
                ProviderFailure::unavailable(error)
            }
            ProviderFailureKind::InvalidConfiguration => {
                ProviderFailure::invalid_configuration(error)
            }
            ProviderFailureKind::InitializationFailed => {
                ProviderFailure::initialization_failed(error)
            }
            _ => ProviderFailure::initialization_failed(error),
        }
    }
}
