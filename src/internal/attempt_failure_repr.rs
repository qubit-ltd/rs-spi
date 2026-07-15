// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for one failed provider resolution attempt.

use crate::{
    ProviderError,
    ProviderId,
    ProviderSelector,
};

/// Variant-specific diagnostics for one failed resolution attempt.
#[derive(Clone, Debug)]
pub(crate) enum AttemptFailureRepr {
    /// Selector lookup reached no provider.
    UnknownProvider {
        /// Selector retained from the request.
        requested_selector: ProviderSelector,
        /// Human-readable lookup failure.
        reason: Box<str>,
    },
    /// A provider factory returned a classified error.
    ProviderError {
        /// Explicit selector, or `None` for automatic selection.
        requested_selector: Option<ProviderSelector>,
        /// Canonical provider reached by lookup.
        provider_id: ProviderId,
        /// Original provider error retained without diagnostic copying.
        error: ProviderError,
    },
}
