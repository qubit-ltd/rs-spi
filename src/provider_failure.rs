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

use crate::{
    ProviderName,
    ProviderRegistryError,
};

/// Failure recorded for one provider candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFailure {
    /// No provider matched the candidate name.
    UnknownProvider {
        /// Candidate provider name.
        name: ProviderName,
    },
    /// A provider matched the candidate name but is unavailable.
    Unavailable {
        /// Candidate provider name.
        name: ProviderName,
        /// Human-readable unavailability reason.
        reason: String,
    },
    /// A provider matched the candidate name but failed while creating a service.
    CreateFailed {
        /// Candidate provider name.
        name: ProviderName,
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
    pub fn unknown(name: &str) -> Result<Self, ProviderRegistryError> {
        Ok(Self::unknown_name(ProviderName::new(name)?))
    }

    /// Creates an unavailable-provider failure.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `reason`: Human-readable unavailability reason.
    ///
    /// # Returns
    /// Unavailable-provider failure.
    pub fn unavailable(name: &str, reason: &str) -> Result<Self, ProviderRegistryError> {
        Ok(Self::unavailable_name(ProviderName::new(name)?, reason))
    }

    /// Creates a provider-creation failure.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `reason`: Human-readable creation failure reason.
    ///
    /// # Returns
    /// Provider-creation failure.
    pub fn create_failed(name: &str, reason: &str) -> Result<Self, ProviderRegistryError> {
        Ok(Self::create_failed_name(ProviderName::new(name)?, reason))
    }

    /// Gets the candidate provider name.
    ///
    /// # Returns
    /// Candidate name associated with this failure.
    pub fn name(&self) -> &str {
        self.provider_name().as_str()
    }

    /// Gets the candidate provider name.
    ///
    /// # Returns
    /// Candidate name associated with this failure.
    pub fn provider_name(&self) -> &ProviderName {
        match self {
            Self::UnknownProvider { name }
            | Self::Unavailable { name, .. }
            | Self::CreateFailed { name, .. } => name,
        }
    }

    /// Creates an unknown-provider failure from a validated provider name.
    ///
    /// # Parameters
    /// - `name`: Validated candidate provider name.
    ///
    /// # Returns
    /// Unknown-provider failure.
    pub(crate) fn unknown_name(name: ProviderName) -> Self {
        Self::UnknownProvider { name }
    }

    /// Creates an unavailable-provider failure from a validated provider name.
    ///
    /// # Parameters
    /// - `name`: Validated candidate provider name.
    /// - `reason`: Human-readable unavailability reason.
    ///
    /// # Returns
    /// Unavailable-provider failure.
    pub(crate) fn unavailable_name(name: ProviderName, reason: &str) -> Self {
        Self::Unavailable {
            name,
            reason: reason.to_owned(),
        }
    }

    /// Creates a creation failure from a validated provider name.
    ///
    /// # Parameters
    /// - `name`: Validated candidate provider name.
    /// - `reason`: Human-readable creation failure reason.
    ///
    /// # Returns
    /// Provider-creation failure.
    pub(crate) fn create_failed_name(name: ProviderName, reason: &str) -> Self {
        Self::CreateFailed {
            name,
            reason: reason.to_owned(),
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
