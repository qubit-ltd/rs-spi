// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while building a provider registry from link-time discovery.

use thiserror::Error;

use crate::error::RegistryMutationError;
use crate::inventory::ProviderRegistrationSource;

/// Error raised when a discovered provider cannot be added to a registry.
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum ProviderInventoryBuildError {
    /// A submitted provider conflicts with registry mutation rules.
    #[error("failed to register provider submitted from {source}: {error}")]
    Registration {
        /// Declaration location of the submitted provider.
        source: ProviderRegistrationSource,
        /// Registry mutation failure that rejected the provider.
        #[source]
        error: RegistryMutationError,
    },
}

impl ProviderInventoryBuildError {
    /// Creates an error for a discovered provider rejected during registration.
    ///
    /// # Parameters
    ///
    /// * `source` - Declaration location of the rejected provider.
    /// * `error` - Registry mutation failure that rejected the provider.
    ///
    /// # Returns
    ///
    /// A registration error retaining both the discovery source and mutation
    /// failure.
    #[must_use]
    pub fn registration(source: ProviderRegistrationSource, error: RegistryMutationError) -> Self {
        Self::Registration { source, error }
    }

    /// Returns the declaration location of the registration that failed.
    ///
    /// # Returns
    ///
    /// The source location retained by this error.
    #[must_use]
    pub const fn source_location(&self) -> ProviderRegistrationSource {
        match self {
            Self::Registration { source, .. } => *source,
        }
    }

    /// Returns the registry mutation error that rejected the registration.
    ///
    /// # Returns
    ///
    /// The underlying registration failure retained by this error.
    pub const fn registration_error(&self) -> &RegistryMutationError {
        match self {
            Self::Registration { error, .. } => error,
        }
    }
}
