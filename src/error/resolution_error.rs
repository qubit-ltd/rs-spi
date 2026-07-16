// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while resolving provider selections.

use std::{
    error::Error,
    fmt,
};

use crate::{
    ProviderSelector,
    ResolutionTermination,
};

use super::{
    AttemptFailure,
    ProviderErrorKind,
    ProviderSelectionError,
    ProviderSelectorError,
};

/// Aggregate error returned when provider resolution cannot create a service.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResolutionError {
    /// Raw selector input failed normalization or syntax validation.
    InvalidSelector {
        /// Zero-based chain position, or `None` for direct named selection.
        selector_index: Option<usize>,
        /// Parser error owning the verbatim selector input.
        source: ProviderSelectorError,
    },
    /// A raw chained selection contained no selectors.
    EmptySelection,
    /// A valid normalized selector matched no registry entry.
    UnknownProvider {
        /// Normalized selector that matched no provider.
        selector: ProviderSelector,
    },
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// Every considered candidate failed or resolution stopped by policy.
    NoProviderSucceeded {
        /// Attempt diagnostics in encounter order.
        attempts: Box<[AttemptFailure]>,
        /// Reason candidate traversal ended without a service.
        termination: ResolutionTermination,
    },
}

impl ResolutionError {
    /// Creates an error for a direct selector that matched no provider.
    ///
    /// # Arguments
    ///
    /// * `selector` - Valid normalized selector that reached no registry entry.
    ///
    /// # Returns
    ///
    /// An unknown-provider resolution error.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_provider(selector: ProviderSelector) -> Self {
        Self::UnknownProvider { selector }
    }

    /// Creates an error for invalid raw selector input.
    ///
    /// # Arguments
    ///
    /// * `selector_index` - Zero-based chain position, or `None` for a named
    ///   selector.
    /// * `source` - Selector parsing error that rejected and owns the input.
    ///
    /// # Returns
    ///
    /// An invalid-selector resolution error retaining its source.
    #[inline]
    #[must_use]
    pub(crate) fn invalid_selector(
        selector_index: Option<usize>,
        source: ProviderSelectorError,
    ) -> Self {
        Self::InvalidSelector {
            selector_index,
            source,
        }
    }

    /// Creates an error for an empty raw chained selection.
    ///
    /// # Returns
    ///
    /// The empty-selection resolution error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_selection() -> Self {
        Self::EmptySelection
    }

    /// Creates an error for automatic resolution from an empty registry.
    ///
    /// # Returns
    ///
    /// The empty-registry resolution error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_registry() -> Self {
        Self::EmptyRegistry
    }

    /// Creates an aggregate error when considered candidates produce no
    /// service.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty failures recorded in encounter order.
    ///
    /// # Returns
    ///
    /// An aggregate resolution error owning every failed attempt.
    ///
    /// # Panics
    ///
    /// Panics when the resolver supplies an empty attempt list, which violates
    /// the internal `NoProviderSucceeded` invariant.
    #[inline]
    #[must_use]
    pub(crate) fn exhausted(attempts: Vec<AttemptFailure>) -> Self {
        Self::no_provider_succeeded(attempts, ResolutionTermination::Exhausted)
    }

    /// Creates an aggregate error after fallback policy stops traversal.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty failures recorded before policy stopped.
    ///
    /// # Returns
    ///
    /// An aggregate resolution error marked as stopped by policy.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    #[inline]
    #[must_use]
    pub(crate) fn stopped_by_policy(attempts: Vec<AttemptFailure>) -> Self {
        Self::no_provider_succeeded(
            attempts,
            ResolutionTermination::StoppedByPolicy,
        )
    }

    /// Creates an aggregate resolution error with an explicit termination.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty failures recorded in encounter order.
    /// * `termination` - Reason traversal ended without a service.
    ///
    /// # Returns
    ///
    /// An aggregate resolution error retaining attempts and termination.
    ///
    /// # Panics
    ///
    /// Panics when `attempts` is empty.
    fn no_provider_succeeded(
        attempts: Vec<AttemptFailure>,
        termination: ResolutionTermination,
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

    /// Returns ordered failed attempts.
    ///
    /// # Returns
    ///
    /// Aggregate attempts, or an empty slice for non-aggregate errors.
    #[inline(always)]
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFailure] {
        match self {
            Self::NoProviderSucceeded { attempts, .. } => attempts,
            _ => &[],
        }
    }

    /// Returns why aggregate candidate traversal ended.
    ///
    /// # Returns
    ///
    /// The aggregate termination reason, or `None` for non-aggregate errors.
    #[inline(always)]
    #[must_use]
    pub const fn termination(&self) -> Option<ResolutionTermination> {
        match self {
            Self::NoProviderSucceeded { termination, .. } => Some(*termination),
            _ => None,
        }
    }

    /// Returns the final recorded attempt.
    ///
    /// # Returns
    ///
    /// The last aggregate attempt, or `None` when no attempts are stored.
    #[inline(always)]
    #[must_use]
    pub fn terminal_attempt(&self) -> Option<&AttemptFailure> {
        self.attempts().last()
    }

    /// Returns the attempt that directly explains the aggregate outcome.
    ///
    /// # Returns
    ///
    /// The terminal attempt after a policy stop, the only attempt after
    /// singleton exhaustion, or `None` for non-aggregate errors and ambiguous
    /// multi-attempt exhaustion.
    #[inline]
    #[must_use]
    pub fn decisive_attempt(&self) -> Option<&AttemptFailure> {
        match self {
            Self::NoProviderSucceeded {
                attempts,
                termination: ResolutionTermination::StoppedByPolicy,
            } => attempts.last(),
            Self::NoProviderSucceeded {
                attempts,
                termination: ResolutionTermination::Exhausted,
            } => match attempts.as_ref() {
                [attempt] => Some(attempt),
                _ => None,
            },
            _ => None,
        }
    }

    /// Reports whether failure means providers were absent or unavailable.
    ///
    /// # Returns
    ///
    /// `true` for an unknown direct provider or an aggregate containing only
    /// unknown, unsupported, or unavailable attempts.
    #[inline]
    #[must_use]
    pub fn is_absence(&self) -> bool {
        match self {
            Self::UnknownProvider { .. } => true,
            Self::NoProviderSucceeded { attempts, .. } => {
                attempts.iter().all(|attempt| match attempt {
                    AttemptFailure::UnknownProvider { .. } => true,
                    AttemptFailure::ProviderError { error, .. } => matches!(
                        error.kind(),
                        ProviderErrorKind::Unsupported
                            | ProviderErrorKind::Unavailable
                    ),
                })
            }
            _ => false,
        }
    }
}

