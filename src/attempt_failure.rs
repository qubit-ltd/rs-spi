// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Diagnostics for individual provider resolution attempts.

use std::{error::Error, fmt};

use crate::internal::AttemptFailureRepr;
use crate::{
    AttemptFailureKind, ProviderError, ProviderErrorKind, ProviderId, ProviderSelector,
};

/// Diagnostic record for one candidate that could not produce a service.
#[derive(Clone, Debug)]
pub struct AttemptFailure {
    /// Variant-specific attempt diagnostics.
    repr: AttemptFailureRepr,
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
        Self {
            repr: AttemptFailureRepr::UnknownProvider {
                reason: format!("unknown provider: {selector}").into(),
                requested_selector: selector,
            },
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
    pub(crate) fn provider_error(
        requested_selector: Option<ProviderSelector>,
        provider_id: ProviderId,
        error: ProviderError,
    ) -> Self {
        Self {
            repr: AttemptFailureRepr::ProviderError {
                requested_selector,
                provider_id,
                error,
            },
        }
    }

    /// Returns the explicit attempt classification.
    ///
    /// # Returns
    ///
    /// The lookup or provider-error classification for this attempt.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> AttemptFailureKind {
        match self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => AttemptFailureKind::UnknownProvider,
            AttemptFailureRepr::ProviderError { .. } => AttemptFailureKind::ProviderError,
        }
    }

    /// Returns the selector that requested this attempt.
    ///
    /// # Returns
    ///
    /// `Some` for explicit lookup and chain attempts, or `None` for automatic
    /// provider selection.
    #[inline(always)]
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
    ///
    /// # Returns
    ///
    /// `Some` when a provider was invoked, or `None` when lookup failed.
    #[inline(always)]
    #[must_use]
    pub fn provider_id(&self) -> Option<&ProviderId> {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => None,
            AttemptFailureRepr::ProviderError { provider_id, .. } => Some(provider_id),
        }
    }

    /// Returns the provider-reported creation failure classification.
    ///
    /// # Returns
    ///
    /// `Some` for an invoked provider, or `None` for an unknown selector.
    #[inline(always)]
    #[must_use]
    pub const fn provider_error_kind(&self) -> Option<ProviderErrorKind> {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => None,
            AttemptFailureRepr::ProviderError { error, .. } => Some(error.kind()),
        }
    }

    /// Returns the human-readable explanation for this failed attempt.
    ///
    /// # Returns
    ///
    /// The lookup diagnostic or original provider-supplied reason.
    #[inline(always)]
    #[must_use]
    pub fn reason(&self) -> &str {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { reason, .. } => reason,
            AttemptFailureRepr::ProviderError { error, .. } => error.reason(),
        }
    }

    /// Returns the underlying cause retained from the provider error.
    ///
    /// # Returns
    ///
    /// `Some` with the provider's source error when one exists, or `None` for
    /// lookup failures and provider errors without a source.
    #[inline(always)]
    #[must_use]
    pub fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.repr {
            AttemptFailureRepr::UnknownProvider { .. } => None,
            AttemptFailureRepr::ProviderError { error, .. } => Error::source(error),
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
                error,
            } => {
                write!(
                    formatter,
                    "provider {provider_id} failed with {:?}: {}",
                    error.kind(),
                    error.reason(),
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
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Self::source(self)
    }
}
