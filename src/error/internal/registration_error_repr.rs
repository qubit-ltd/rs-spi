// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private representation for provider registration conflicts.

use thiserror::Error;

/// Variant-specific provider registration conflict.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum RegistrationErrorRepr {
    /// A selector is already owned by a registered provider.
    #[error(
        "provider selector {selector} claimed by {provider} is already owned by {existing_provider}"
    )]
    DuplicateSelector {
        /// Conflicting canonical ID or alias.
        selector: Box<str>,
        /// Canonical ID that already owns the selector.
        existing_provider: Box<str>,
        /// Canonical ID attempting the new claim.
        provider: Box<str>,
    },
}
