// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classifications for individual provider resolution failures.

/// Classification of one failed resolution attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AttemptFailureKind {
    /// Selector lookup reached no provider.
    UnknownProvider,
    /// A resolved provider failed to create its service.
    ProviderError,
}
