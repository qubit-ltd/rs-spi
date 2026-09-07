// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Aggregate errors raised while a resolver tries provider candidates.

use std::error::Error;
use std::fmt;

use super::ProviderAttemptFailure;
use crate::ProviderCreationTermination;

/// Nonempty aggregate returned when a resolver cannot create a service.
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
/// assert!(error.is_absence());
/// assert_eq!(1, error.attempts().len());
/// assert_eq!("echo", error.decisive_attempt().provider_id().as_str());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
#[must_use]
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
    pub(crate) fn stopped_by_policy(attempts: Vec<ProviderAttemptFailure<E>>) -> Self {
        Self::new(attempts, ProviderCreationTermination::StoppedByPolicy)
    }

    /// Returns ordered actual provider failures.
    ///
    /// # Returns
    ///
    /// The nonempty attempt sequence retained by this aggregate.
    #[inline(always)]
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
    pub fn into_parts(self) -> (Box<[ProviderAttemptFailure<E>]>, ProviderCreationTermination) {
        (self.attempts, self.termination)
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
    fn new(attempts: Vec<ProviderAttemptFailure<E>>, termination: ProviderCreationTermination) -> Self {
        assert!(
            !attempts.is_empty(),
            "provider creation errors require at least one attempt",
        );
        Self {
            attempts: attempts.into_boxed_slice(),
            termination,
        }
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
