// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reasons provider creation ended without producing a service.

/// Describes why candidate traversal ended unsuccessfully.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderCreationTermination {
    /// Every candidate admitted by the selection was attempted.
    Exhausted,
    /// Fallback policy rejected continuing after the terminal failure.
    StoppedByPolicy,
}
