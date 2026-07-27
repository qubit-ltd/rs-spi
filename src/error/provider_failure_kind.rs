// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fallback classifications reported by provider construction.

/// Classification of a failure reported while a provider creates a service.
///
/// Resolver fallback uses this classification while the associated domain
/// error remains in ProviderFailure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ProviderFailureKind {
    /// The provider does not support this otherwise valid request.
    Unsupported,
    /// The provider cannot run in the current environment.
    Unavailable,
    /// The provider-specific configuration is invalid.
    InvalidConfiguration,
    /// Provider initialization failed after accepting the request.
    InitializationFailed,
}

impl ProviderFailureKind {
    /// Reports whether this failure denotes provider absence.
    ///
    /// # Returns
    ///
    /// True for unsupported and unavailable providers.
    #[inline(always)]
    #[must_use]
    pub const fn is_absence(self) -> bool {
        matches!(self, Self::Unsupported | Self::Unavailable)
    }
}
