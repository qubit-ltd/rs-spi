// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for aggregate provider resolution failures.

use crate::{
    AttemptFailure,
    ProviderSelector,
    ProviderSelectorError,
};

/// Variant-specific storage for an aggregate resolution failure.
#[derive(Clone, Debug)]
pub(crate) enum ResolutionErrorRepr {
    /// Raw selector input was invalid.
    InvalidSelector {
        /// Verbatim input supplied by the caller.
        input: Box<str>,
        /// Zero-based chain position, or `None` for named selection.
        selector_index: Option<usize>,
        /// Selector parsing failure.
        source: ProviderSelectorError,
    },
    /// A raw chained selection contains no inputs.
    EmptySelection,
    /// A valid normalized selector matched no provider.
    UnknownProvider {
        /// Normalized unknown selector.
        selector: ProviderSelector,
    },
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// No considered candidate produced a service.
    NoProviderSucceeded {
        /// Failures retained in encounter order.
        attempts: Box<[AttemptFailure]>,
    },
}
