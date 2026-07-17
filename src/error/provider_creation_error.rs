// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while providers create service instances.

use std::{
    error::Error,
    fmt,
};

use crate::ProviderCreationTermination;

use super::{
    ProviderAttemptFailure,
    ProviderError,
    ProviderErrorKind,
};

/// Error returned when a provider cannot create a requested service.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ProviderCreationError {
    /// One directly invoked provider reported a classified failure.
    Provider(ProviderError),
    /// Every considered provider failed or traversal stopped by policy.
    #[non_exhaustive]
    NoProviderSucceeded {
        /// Actual provider failures in encounter order.
        attempts: Box<[ProviderAttemptFailure]>,
        /// Reason candidate traversal ended without a service.
        termination: ProviderCreationTermination,
    },
}

impl ProviderCreationError {
    /// Creates an aggregate after all admitted candidates fail.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty provider failures in encounter order.
    ///
    /// # Returns
    ///
    /// An aggregate creation error marked as exhausted.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    #[inline]
    #[must_use]
    pub(crate) fn exhausted(attempts: Vec<ProviderAttemptFailure>) -> Self {
        Self::no_provider_succeeded(
            attempts,
            ProviderCreationTermination::Exhausted,
        )
    }

    /// Creates an aggregate after fallback policy stops traversal.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty provider failures recorded before the stop.
    ///
    /// # Returns
    ///
    /// An aggregate creation error marked as stopped by policy.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    #[inline]
    #[must_use]
    pub(crate) fn stopped_by_policy(
        attempts: Vec<ProviderAttemptFailure>,
    ) -> Self {
        Self::no_provider_succeeded(
            attempts,
            ProviderCreationTermination::StoppedByPolicy,
        )
    }

    /// Creates an aggregate with an explicit traversal termination.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty provider failures in encounter order.
    /// * `termination` - Reason traversal ended without a service.
    ///
    /// # Returns
    ///
    /// An aggregate creation error retaining all attempts.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    fn no_provider_succeeded(
        attempts: Vec<ProviderAttemptFailure>,
        termination: ProviderCreationTermination,
    ) -> Self {
        assert!(
            !attempts.is_empty(),
            "no-provider-succeeded errors require at least one attempt",
        );
        Self::NoProviderSucceeded {
            attempts: attempts.into_boxed_slice(),
            termination,
        }
    }

    /// Returns ordered actual provider failures.
    ///
    /// # Returns
    ///
    /// Aggregate attempts, or an empty slice for a direct provider error.
    #[inline(always)]
    #[must_use]
    pub fn attempts(&self) -> &[ProviderAttemptFailure] {
        match self {
            Self::Provider(_) => &[],
            Self::NoProviderSucceeded { attempts, .. } => attempts,
        }
    }

    /// Returns why aggregate candidate traversal ended.
    ///
    /// # Returns
    ///
    /// The aggregate termination reason, or `None` for a direct provider error.
    #[inline(always)]
    #[must_use]
    pub const fn termination(&self) -> Option<ProviderCreationTermination> {
        match self {
            Self::Provider(_) => None,
            Self::NoProviderSucceeded { termination, .. } => Some(*termination),
        }
    }

    /// Returns the final recorded provider failure.
    ///
    /// # Returns
    ///
    /// The last aggregate attempt, or `None` for a direct provider error.
    #[inline(always)]
    #[must_use]
    pub fn terminal_attempt(&self) -> Option<&ProviderAttemptFailure> {
        self.attempts().last()
    }

    /// Returns the provider failure that directly explains the aggregate.
    ///
    /// # Returns
    ///
    /// The terminal attempt after a policy stop, the only attempt after
    /// singleton exhaustion, or `None` when no single aggregate attempt is
    /// decisive.
    #[inline]
    #[must_use]
    pub fn decisive_attempt(&self) -> Option<&ProviderAttemptFailure> {
        match self {
            Self::NoProviderSucceeded {
                attempts,
                termination: ProviderCreationTermination::StoppedByPolicy,
            } => attempts.last(),
            Self::NoProviderSucceeded {
                attempts,
                termination: ProviderCreationTermination::Exhausted,
            } => match attempts.as_ref() {
                [attempt] => Some(attempt),
                _ => None,
            },
            Self::Provider(_) => None,
        }
    }

    /// Reports whether the failure denotes unsupported or unavailable service.
    ///
    /// # Returns
    ///
    /// `true` when the direct error, or every aggregate attempt, is classified
    /// as unsupported or unavailable.
    #[inline]
    #[must_use]
    pub fn is_absence(&self) -> bool {
        match self {
            Self::Provider(error) => is_absence_kind(error.kind()),
            Self::NoProviderSucceeded { attempts, .. } => attempts
                .iter()
                .all(|attempt| is_absence_kind(attempt.error().kind())),
        }
    }
}

impl From<ProviderError> for ProviderCreationError {
    /// Wraps one provider's classified failure for the unified provider API.
    ///
    /// # Arguments
    ///
    /// * `error` - Leaf provider error to preserve.
    ///
    /// # Returns
    ///
    /// A direct provider creation error retaining its source chain.
    #[inline(always)]
    fn from(error: ProviderError) -> Self {
        Self::Provider(error)
    }
}

impl fmt::Display for ProviderCreationError {
    /// Formats direct or aggregate provider creation diagnostics.
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
        match self {
            Self::Provider(error) => write!(formatter, "{error}"),
            Self::NoProviderSucceeded {
                attempts,
                termination,
            } => {
                match termination {
                    ProviderCreationTermination::Exhausted => write!(
                        formatter,
                        "no provider succeeded after {} attempt(s)",
                        attempts.len(),
                    )?,
                    ProviderCreationTermination::StoppedByPolicy => write!(
                        formatter,
                        "provider creation stopped by fallback policy after {} attempt(s)",
                        attempts.len(),
                    )?,
                }
                for (index, attempt) in attempts.iter().enumerate() {
                    write!(formatter, "; attempt {}: {attempt}", index + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ProviderCreationError {
    /// Returns the direct cause when one source explains the outcome.
    ///
    /// # Returns
    ///
    /// The leaf provider error, the decisive aggregate attempt, or `None` when
    /// multiple exhausted attempts are equally relevant.
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Provider(error) => Some(error),
            Self::NoProviderSucceeded { .. } => self
                .decisive_attempt()
                .map(|attempt| attempt as &(dyn Error + 'static)),
        }
    }
}

/// Reports whether a provider failure kind represents absence.
///
/// # Arguments
///
/// * `kind` - Provider-reported failure classification.
///
/// # Returns
///
/// `true` for unsupported and unavailable providers.
#[inline(always)]
const fn is_absence_kind(kind: ProviderErrorKind) -> bool {
    matches!(
        kind,
        ProviderErrorKind::Unsupported | ProviderErrorKind::Unavailable
    )
}
