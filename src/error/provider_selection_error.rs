// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider selections.

use std::{
    error::Error,
    fmt,
};

use crate::ProviderSelector;

use super::ProviderSelectorError;

/// Error returned when a provider selection cannot be constructed or resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionError {
    /// One selector input cannot be parsed.
    #[non_exhaustive]
    InvalidSelector {
        /// Zero-based selector position, or `None` for a named selection.
        selector_index: Option<usize>,
        /// Selector parsing failure owning the verbatim input.
        source: ProviderSelectorError,
    },
    /// A chained selection contains no selector inputs.
    EmptyChain,
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
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
}

impl ProviderSelectionError {
    /// Creates an error for an invalid selection input.
    ///
    /// # Arguments
    ///
    /// * `selector_index` - Zero-based selector position, or `None` for a named
    ///   selection.
    /// * `source` - Selector parsing error that rejected the input.
    ///
    /// # Returns
    ///
    /// An invalid-selector selection error retaining its source.
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

    /// Creates an error for an empty chained selection.
    ///
    /// # Returns
    ///
    /// The empty-chain selection error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_chain() -> Self {
        Self::EmptyChain
    }

    /// Creates an error for a named selector that matched no provider.
    ///
    /// # Arguments
    ///
    /// * `selector` - Valid normalized selector that reached no registry entry.
    ///
    /// # Returns
    ///
    /// An unknown-provider selection error.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_provider(selector: ProviderSelector) -> Self {
        Self::UnknownProvider { selector }
    }

    /// Creates an error when a selector chain yields no provider candidates.
    ///
    /// # Arguments
    ///
    /// * `selectors` - Non-empty normalized selectors in input order.
    ///
    /// # Returns
    ///
    /// A no-candidates selection error retaining every requested selector.
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

    /// Creates an error for automatic selection from an empty registry.
    ///
    /// # Returns
    ///
    /// The empty-registry selection error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_registry() -> Self {
        Self::EmptyRegistry
    }
}

impl fmt::Display for ProviderSelectionError {
    /// Formats the selection construction failure.
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
                    "invalid provider selector at selection index {index}: {:?}",
                    source.input(),
                ),
                None => write!(
                    formatter,
                    "invalid provider selector {:?}",
                    source.input(),
                ),
            },
            Self::EmptyChain => formatter
                .write_str("provider selection chain must not be empty"),
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

impl Error for ProviderSelectionError {
    /// Returns the selector parsing failure when one is available.
    ///
    /// # Returns
    ///
    /// The selector parsing source for invalid input, or `None` for
    /// non-parser selection failures.
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSelector { source, .. } => Some(source),
            Self::EmptyChain
            | Self::UnknownProvider { .. }
            | Self::NoCandidates { .. }
            | Self::EmptyRegistry => None,
        }
    }
}
