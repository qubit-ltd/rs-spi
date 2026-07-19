// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed views of validated provider selection targets.

use crate::{
    MissingProviderPolicy,
    ProviderSelector,
};

/// Borrowed, lossless view of a provider selection target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderSelectionTargetRef<'a> {
    /// Providers are selected in deterministic automatic order.
    Auto,
    /// Exactly one normalized selector is requested.
    Named(
        /// Normalized selector naming the only candidate.
        &'a ProviderSelector,
    ),
    /// A nonempty ordered selector chain is requested.
    Chain {
        /// Normalized selectors retained in caller-supplied order.
        selectors: &'a [ProviderSelector],
        /// Policy applied to selectors that are not registered.
        missing_policy: MissingProviderPolicy,
    },
}
