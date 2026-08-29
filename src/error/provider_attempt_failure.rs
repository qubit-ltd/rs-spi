// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Diagnostics for one attempted provider creation.

use std::error::Error;
use std::fmt;

use super::ProviderFailure;
use crate::ProviderId;

/// Diagnostic record for one provider that failed to create a service.
#[derive(Clone, Debug)]
pub struct ProviderAttemptFailure<E> {
    /// Canonical identifier of the provider that was invoked.
    provider_id: ProviderId,
    /// Original provider failure retained with its causal source.
    failure: ProviderFailure<E>,
}

impl<E> ProviderAttemptFailure<E> {
    /// Creates a diagnostic from an actual provider invocation failure.
    ///
    /// # Parameters
    ///
    /// * `provider_id` - Canonical ID of the provider that was invoked.
    /// * failure - Original provider failure transferred into the diagnostic.
    ///
    /// # Returns
    ///
    /// A provider attempt retaining its identity and causal error.
    #[inline]
    #[must_use]
    pub(crate) fn new(provider_id: ProviderId, failure: ProviderFailure<E>) -> Self {
        Self { provider_id, failure }
    }

    /// Returns the canonical ID of the attempted provider.
    ///
    /// # Returns
    ///
    /// The provider identity captured before creation was attempted.
    #[inline(always)]
    #[must_use]
    pub const fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the provider's original typed failure.
    ///
    /// # Returns
    ///
    /// The retained leaf provider failure.
    #[inline(always)]
    #[must_use]
    pub const fn failure(&self) -> &ProviderFailure<E> {
        &self.failure
    }

    /// Transfers ownership of the provider identity and failure.
    ///
    /// # Returns
    ///
    /// The provider ID captured before invocation and its typed failure.
    #[inline(always)]
    #[must_use]
    pub fn into_parts(self) -> (ProviderId, ProviderFailure<E>) {
        (self.provider_id, self.failure)
    }
}

impl<E> fmt::Display for ProviderAttemptFailure<E>
where
    E: fmt::Display,
{
    /// Formats the failure with canonical provider context.
    ///
    /// # Parameters
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
        write!(formatter, "provider {} failed: {}", self.provider_id, self.failure,)
    }
}

impl<E> Error for ProviderAttemptFailure<E>
where
    E: Error + 'static,
{
    /// Returns the retained provider failure.
    ///
    /// # Returns
    ///
    /// The provider's original typed failure.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.failure)
    }
}
