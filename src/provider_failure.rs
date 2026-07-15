// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Candidate failure details collected during fallback selection.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::{ProviderCreateError, ProviderName, ProviderRegistryError};

/// Failure recorded for one provider candidate.
#[derive(Debug, Clone)]
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
        /// Provider-level unavailability error.
        source: ProviderCreateError,
    },
    /// A provider matched the candidate name but failed while creating a
    /// service.
    CreateFailed {
        /// Candidate provider name.
        name: ProviderName,
        /// Provider-level creation error.
        source: ProviderCreateError,
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
    #[inline]
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
    #[inline]
    pub fn unavailable(name: &str, reason: &str) -> Result<Self, ProviderRegistryError> {
        Self::unavailable_from_error(name, ProviderCreateError::unavailable(reason))
    }

    /// Creates a provider-creation failure.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `reason`: Human-readable creation failure reason.
    ///
    /// # Returns
    /// Provider-creation failure.
    #[inline]
    pub fn create_failed(name: &str, reason: &str) -> Result<Self, ProviderRegistryError> {
        Self::create_failed_from_error(name, ProviderCreateError::failed(reason))
    }

    /// Creates an unavailable-provider failure from a provider-level error.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `source`: Provider-level unavailability error.
    ///
    /// # Returns
    /// Unavailable-provider failure preserving the source error.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when `name` is not valid.
    #[inline]
    pub fn unavailable_from_error(
        name: &str,
        source: ProviderCreateError,
    ) -> Result<Self, ProviderRegistryError> {
        Ok(Self::unavailable_error(ProviderName::new(name)?, source))
    }

    /// Creates a provider-creation failure from a provider-level error.
    ///
    /// # Parameters
    /// - `name`: Candidate provider name.
    /// - `source`: Provider-level creation error.
    ///
    /// # Returns
    /// Provider-creation failure preserving the source error.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when `name` is not valid.
    #[inline]
    pub fn create_failed_from_error(
        name: &str,
        source: ProviderCreateError,
    ) -> Result<Self, ProviderRegistryError> {
        Ok(Self::create_failed_error(ProviderName::new(name)?, source))
    }

    /// Gets the candidate provider name.
    ///
    /// # Returns
    /// Candidate name associated with this failure.
    #[inline]
    pub fn name(&self) -> &str {
        self.provider_name().as_str()
    }

    /// Gets the candidate provider name.
    ///
    /// # Returns
    /// Candidate name associated with this failure.
    #[inline]
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
    #[inline]
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
    #[inline]
    pub(crate) fn unavailable_name(name: ProviderName, reason: &str) -> Self {
        Self::unavailable_error(name, ProviderCreateError::unavailable(reason))
    }

    /// Creates an unavailable-provider failure from a provider error.
    ///
    /// # Parameters
    /// - `name`: Validated candidate provider name.
    /// - `source`: Provider-level unavailability error.
    ///
    /// # Returns
    /// Unavailable-provider failure.
    #[inline]
    pub(crate) fn unavailable_error(name: ProviderName, source: ProviderCreateError) -> Self {
        Self::Unavailable { name, source }
    }

    /// Creates a creation failure from a provider error.
    ///
    /// # Parameters
    /// - `name`: Validated candidate provider name.
    /// - `source`: Provider-level creation error.
    ///
    /// # Returns
    /// Provider-creation failure.
    #[inline]
    pub(crate) fn create_failed_error(name: ProviderName, source: ProviderCreateError) -> Self {
        Self::CreateFailed { name, source }
    }
}

impl Display for ProviderFailure {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::UnknownProvider { name } => {
                write!(formatter, "unknown provider: {name}")
            }
            Self::Unavailable { name, source } => {
                write!(
                    formatter,
                    "provider '{name}' is unavailable: {}",
                    source.reason(),
                )
            }
            Self::CreateFailed { name, source } => {
                write!(
                    formatter,
                    "provider '{name}' failed to create service: {}",
                    source.reason(),
                )
            }
        }
    }
}

impl Error for ProviderFailure {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnknownProvider { .. } => None,
            Self::Unavailable { source, .. } | Self::CreateFailed { source, .. } => Some(source),
        }
    }
}
