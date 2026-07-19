// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy for selectors that do not match a registered provider.

/// Controls how a provider chain handles selectors that are not registered.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MissingProviderPolicy {
    /// Reject the complete selection when any selector is unknown.
    #[default]
    Reject,
    /// Ignore unknown selectors and retain every known candidate.
    Ignore,
}
