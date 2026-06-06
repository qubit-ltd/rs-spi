// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Runtime availability state for service providers.

/// Availability of a provider in the current runtime environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderAvailability {
    /// The provider can create service instances.
    Available,
    /// The provider cannot create service instances.
    Unavailable {
        /// Human-readable reason explaining why the provider is unavailable.
        reason: String,
    },
}

impl ProviderAvailability {
    /// Creates an unavailable state with a human-readable reason.
    ///
    /// # Parameters
    /// - `reason`: Reason shown to callers when the provider cannot be used.
    ///
    /// # Returns
    /// Unavailable provider state.
    #[inline]
    pub fn unavailable(reason: &str) -> Self {
        Self::Unavailable {
            reason: reason.to_owned(),
        }
    }

    /// Tells whether this state allows service creation.
    ///
    /// # Returns
    /// `true` when the provider is available.
    #[inline]
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}
