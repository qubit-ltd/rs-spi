// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed views of validated provider selection targets.

use crate::MissingProviderPolicy;
use crate::ProviderSelector;

/// Borrowed, lossless view of a provider selection target.
///
/// This enum is non-exhaustive. Downstream matches must include a wildcard arm
/// so future selection targets remain source-compatible.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of selectors borrowed from the owning selection.
///
/// ```compile_fail
/// use qubit_spi::ProviderSelectionTargetRef;
///
/// fn describe(target: ProviderSelectionTargetRef<'_>) -> &'static str {
///     match target {
///         ProviderSelectionTargetRef::Auto => "auto",
///         ProviderSelectionTargetRef::Named(_) => "named",
///         ProviderSelectionTargetRef::Chain { .. } => "chain",
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
