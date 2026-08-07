// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Typed leaf failures returned by provider construction.

use std::error::Error;
use std::fmt;

use super::ProviderFailureKind;

/// Failure returned by one provider while creating a service.
///
/// E is the domain error declared by the service specification. The failure
/// kind controls fallback independently from the retained domain diagnostic.
#[derive(Clone, Debug)]
pub struct ProviderFailure<E> {
    /// Classification consumed by resolver fallback policy.
    kind: ProviderFailureKind,
    /// Domain error preserving the actual failure.
    error: E,
}

impl<E> ProviderFailure<E> {
    /// Creates an unsupported-request failure.
    ///
    /// # Parameters
    ///
    /// * error - Domain diagnostic describing the unsupported request.
    ///
    /// # Returns
    ///
    /// A failure classified as unsupported.
    #[inline(always)]
    #[must_use]
    pub const fn unsupported(error: E) -> Self {
        Self {
            kind: ProviderFailureKind::Unsupported,
            error,
        }
    }

    /// Creates an unavailable-provider failure.
    ///
    /// # Parameters
    ///
    /// * error - Domain diagnostic describing the unavailable dependency.
    ///
    /// # Returns
    ///
    /// A failure classified as unavailable.
    #[inline(always)]
    #[must_use]
    pub const fn unavailable(error: E) -> Self {
        Self {
            kind: ProviderFailureKind::Unavailable,
            error,
        }
    }

    /// Creates an invalid-provider-configuration failure.
    ///
    /// # Parameters
    ///
    /// * error - Domain diagnostic describing the invalid configuration.
    ///
    /// # Returns
    ///
    /// A failure classified as invalid configuration.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_configuration(error: E) -> Self {
        Self {
            kind: ProviderFailureKind::InvalidConfiguration,
            error,
        }
    }

    /// Creates an initialization failure after request acceptance.
    ///
    /// # Parameters
    ///
    /// * error - Domain diagnostic describing the initialization failure.
    ///
    /// # Returns
    ///
    /// A failure classified as initialization failed.
    #[inline(always)]
    #[must_use]
    pub const fn initialization_failed(error: E) -> Self {
        Self {
            kind: ProviderFailureKind::InitializationFailed,
            error,
        }
    }

    /// Returns the fallback classification.
    ///
    /// # Returns
    ///
    /// The classification consulted by resolver fallback policy.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderFailureKind {
        self.kind
    }

    /// Returns the retained domain error.
    ///
    /// # Returns
    ///
    /// A shared reference to the service-specific diagnostic.
    #[inline(always)]
    #[must_use]
    pub const fn error(&self) -> &E {
        &self.error
    }

    /// Transfers ownership of the retained domain error.
    ///
    /// # Returns
    ///
    /// The service-specific diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn into_error(self) -> E {
        self.error
    }

    /// Transfers ownership of the classification and domain error.
    ///
    /// # Returns
    ///
    /// The fallback classification and service-specific diagnostic.
    #[inline(always)]
    #[must_use]
    pub fn into_parts(self) -> (ProviderFailureKind, E) {
        (self.kind, self.error)
    }
}

impl<E> fmt::Display for ProviderFailure<E>
where
    E: fmt::Display,
{
    /// Formats fallback classification and the domain diagnostic.
    ///
    /// # Parameters
    ///
    /// * formatter - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns fmt::Error when the formatter rejects output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "provider {:?}: {}", self.kind, self.error)
    }
}

impl<E> Error for ProviderFailure<E>
where
    E: Error + 'static,
{
    /// Returns the retained domain error.
    ///
    /// # Returns
    ///
    /// The service-specific diagnostic supplied by the provider.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}
