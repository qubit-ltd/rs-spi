// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
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

/// Error returned when a Registry cannot resolve provider candidates.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderResolutionError {
    /// One named selection matched no registered provider.
    #[non_exhaustive]
    UnknownProvider {
        /// Normalized selector that matched no provider.
        selector: ProviderSelector,
    },
    /// A nonempty selector chain matched no registered provider candidates.
    #[non_exhaustive]
    NoCandidates {
        /// Normalized selectors that matched no candidates, in input order.
        selectors: Box<[ProviderSelector]>,
    },
    /// Automatic selection was requested from an empty Registry.
    EmptyRegistry,
}

impl ProviderResolutionError {
    /// Creates an error for a named selector that matched no provider.
    ///
    /// # Parameters
    ///
    /// * `selector` - Valid normalized selector that reached no Registry entry.
    ///
    /// # Returns
    ///
    /// An unknown-provider resolution error.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_provider(selector: ProviderSelector) -> Self {
        Self::UnknownProvider { selector }
    }

    /// Creates an error when a selector chain yields no provider candidates.
    ///
    /// # Parameters
    ///
    /// * `selectors` - Non-empty normalized selectors in input order.
    ///
    /// # Returns
    ///
    /// A no-candidates resolution error retaining every requested selector.
    ///
    /// # Panics
    ///
    /// Panics when `selectors` is empty.
    #[inline]
    #[must_use]
    pub(crate) fn no_candidates(selectors: Vec<ProviderSelector>) -> Self {
        assert!(
            !selectors.is_empty(),
            "no-candidates errors require at least one selector",
        );
        Self::NoCandidates {
            selectors: selectors.into_boxed_slice(),
        }
    }

    /// Creates an error for automatic selection from an empty Registry.
    ///
    /// # Returns
    ///
    /// The empty-Registry resolution error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_registry() -> Self {
        Self::EmptyRegistry
    }
}

impl fmt::Display for ProviderResolutionError {
    /// Formats the provider resolution failure.
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
        match self {
            Self::UnknownProvider { selector } => {
                write!(formatter, "unknown provider: {selector}")
            }
            Self::NoCandidates { selectors } => {
                formatter.write_str("no provider candidates matched")?;
                for selector in selectors {
                    write!(formatter, "; {selector}")?;
                }
                Ok(())
            }
            Self::EmptyRegistry => formatter
                .write_str("cannot select a provider from an empty registry"),
        }
    }
}

impl Error for ProviderResolutionError {}
