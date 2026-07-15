// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for provider selector parsing failures.

use thiserror::Error;

/// Variant-specific provider selector parsing failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum ProviderSelectorErrorRepr {
    /// Trimming the input produced an empty selector.
    #[error("provider selector must not be empty")]
    Empty {
        /// Verbatim selector input.
        input: Box<str>,
    },
    /// The normalized selector violated selector syntax.
    #[error("invalid provider selector {input:?} (normalized as {normalized:?})")]
    Invalid {
        /// Verbatim selector input.
        input: Box<str>,
        /// Trimmed and ASCII-lowercased selector input.
        normalized: Box<str>,
    },
}
