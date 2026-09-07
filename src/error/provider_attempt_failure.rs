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
///
/// # Type Parameters
///
/// * `E` - Domain error declared by the service specification.
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, SyncServiceSpec};
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = String;
///     type Error = std::io::Error;
/// }
/// impl SyncServiceSpec for Spec { type Output = String; }
/// use qubit_spi::{ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider};
/// use qubit_spi::error::ProviderFailure;
/// struct Echo;
/// impl ProviderMetadata for Echo {
///     fn descriptor(&self) -> ProviderDescriptor {
///         ProviderDescriptor::new(ProviderId::new("echo").expect("valid static ID"))
///     }
/// }
/// impl ServiceProvider<Spec> for Echo {
///     fn create_configured(&self, config: &String) -> Result<String, ProviderFailure<std::io::Error>> {
///         Err(ProviderFailure::unavailable(std::io::Error::other(config.clone())))
///     }
/// }
/// let registry = qubit_spi::ProviderRegistry::<Spec>::default();
/// registry.register(Echo)?;
/// let error = registry.resolve()?.create_configured(&"offline".to_owned()).expect_err("provider is unavailable");
/// let attempt = &error.attempts()[0];
/// assert_eq!("echo", attempt.provider_id().as_str());
/// assert_eq!("offline", attempt.failure().error().to_string());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
#[must_use]
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
    pub(crate) fn new(provider_id: ProviderId, failure: ProviderFailure<E>) -> Self {
        Self { provider_id, failure }
    }

    /// Returns the canonical ID of the attempted provider.
    ///
    /// # Returns
    ///
    /// The provider identity captured before creation was attempted.
    #[inline(always)]
    #[must_use = "the retained provider failure carries diagnostic context"]
    pub const fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the provider's original typed failure.
    ///
    /// # Returns
    ///
    /// The retained leaf provider failure.
    #[inline(always)]
    #[must_use = "the provider ID and failure preserve the attempted operation"]
    pub const fn failure(&self) -> &ProviderFailure<E> {
        &self.failure
    }

    /// Transfers ownership of the provider identity and failure.
    ///
    /// # Returns
    ///
    /// The provider ID captured before invocation and its typed failure.
    #[inline(always)]
    #[must_use = "the provider ID and failure preserve the attempted operation"]
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
