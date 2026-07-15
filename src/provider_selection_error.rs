// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while constructing provider selections.

use thiserror::Error;

use crate::internal::ProviderSelectionErrorRepr;
use crate::{
    ProviderSelectionErrorKind,
    ProviderSelectorError,
};

/// Error returned when a provider selection cannot be constructed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct ProviderSelectionError(
    /// Variant-specific provider selection construction failure.
    ProviderSelectionErrorRepr,
);

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
        Self(ProviderSelectionErrorRepr::InvalidSelector {
            selector_index,
            selector_input: selector_input.into(),
            source,
        })
    }

    /// Creates an error for an empty chained selection.
    ///
    /// # Returns
    ///
    /// The empty-chain selection error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_chain() -> Self {
        Self(ProviderSelectionErrorRepr::EmptyChain)
    }

    /// Returns the selection construction rule that failed.
    ///
    /// # Returns
    ///
    /// The invalid-selector or empty-chain classification.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectionErrorKind {
        match self.0 {
            ProviderSelectionErrorRepr::InvalidSelector { .. } => {
                ProviderSelectionErrorKind::InvalidSelector
            }
            ProviderSelectionErrorRepr::EmptyChain => {
                ProviderSelectionErrorKind::EmptyChain
            }
        }
    }

    /// Returns the zero-based invalid selector position.
    ///
    /// # Returns
    ///
    /// `Some` for invalid selector input, or `None` for an empty chain.
    #[inline(always)]
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        match self.0 {
            ProviderSelectionErrorRepr::InvalidSelector {
                selector_index,
                ..
            } => Some(selector_index),
            ProviderSelectionErrorRepr::EmptyChain => None,
        }
    }

    /// Returns the verbatim invalid selector input.
    ///
    /// # Returns
    ///
    /// `Some` for invalid selector input, or `None` for an empty chain.
    #[inline(always)]
    #[must_use]
    pub fn selector_input(&self) -> Option<&str> {
        match &self.0 {
            ProviderSelectionErrorRepr::InvalidSelector {
                selector_input,
                ..
            } => Some(selector_input),
            ProviderSelectionErrorRepr::EmptyChain => None,
        }
    }
}
