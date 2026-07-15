// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for canonical provider ID validation failures.

use thiserror::Error;

/// Variant-specific canonical provider ID validation failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum ProviderIdErrorRepr {
    /// The supplied provider ID was empty.
    #[error("provider ID must not be empty")]
    Empty {
        /// Verbatim empty input retained for diagnostics.
        input: Box<str>,
    },
    /// The supplied provider ID violated canonical syntax.
    #[error("provider ID is not canonical: {input}")]
    NonCanonical {
        /// Verbatim noncanonical input retained for diagnostics.
        input: Box<str>,
    },
}
