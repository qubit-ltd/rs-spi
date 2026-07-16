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

use crate::ProviderSelector;

use super::{
    AttemptFailure,
    ProviderSelectorError,
};

/// Aggregate error returned when provider resolution cannot create a service.
#[derive(Clone, Debug)]
pub enum ResolutionError {
    /// Raw selector input failed normalization or syntax validation.
    InvalidSelector {
        /// Verbatim selector supplied by the caller.
        input: Box<str>,
        /// Zero-based chain position, or `None` for direct named selection.
        selector_index: Option<usize>,
        /// Parser error explaining why `input` was rejected.
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
    /// * `input` - Verbatim selector input supplied by the caller.
    /// * `selector_index` - Zero-based chain position, or `None` for a named
    ///   selector.
    /// * `source` - Selector parsing error that rejected `input`.
    ///
    /// # Returns
    ///
    /// An invalid-selector resolution error retaining its source.
    #[inline]
    #[must_use]
    pub(crate) fn invalid_selector(
        input: &str,
        selector_index: Option<usize>,
        source: ProviderSelectorError,
    ) -> Self {
        Self::InvalidSelector {
            input: input.into(),
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
    pub(crate) fn no_provider_succeeded(attempts: Vec<AttemptFailure>) -> Self {
        assert!(
            !attempts.is_empty(),
            "no-provider-succeeded errors require at least one attempt",
        );
        Self::NoProviderSucceeded {
            attempts: attempts.into_boxed_slice(),
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
                input,
                selector_index,
                ..
            } => match selector_index {
                Some(index) => write!(
                    formatter,
                    "invalid provider selector at chain index {index}: {input:?}",
                ),
                None => {
                    write!(formatter, "invalid provider selector {input:?}")
                }
            },
            Self::EmptySelection => formatter
                .write_str("provider selection chain must not be empty"),
            Self::UnknownProvider { selector } => {
                write!(formatter, "unknown provider: {selector}")
            }
            Self::EmptyRegistry => formatter
                .write_str("cannot resolve a provider from an empty registry"),
            Self::NoProviderSucceeded { attempts } => {
                write!(
                    formatter,
                    "no provider succeeded after {} attempt(s)",
                    attempts.len(),
                )?;
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
    /// The selector parse source for invalid input, the sole failed attempt for
    /// a single-attempt aggregate, or `None` otherwise.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSelector { source, .. } => Some(source),
            Self::NoProviderSucceeded { attempts } => match attempts.as_ref() {
                [attempt] => Some(attempt),
                _ => None,
            },
            Self::EmptySelection
            | Self::UnknownProvider { .. }
            | Self::EmptyRegistry => None,
        }
    }
}
