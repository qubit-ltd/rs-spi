/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Provider-level service creation errors.

use thiserror::Error;

/// Error returned by one provider while creating a service.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ProviderCreateError {
    /// The provider discovered at creation time that it cannot be used.
    #[error("provider is unavailable: {reason}")]
    Unavailable {
        /// Human-readable unavailability reason.
        reason: String,
    },
    /// The provider failed while creating the service.
    #[error("provider failed to create service: {reason}")]
    Failed {
        /// Human-readable failure reason.
        reason: String,
    },
}

impl ProviderCreateError {
    /// Creates an unavailable-provider error.
    ///
    /// # Parameters
    /// - `reason`: Human-readable unavailability reason.
    ///
    /// # Returns
    /// Provider creation error.
    pub fn unavailable(reason: &str) -> Self {
        Self::Unavailable {
            reason: reason.to_owned(),
        }
    }

    /// Creates a provider failure error.
    ///
    /// # Parameters
    /// - `reason`: Human-readable failure reason.
    ///
    /// # Returns
    /// Provider creation error.
    pub fn failed(reason: &str) -> Self {
        Self::Failed {
            reason: reason.to_owned(),
        }
    }
}
