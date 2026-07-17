// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Error classifications reported by provider service construction.

/// Classification of a failure reported while a provider creates a service.
///
/// Leaf providers return these variants so [`crate::ResolvingServiceProvider`]
/// can decide whether its fallback policy permits another provider attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderErrorKind {
    /// The provider does not support this request or configuration.
    Unsupported,
    /// The provider cannot run in the current environment.
    Unavailable,
    /// The provider-specific configuration is invalid.
    InvalidConfiguration,
    /// Provider initialization failed unexpectedly.
    InitializationFailed,
}
