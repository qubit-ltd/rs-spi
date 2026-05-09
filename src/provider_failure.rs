/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Candidate failure details collected during fallback selection.

use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};

/// Failure recorded for one provider candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFailure {
    /// No provider matched the candidate name.
    UnknownProvider {
        /// Candidate provider name.
        name: String,
    },
    /// A provider matched the candidate name but is unavailable.
    Unavailable {
        /// Candidate provider name.
        name: String,
        /// Human-readable unavailability reason.
        reason: String,
    },
    /// A provider matched the candidate name but failed while creating a service.
    CreateFailed {
        /// Candidate provider name.
        name: String,
        /// Human-readable creation failure reason.
        reason: String,
    },
}

impl ProviderFailure {
    /// Creates an unknown-provider failure.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    ///
    /// # Returns
    /// Unknown-provider failure.
    pub fn unknown(name: &str) -> Self {
        Self::UnknownProvider {
            name: name.to_owned(),
        }
    }

    /// Creates an unavailable-provider failure.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `reason`: Human-readable unavailability reason.
    ///
    /// # Returns
    /// Unavailable-provider failure.
    pub fn unavailable(name: &str, reason: &str) -> Self {
        Self::Unavailable {
            name: name.to_owned(),
            reason: reason.to_owned(),
        }
    }

    /// Creates a provider-creation failure.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `reason`: Human-readable creation failure reason.
    ///
    /// # Returns
    /// Provider-creation failure.
    pub fn create_failed(name: &str, reason: &str) -> Self {
        Self::CreateFailed {
            name: name.to_owned(),
            reason: reason.to_owned(),
        }
    }

    /// Gets the candidate provider name.
    ///
    /// # Returns
    /// Candidate name associated with this failure.
    pub fn name(&self) -> &str {
        match self {
            Self::UnknownProvider { name }
            | Self::Unavailable { name, .. }
            | Self::CreateFailed { name, .. } => name,
        }
    }
}

impl Display for ProviderFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::UnknownProvider { name } => {
                write!(formatter, "unknown provider: {name}")
            }
            Self::Unavailable { name, reason } => {
                write!(formatter, "provider '{name}' is unavailable: {reason}")
            }
            Self::CreateFailed { name, reason } => {
                write!(
                    formatter,
                    "provider '{name}' failed to create service: {reason}"
                )
            }
        }
    }
}
