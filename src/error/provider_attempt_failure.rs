// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Diagnostics for one attempted provider creation.

use std::{
    error::Error,
    fmt,
};

use crate::ProviderId;

use super::ProviderError;

/// Diagnostic record for one provider that failed to create a service.
#[derive(Clone, Debug)]
pub struct ProviderAttemptFailure {
    /// Canonical identifier of the provider that was invoked.
    provider_id: ProviderId,
    /// Original provider failure retained with its causal source.
    error: ProviderError,
}

impl ProviderAttemptFailure {
    /// Creates a diagnostic from an actual provider invocation failure.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - Canonical ID of the provider that was invoked.
    /// * `error` - Original provider error transferred into the diagnostic.
    ///
    /// # Returns
    ///
    /// A provider attempt retaining its identity and causal error.
    #[inline]
    #[must_use]
    pub(crate) fn new(provider_id: ProviderId, error: ProviderError) -> Self {
        Self { provider_id, error }
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

    /// Returns the provider's original classified failure.
    ///
    /// # Returns
    ///
    /// The retained leaf provider error.
    #[inline(always)]
    #[must_use]
    pub const fn error(&self) -> &ProviderError {
        &self.error
    }
}

impl fmt::Display for ProviderAttemptFailure {
    /// Formats the failure with canonical provider context.
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
        write!(
            formatter,
            "provider {} failed with {:?}: {}",
            self.provider_id,
            self.error.kind(),
            self.error.reason(),
        )
    }
}

impl Error for ProviderAttemptFailure {
    /// Returns the retained provider error.
    ///
    /// # Returns
    ///
    /// The provider's original classified failure.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}
