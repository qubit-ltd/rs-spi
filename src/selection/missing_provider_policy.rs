// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy for selectors that do not match a registered provider.

/// Controls how a provider chain handles selectors that are not registered.
///
/// This enum is non-exhaustive. Downstream matches must include a wildcard arm
/// so future policies remain source-compatible.
///
/// ```compile_fail
/// use qubit_spi::MissingProviderPolicy;
///
/// fn describe(policy: MissingProviderPolicy) -> &'static str {
///     match policy {
///         MissingProviderPolicy::Reject => "reject",
///         MissingProviderPolicy::Ignore => "ignore",
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum MissingProviderPolicy {
    /// Reject the complete selection when any selector is unknown.
    #[default]
    Reject,
    /// Ignore unknown selectors and retain every known candidate.
    Ignore,
}
