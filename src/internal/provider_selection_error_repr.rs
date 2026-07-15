// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for provider selection construction failures.

use thiserror::Error;

use crate::ProviderSelectorError;

/// Variant-specific provider selection construction failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum ProviderSelectionErrorRepr {
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
