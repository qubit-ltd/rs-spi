// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Diagnostics for individual provider resolution attempts.

use std::{
    error::Error,
    fmt,
};

use crate::{
    ProviderId,
    ProviderSelector,
};

use super::AttemptFailureKind;
use super::ProviderError;

/// Diagnostic record for one candidate that could not produce a service.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AttemptFailure {
    /// Selector lookup reached no provider.
    UnknownProvider {
        /// Normalized selector retained from the request.
        requested_selector: ProviderSelector,
    },
    /// A provider factory returned a classified error.
    ProviderError {
        /// Explicit selector, or `None` for automatic selection.
        requested_selector: Option<ProviderSelector>,
        /// Canonical provider reached by lookup.
        provider_id: ProviderId,
        /// Original provider error retained with its causal source.
        error: ProviderError,
    },
}

impl AttemptFailure {
    /// Creates a failed attempt for a selector that matched no provider.
    ///
    /// # Arguments
    ///
    /// * `selector` - Normalized selector that reached no registry entry.
    ///
    /// # Returns
    ///
    /// A resolver-owned unknown-provider attempt.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_provider(selector: ProviderSelector) -> Self {
        Self::UnknownProvider {
            requested_selector: selector,
        }
    }

    /// Creates a failed attempt from an error returned by a provider factory.
    ///
    /// # Arguments
    ///
    /// * `requested_selector` - Explicit selector, or `None` for automatic
    ///   selection.
    /// * `provider_id` - Canonical ID of the provider that was invoked.
    /// * `error` - Original provider error transferred into the attempt.
    ///
    /// # Returns
    ///
    /// A resolver-owned provider failure retaining the original error.
    #[inline]
    #[must_use]
    pub(crate) fn from_provider_error(
        requested_selector: Option<ProviderSelector>,
        provider_id: ProviderId,
        error: ProviderError,
    ) -> Self {
        Self::ProviderError {
            requested_selector,
            provider_id,
            error,
        }
    }

    /// Returns this attempt's stable classification.
    ///
    /// # Returns
    ///
    /// The lookup or provider-error classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> AttemptFailureKind {
        match self {
            Self::UnknownProvider { .. } => AttemptFailureKind::UnknownProvider,
            Self::ProviderError { .. } => AttemptFailureKind::ProviderError,
        }
    }

    /// Returns the selector requested for this attempt.
    ///
    /// # Returns
    ///
    /// The explicit selector for lookup and named/chained provider attempts,
    /// or `None` for automatic provider attempts.
    #[inline(always)]
    #[must_use]
    pub const fn requested_selector(&self) -> Option<&ProviderSelector> {
        match self {
            Self::UnknownProvider { requested_selector }
            | Self::ProviderError {
                requested_selector: Some(requested_selector),
                ..
            } => Some(requested_selector),
            Self::ProviderError {
                requested_selector: None,
                ..
            } => None,
        }
    }

    /// Returns the provider reached by this attempt.
    ///
    /// # Returns
    ///
    /// The canonical provider ID for provider failures, or `None` when lookup
    /// reached no provider.
    #[inline(always)]
    #[must_use]
    pub const fn provider_id(&self) -> Option<&ProviderId> {
        match self {
            Self::ProviderError { provider_id, .. } => Some(provider_id),
            Self::UnknownProvider { .. } => None,
        }
    }

    /// Returns the error reported by the attempted provider.
    ///
    /// # Returns
    ///
    /// The classified provider error, or `None` when lookup reached no
    /// provider.
    #[inline(always)]
    #[must_use]
    pub const fn provider_error(&self) -> Option<&ProviderError> {
        match self {
            Self::ProviderError { error, .. } => Some(error),
            Self::UnknownProvider { .. } => None,
        }
    }
}

impl fmt::Display for AttemptFailure {
    /// Formats this failed attempt with selector or provider context.
    ///
    /// # Arguments
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects diagnostic output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProvider { requested_selector } => {
                write!(formatter, "unknown provider: {requested_selector}")
            }
            Self::ProviderError {
                requested_selector,
                provider_id,
                error,
            } => match requested_selector {
                Some(selector) => write!(
                    formatter,
                    "provider {provider_id} failed with {:?}: {} (requested as {selector})",
                    error.kind(),
                    error.reason(),
                ),
                None => write!(
                    formatter,
                    "provider {provider_id} failed with {:?}: {}",
                    error.kind(),
                    error.reason(),
                ),
            },
        }
    }
}

impl Error for AttemptFailure {
    /// Returns the retained provider error, when one exists.
    ///
    /// # Returns
    ///
    /// The provider error for an invoked provider, or `None` for failed lookup.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnknownProvider { .. } => None,
            Self::ProviderError { error, .. } => Some(error),
        }
    }
}
