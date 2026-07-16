// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reasons provider resolution ended without creating a service.

/// Describes why candidate traversal ended unsuccessfully.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionTermination {
    /// Every candidate admitted by the selection was considered.
    Exhausted,
    /// Fallback policy rejected continuing after the terminal failure.
    StoppedByPolicy,
}
