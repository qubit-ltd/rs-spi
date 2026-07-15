// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Classifications for provider registration conflicts.

/// Classification of a provider registration conflict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistrationErrorKind {
    /// A canonical ID or alias is already owned by another registration.
    DuplicateSelector,
}
