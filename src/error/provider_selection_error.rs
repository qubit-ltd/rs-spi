// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider selections.

use thiserror::Error;

use super::{
    ProviderSelectionErrorKind,
    ProviderSelectorError,
};

/// Error returned when a provider selection cannot be constructed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionError {
    /// One selector input cannot be parsed.
    #[error(
        "invalid provider selector at selection index {selector_index}: {selector_input:?}"
    )]
    InvalidSelector {
        /// Zero-based selector position.
        selector_index: usize,
        /// Verbatim invalid selector input.
        selector_input: Box<str>,
        /// Selector parsing failure.
        #[source]
        source: ProviderSelectorError,
    },
    /// A chained selection contains no selector inputs.
    #[error("provider selection chain must not be empty")]
    EmptyChain,
}

impl ProviderSelectionError {
    /// Creates an error for an invalid selection input.
    ///
    /// # Arguments
    ///
    /// * `selector_index` - Zero-based selector position.
    /// * `selector_input` - Verbatim invalid selector input.
    /// * `source` - Selector parsing error that rejected the input.
    ///
    /// # Returns
    ///
    /// An invalid-selector selection error retaining its source.
    #[inline]
    #[must_use]
    pub(crate) fn invalid_selector(
        selector_index: usize,
        selector_input: &str,
        source: ProviderSelectorError,
    ) -> Self {
        Self::InvalidSelector {
            selector_index,
            selector_input: selector_input.into(),
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

    /// Returns this selection error's stable classification.
    ///
    /// # Returns
    ///
    /// The invalid-selector or empty-chain classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectionErrorKind {
        match self {
            Self::InvalidSelector { .. } => {
                ProviderSelectionErrorKind::InvalidSelector
            }
            Self::EmptyChain => ProviderSelectionErrorKind::EmptyChain,
        }
    }

    /// Returns the zero-based invalid selector position.
    ///
    /// # Returns
    ///
    /// The invalid selector index, or `None` for an empty chain.
    #[inline(always)]
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        match self {
            Self::InvalidSelector { selector_index, .. } => {
                Some(*selector_index)
            }
            Self::EmptyChain => None,
        }
    }

    /// Returns verbatim invalid selector input.
    ///
    /// # Returns
    ///
    /// The invalid input, or `None` for an empty chain.
    #[inline(always)]
    #[must_use]
    pub fn selector_input(&self) -> Option<&str> {
        match self {
            Self::InvalidSelector { selector_input, .. } => {
                Some(selector_input)
            }
            Self::EmptyChain => None,
        }
    }

    /// Returns the selector parser error retained by invalid input.
    ///
    /// # Returns
    ///
    /// The parser source, or `None` for an empty chain.
    #[inline(always)]
    #[must_use]
    pub const fn selector_error(&self) -> Option<&ProviderSelectorError> {
        match self {
            Self::InvalidSelector { source, .. } => Some(source),
            Self::EmptyChain => None,
        }
    }

    /// Reports whether selection construction received an empty chain.
    ///
    /// # Returns
    ///
    /// `true` only for [`Self::EmptyChain`].
    #[inline(always)]
    #[must_use]
    pub const fn is_empty_chain(&self) -> bool {
        matches!(self, Self::EmptyChain)
    }
}
