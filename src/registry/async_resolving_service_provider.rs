// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous resolver built from a provider candidate snapshot.

use std::fmt;

use crate::error::ProviderCreationError;
use crate::registry::internal::{
    FallbackState,
    RegistryEntry,
};
use crate::{
    AsyncProviderDefinition,
    AsyncServiceSpec,
    FallbackPolicy,
    ProviderFuture,
};

/// Resolver that awaits a point-in-time snapshot of asynchronous candidates.
pub struct AsyncResolvingServiceProvider<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Nonempty provider candidates in deterministic attempt order.
    candidates: Box<[RegistryEntry<dyn AsyncProviderDefinition<S>>]>,
    /// Policy deciding which leaf failures permit another attempt.
    fallback_policy: FallbackPolicy,
}

impl<S> AsyncResolvingServiceProvider<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Creates an asynchronous resolver from resolved candidates.
    ///
    /// # Panics
    ///
    /// Panics when `candidates` is empty.
    #[must_use]
    pub(crate) fn new(
        candidates: Box<[RegistryEntry<dyn AsyncProviderDefinition<S>>]>,
        fallback_policy: FallbackPolicy,
    ) -> Self {
        assert!(
            !candidates.is_empty(),
            "asynchronous resolving providers require at least one candidate",
        );
        Self {
            candidates,
            fallback_policy,
        }
    }

    /// Creates a service output from the supplied configuration.
    ///
    /// No Registry lock is retained by the returned future.
    ///
    /// # Errors
    ///
    /// The future yields [`ProviderCreationError`] when all candidates fail or
    /// fallback policy stops traversal.
    pub fn create_configured<'a>(
        &'a self,
        config: &'a S::Config,
    ) -> ProviderFuture<'a, Result<S::Output, ProviderCreationError>> {
        Box::pin(async move {
            let mut fallback = FallbackState::new(self.fallback_policy);
            let candidate_count = self.candidates.len();
            for (index, candidate) in self.candidates.iter().enumerate() {
                match candidate.provider.create_configured(config).await {
                    Ok(output) => return Ok(output),
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
            unreachable!("resolved provider candidates are nonempty")
        })
    }

    /// Creates a service output with the default configuration.
    ///
    /// # Errors
    ///
    /// The future yields [`ProviderCreationError`] under the same conditions as
    /// [`Self::create_configured`].
    pub fn create(
        &self,
    ) -> ProviderFuture<'_, Result<S::Output, ProviderCreationError>>
    where
        S::Config: Default + Send,
    {
        Box::pin(async move {
            let config = S::Config::default();
            self.create_configured(&config).await
        })
    }
}

impl<S> Clone for AsyncResolvingServiceProvider<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Clones the candidate snapshot and shared provider handles.
    fn clone(&self) -> Self {
        Self {
            candidates: self.candidates.to_vec().into_boxed_slice(),
            fallback_policy: self.fallback_policy,
        }
    }
}

impl<S> fmt::Debug for AsyncResolvingServiceProvider<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Formats candidate descriptors and fallback policy.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncResolvingServiceProvider")
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
