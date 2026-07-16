// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stable classifications for individual failed provider attempts.

/// Classification of one failed resolution attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AttemptFailureKind {
    /// A selector matched no registered provider.
    UnknownProvider,
    /// A selected provider returned a classified creation error.
    ProviderError,
}
