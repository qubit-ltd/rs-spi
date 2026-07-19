// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Composing service provider built from a resolved candidate snapshot.

use std::fmt;

use crate::error::ProviderCreationError;
use crate::internal::{
    FallbackState,
    RegistryEntry,
};
use crate::{
    FallbackPolicy,
    ProviderDefinition,
    SyncServiceSpec,
};

/// Service provider that tries a point-in-time snapshot of registry candidates.
///
/// Registry resolution fixes candidate identity and order. Service creation is
/// a separate operation that supplies configuration and applies the selection's
/// fallback policy. Successful creation returns the service output directly;
/// failures from later operations on that output do not re-enter fallback.
pub struct ResolvingServiceProvider<S>
where
    S: SyncServiceSpec,
{
    /// Nonempty provider candidates in deterministic attempt order.
    candidates: Box<[RegistryEntry<dyn ProviderDefinition<S>>]>,
    /// Policy deciding which leaf failures permit another attempt.
    fallback_policy: FallbackPolicy,
}

impl<S> ResolvingServiceProvider<S>
where
    S: SyncServiceSpec,
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
        candidates: Box<[RegistryEntry<dyn ProviderDefinition<S>>]>,
        fallback_policy: FallbackPolicy,
    ) -> Self {
        assert!(
            !candidates.is_empty(),
            "resolving service providers require at least one candidate",
        );
        Self {
            candidates,
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
        let mut fallback = FallbackState::new(self.fallback_policy);
        let candidate_count = self.candidates.len();
        for (index, candidate) in self.candidates.iter().enumerate() {
            match candidate.provider.create_configured(config) {
                Ok(service) => return Ok(service),
                Err(error) => {
                    let has_remaining = index + 1 < candidate_count;
                    if let Some(error) = fallback.record_failure(
                        candidate.descriptor.id().clone(),
                        error,
                        has_remaining,
                    ) {
                        return Err(error);
                    }
                }
            }
        }
        unreachable!("resolving providers always contain a candidate")
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
        self.create_configured(&S::Config::default())
    }
}

impl<S> Clone for ResolvingServiceProvider<S>
where
    S: SyncServiceSpec,
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
    S: SyncServiceSpec,
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