impl From<ProviderSelectionError> for ResolutionError {
    /// Converts validated-selection construction failures for resolver use.
    ///
    /// # Arguments
    ///
    /// * `error` - Selection construction failure to preserve.
    ///
    /// # Returns
    ///
    /// The corresponding invalid-selector or empty-selection resolution error.
    #[inline]
    fn from(error: ProviderSelectionError) -> Self {
        match error {
            ProviderSelectionError::InvalidSelector {
                selector_index,
                source,
            } => Self::invalid_selector(selector_index, source),
            ProviderSelectionError::EmptyChain => Self::empty_selection(),
        }
    }
}

impl fmt::Display for ResolutionError {
    /// Formats the resolution failure and ordered attempt diagnostics.
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
            Self::InvalidSelector {
                selector_index,
                source,
            } => match selector_index {
                Some(index) => write!(
                    formatter,
                    "invalid provider selector at chain index {index}: {:?}",
                    source.input(),
                ),
                None => write!(
                    formatter,
                    "invalid provider selector {:?}",
                    source.input(),
                ),
            },
            Self::EmptySelection => formatter
                .write_str("provider selection chain must not be empty"),
            Self::UnknownProvider { selector } => {
                write!(formatter, "unknown provider: {selector}")
            }
            Self::EmptyRegistry => formatter
                .write_str("cannot resolve a provider from an empty registry"),
            Self::NoProviderSucceeded {
                attempts,
                termination,
            } => {
                match termination {
                    ResolutionTermination::Exhausted => write!(
                        formatter,
                        "no provider succeeded after {} attempt(s)",
                        attempts.len(),
                    )?,
                    ResolutionTermination::StoppedByPolicy => write!(
                        formatter,
                        "provider resolution stopped by fallback policy after {} attempt(s)",
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

impl Error for ResolutionError {
    /// Returns the direct cause when the failure has one unambiguous source.
    ///
    /// # Returns
    ///
    /// The selector parse source for invalid input, the decisive failed attempt
    /// for an aggregate, or `None` when no single attempt explains the outcome.
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSelector { source, .. } => Some(source),
            Self::NoProviderSucceeded { .. } => self
                .decisive_attempt()
                .map(|attempt| attempt as &(dyn Error + 'static)),
            Self::EmptySelection
            | Self::UnknownProvider { .. }
            | Self::EmptyRegistry => None,
        }
    }
}
