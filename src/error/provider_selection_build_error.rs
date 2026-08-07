// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider selections.

use std::error::Error;
use std::fmt;

use super::ProviderSelectorError;

/// Error returned when a provider selection cannot be constructed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionBuildError {
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
}

impl ProviderSelectionBuildError {
    /// Returns the selector parsing failure, when construction failed because
    /// of an invalid selector.
    ///
    /// # Returns
    ///
    /// The selector parsing failure for [`Self::InvalidSelector`], or `None`
    /// for [`Self::EmptyChain`].
    #[inline(always)]
    #[must_use]
    pub const fn selector_error(&self) -> Option<&ProviderSelectorError> {
        match self {
            Self::InvalidSelector { source, .. } => Some(source),
            Self::EmptyChain => None,
        }
    }

    /// Returns the zero-based position of an invalid chained selector.
    ///
    /// # Returns
    ///
    /// The selector position for an invalid chained selection, or `None` for
    /// an invalid named selection and an empty chain.
    #[inline(always)]
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        match self {
            Self::InvalidSelector { selector_index, .. } => *selector_index,
            Self::EmptyChain => None,
        }
    }

    /// Tests whether an empty selector chain caused this failure.
    ///
    /// # Returns
    ///
    /// `true` for [`Self::EmptyChain`]; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub const fn is_empty_chain(&self) -> bool {
        matches!(self, Self::EmptyChain)
    }

    /// Creates an error for an invalid selection input.
    ///
    /// # Parameters
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
}

impl fmt::Display for ProviderSelectionBuildError {
    /// Formats the selection construction failure.
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
        }
    }
}

impl Error for ProviderSelectionBuildError {
    /// Returns the selector parsing failure when one is available.
    ///
    /// # Returns
    ///
    /// The selector parsing source for invalid input, or `None` for an empty
    /// selection chain.
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSelector { source, .. } => Some(source),
            Self::EmptyChain => None,
        }
    }
}
