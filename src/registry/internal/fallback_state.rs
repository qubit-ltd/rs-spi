// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared fallback state transitions for provider resolvers.

use crate::error::{
    ProviderAttemptFailure,
    ProviderCreationError,
    ProviderFailure,
};
use crate::{
    FallbackPolicy,
    ProviderId,
};

/// Mutable failure state retained while a resolver traverses candidates.
pub(crate) struct FallbackState<E> {
    /// Policy deciding whether an untried candidate remains admissible.
    policy: FallbackPolicy,
    /// Actual provider failures in encounter order.
    attempts: Vec<ProviderAttemptFailure<E>>,
}

impl<E> FallbackState<E> {
    /// Creates empty traversal state for one fallback policy.
    ///
    /// # Parameters
    ///
    /// * `policy` - Policy applied after each candidate failure.
    ///
    /// # Returns
    ///
    /// Empty state ready to record the first attempt.
    #[inline]
    #[must_use]
    pub(crate) const fn new(policy: FallbackPolicy) -> Self {
        Self {
            policy,
            attempts: Vec::new(),
        }
    }

    /// Records one failure and decides whether traversal must terminate.
    ///
    /// # Parameters
    ///
    /// * `provider_id` - Canonical ID of the provider that failed.
    /// * `error` - Classified leaf provider error.
    /// * `has_remaining` - Whether an untried candidate remains.
    ///
    /// # Returns
    ///
    /// `Some(error)` when traversal is exhausted or stopped by policy;
    /// otherwise, `None` to permit the next candidate.
    pub(crate) fn record_failure(
        &mut self,
        provider_id: ProviderId,
        failure: ProviderFailure<E>,
        has_remaining: bool,
    ) -> Option<ProviderCreationError<E>> {
        let kind = failure.kind();
        self.attempts
            .push(ProviderAttemptFailure::new(provider_id, failure));
        if !has_remaining {
            return Some(ProviderCreationError::exhausted(std::mem::take(
                &mut self.attempts,
            )));
        }
        if !self.policy.should_continue_after(kind) {
            return Some(ProviderCreationError::stopped_by_policy(
                std::mem::take(&mut self.attempts),
            ));
        }
        None
    }
}
