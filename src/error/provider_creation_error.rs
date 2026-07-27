// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Aggregate errors raised while a resolver tries provider candidates.

use std::{
    error::Error,
    fmt,
};

use crate::ProviderCreationTermination;

use super::ProviderAttemptFailure;

/// Nonempty aggregate returned when a resolver cannot create a service.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ProviderCreationError<E> {
    /// Actual provider failures in encounter order.
    attempts: Box<[ProviderAttemptFailure<E>]>,
    /// Reason candidate traversal ended without a service.
    termination: ProviderCreationTermination,
}

impl<E> ProviderCreationError<E> {
    /// Creates an aggregate after every admitted candidate fails.
    ///
    /// # Parameters
    ///
    /// * `attempts` - Nonempty provider failures in encounter order.
    ///
    /// # Returns
    ///
    /// An aggregate creation error marked as exhausted.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    #[inline(always)]
    #[must_use]
    pub(crate) fn exhausted(attempts: Vec<ProviderAttemptFailure<E>>) -> Self {
        Self::new(attempts, ProviderCreationTermination::Exhausted)
    }

    /// Creates an aggregate after fallback policy stops traversal.
    ///
    /// # Parameters
    ///
    /// * `attempts` - Nonempty provider failures recorded before the stop.
    ///
    /// # Returns
    ///
    /// An aggregate creation error marked as stopped by policy.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    #[inline(always)]
    #[must_use]
    pub(crate) fn stopped_by_policy(
        attempts: Vec<ProviderAttemptFailure<E>>,
    ) -> Self {
        Self::new(attempts, ProviderCreationTermination::StoppedByPolicy)
    }

    /// Creates an aggregate with an explicit traversal termination.
    ///
    /// # Parameters
    ///
    /// * `attempts` - Nonempty provider failures in encounter order.
    /// * `termination` - Reason traversal ended without a service.
    ///
    /// # Returns
    ///
    /// An aggregate creation error retaining all attempts.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    #[inline]
    #[must_use]
    fn new(
        attempts: Vec<ProviderAttemptFailure<E>>,
        termination: ProviderCreationTermination,
    ) -> Self {
        assert!(
            !attempts.is_empty(),
            "provider creation errors require at least one attempt",
        );
        Self {
            attempts: attempts.into_boxed_slice(),
            termination,
        }
    }

    /// Returns ordered actual provider failures.
    ///
    /// # Returns
    ///
    /// The nonempty attempt sequence retained by this aggregate.
    #[inline(always)]
    #[must_use]
    pub const fn attempts(&self) -> &[ProviderAttemptFailure<E>] {
        &self.attempts
    }

    /// Returns why candidate traversal ended.
    ///
    /// # Returns
    ///
    /// The resolver's traversal termination reason.
    #[inline(always)]
    #[must_use]
    pub const fn termination(&self) -> ProviderCreationTermination {
        self.termination
    }

    /// Returns the final actual provider failure.
    ///
    /// # Returns
    ///
    /// The last attempt, which directly terminated or exhausted traversal.
    ///
    /// # Panics
    ///
    /// Panics only if the internal nonempty-attempt invariant is violated.
    #[inline]
    #[must_use]
    pub fn decisive_attempt(&self) -> &ProviderAttemptFailure<E> {
        self.attempts
            .last()
            .expect("provider creation errors contain an attempt")
    }

    /// Reports whether every failure denotes unsupported or unavailable
    /// service.
    ///
    /// # Returns
    ///
    /// `true` when every attempt is classified as unsupported or unavailable.
    #[must_use]
    pub fn is_absence(&self) -> bool {
        self.attempts
            .iter()
            .all(|attempt| attempt.failure().kind().is_absence())
    }

    /// Transfers ownership of all attempts and the termination reason.
    ///
    /// # Returns
    ///
    /// The ordered attempts and the reason traversal ended.
    #[inline(always)]
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Box<[ProviderAttemptFailure<E>]>,
        ProviderCreationTermination,
    ) {
        (self.attempts, self.termination)
    }
}

impl<E> fmt::Display for ProviderCreationError<E>
where
    E: fmt::Display,
{
    /// Formats aggregate provider creation diagnostics.
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
        match self.termination {
            ProviderCreationTermination::Exhausted => write!(
                formatter,
                "no provider succeeded after {} attempt(s)",
                self.attempts.len(),
            )?,
            ProviderCreationTermination::StoppedByPolicy => write!(
                formatter,
                "provider creation stopped by fallback policy after {} attempt(s)",
                self.attempts.len(),
            )?,
        }
        for (index, attempt) in self.attempts.iter().enumerate() {
            write!(formatter, "; attempt {}: {attempt}", index + 1)?;
        }
        Ok(())
    }
}

impl<E> Error for ProviderCreationError<E>
where
    E: Error + 'static,
{
    /// Returns the final provider attempt as the decisive cause.
    ///
    /// # Returns
    ///
    /// The final actual provider failure.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.decisive_attempt())
    }
}
