// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider selections.

use thiserror::Error;

use crate::ProviderSelectorError;

/// Classification of a provider selection construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionErrorKind {
    /// One selector input cannot be parsed.
    InvalidSelector,
    /// A chained selection contains no selector inputs.
    EmptyChain,
}

/// Error returned when a provider selection cannot be constructed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderSelectionError(ProviderSelectionErrorRepr);

/// Private representation of provider selection construction failures.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
enum ProviderSelectionErrorRepr {
    /// One selector input cannot be parsed.
    #[error("invalid provider selector at selection index {selector_index}: {selector_input:?}")]
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
    #[must_use]
    pub(crate) fn invalid_selector(
        selector_index: usize,
        selector_input: impl AsRef<str>,
        source: ProviderSelectorError,
    ) -> Self {
        Self(ProviderSelectionErrorRepr::InvalidSelector {
            selector_index,
            selector_input: selector_input.as_ref().into(),
            source,
        })
    }

    /// Creates an error for an empty chained selection.
    #[must_use]
    pub(crate) const fn empty_chain() -> Self {
        Self(ProviderSelectionErrorRepr::EmptyChain)
    }

    /// Returns the selection construction rule that failed.
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectionErrorKind {
        match self.0 {
            ProviderSelectionErrorRepr::InvalidSelector { .. } => {
                ProviderSelectionErrorKind::InvalidSelector
            }
            ProviderSelectionErrorRepr::EmptyChain => ProviderSelectionErrorKind::EmptyChain,
        }
    }

    /// Returns the zero-based invalid selector position, when applicable.
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        match self.0 {
            ProviderSelectionErrorRepr::InvalidSelector { selector_index, .. } => {
                Some(selector_index)
            }
            ProviderSelectionErrorRepr::EmptyChain => None,
        }
    }

    /// Returns the verbatim invalid selector input, when applicable.
    #[must_use]
    pub fn selector_input(&self) -> Option<&str> {
        match &self.0 {
            ProviderSelectionErrorRepr::InvalidSelector { selector_input, .. } => {
                Some(selector_input)
            }
            ProviderSelectionErrorRepr::EmptyChain => None,
        }
    }
}
