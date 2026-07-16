// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider selections.

use thiserror::Error;

use super::ProviderSelectorError;

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
}
