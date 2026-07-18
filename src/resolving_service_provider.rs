// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Composing service provider built from a resolved candidate snapshot.

use std::fmt;

use crate::error::{
    ProviderAttemptFailure,
    ProviderCreationError,
    ProviderError,
    ProviderErrorKind,
};
use crate::internal::RegistryEntry;
use crate::{
    FallbackPolicy,
    ServiceProvider,
    ServiceSpec,
};

/// Service provider that tries a point-in-time snapshot of registry candidates.
///
/// Registry resolution fixes candidate identity and order. Service creation is
/// a separate operation that supplies configuration and applies the selection's
/// fallback policy. Successful creation returns the service output directly;
/// failures from later operations on that output do not re-enter fallback.
pub struct ResolvingServiceProvider<S>
where
    S: ServiceSpec,
{
    /// Nonempty provider candidates in deterministic attempt order.
    candidates: Box<[RegistryEntry<S>]>,
    /// Policy deciding which leaf failures permit another attempt.
    fallback_policy: FallbackPolicy,
}

impl<S> ResolvingServiceProvider<S>
where
    S: ServiceSpec,
{
    /// Creates a composing provider from resolved candidates.
    ///
    /// # Parameters
    ///
    /// * `candidates` - Nonempty provider snapshots in attempt order.
    /// * `fallback_policy` - Policy controlling fallback after leaf failures.
    ///
    /// # Returns
    ///
    /// A composing provider owning the candidate snapshot.
    ///
    /// # Panics
    ///
    /// Panics when `candidates` is empty. Registry selection rejects empty
    /// candidate sets before calling this constructor.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        candidates: Vec<RegistryEntry<S>>,
        fallback_policy: FallbackPolicy,
    ) -> Self {
        assert!(
            !candidates.is_empty(),
            "resolving service providers require at least one candidate",
        );
        Self {
            candidates: candidates.into_boxed_slice(),
            fallback_policy,
        }
    }

    /// Creates a service output from the supplied configuration.
    ///
    /// # Parameters
    ///
    /// * `config` - Service configuration forwarded to resolved candidates.
    ///
    /// # Returns
    ///
    /// The first service output created successfully.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderCreationError`] when every candidate fails or the
    /// fallback policy stops traversal.
    #[inline(always)]
    pub fn create_configured(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderCreationError> {
        <Self as ServiceProvider<S>>::create_configured(self, config)
    }

    /// Creates a service output with the default configuration.
    ///
    /// # Returns
    ///
    /// The first service output created successfully.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderCreationError`] when every candidate fails or the
    /// fallback policy stops traversal.
    #[inline(always)]
    pub fn create(&self) -> Result<S::Output, ProviderCreationError>
    where
        S::Config: Default,
    {
        <Self as ServiceProvider<S>>::create(self)
    }

    /// Reports whether fallback may continue after one leaf error kind.
    ///
    /// # Parameters
    ///
    /// * `kind` - Provider-reported leaf failure classification.
    ///
    /// # Returns
    ///
    /// `true` when this provider's fallback policy permits another attempt.
    #[inline]
    #[must_use]
    fn allows_fallback(&self, kind: ProviderErrorKind) -> bool {
        match self.fallback_policy {
            FallbackPolicy::Never => false,
            FallbackPolicy::OnAbsence => matches!(
                kind,
                ProviderErrorKind::Unsupported | ProviderErrorKind::Unavailable
            ),
            FallbackPolicy::OnAnyError => true,
        }
    }
}

impl<S> ServiceProvider<S> for ResolvingServiceProvider<S>
where
    S: ServiceSpec,
{
    /// Tries resolved candidates in order with the supplied configuration.
    ///
    /// # Parameters
    ///
    /// * `config` - Service configuration forwarded unchanged to each attempt.
    ///
    /// # Returns
    ///
    /// The first service output created successfully.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderCreationError`] containing ordered actual provider
    /// failures when all candidates fail or fallback policy stops traversal.
    /// A nested aggregate from a registered provider is classified as an
    /// initialization failure and always stops traversal.
    fn create_configured(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderCreationError> {
        let mut failures = Vec::new();
        for candidate in &self.candidates {
            match candidate.provider.create_configured(config) {
                Ok(service) => {
                    // TODO: When `failures` is non-empty, publish an internal
                    // fallback observation through IoC-injected collector and
                    // processor components before returning. Observability
                    // remains an internal library concern.
                    return Ok(service);
                }
                Err(ProviderCreationError::Provider(error)) => {
                    let kind = error.kind();
                    failures.push(ProviderAttemptFailure::new(
                        candidate.descriptor.id().clone(),
                        error,
                    ));
                    if !self.allows_fallback(kind) {
                        return Err(ProviderCreationError::stopped_by_policy(
                            failures,
                        ));
                    }
                }
                Err(
                    aggregate @ ProviderCreationError::NoProviderSucceeded {
                        ..
                    },
                ) => {
                    let error =
                        ProviderError::initialization_failed_with_source(
                            "registered provider returned an aggregate creation error",
                            aggregate,
                        );
                    failures.push(ProviderAttemptFailure::new(
                        candidate.descriptor.id().clone(),
                        error,
                    ));
                    return Err(ProviderCreationError::stopped_by_policy(
                        failures,
                    ));
                }
            }
        }
        Err(ProviderCreationError::exhausted(failures))
    }
}

impl<S> Clone for ResolvingServiceProvider<S>
where
    S: ServiceSpec,
{
    /// Clones the candidate snapshot and shared provider handles.
    ///
    /// # Returns
    ///
    /// An independent composing provider with the same candidates and policy.
    fn clone(&self) -> Self {
        Self {
            candidates: self.candidates.to_vec().into_boxed_slice(),
            fallback_policy: self.fallback_policy,
        }
    }
}

impl<S> fmt::Debug for ResolvingServiceProvider<S>
where
    S: ServiceSpec,
{
    /// Formats candidate descriptors and fallback policy.
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
    /// Returns [`fmt::Error`] when the formatter rejects debug output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvingServiceProvider")
            .field(
                "candidates",
                &self
                    .candidates
                    .iter()
                    .map(|candidate| &candidate.descriptor)
                    .collect::<Vec<_>>(),
            )
            .field("fallback_policy", &self.fallback_policy)
            .finish()
    }
}
