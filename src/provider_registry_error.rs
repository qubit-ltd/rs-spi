/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Error type for provider registration and selection.

use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};

use crate::ProviderFailure;

/// Error returned by provider registries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderRegistryError {
    /// A provider id, alias, or selector is empty after trimming.
    EmptyProviderName,
    /// A provider id or alias conflicts with an already registered name.
    DuplicateProviderName {
        /// Conflicting provider name.
        name: String,
    },
    /// No registered provider matches the requested selector.
    UnknownProvider {
        /// Requested provider selector.
        name: String,
    },
    /// The selected provider is not available in the current environment.
    ProviderUnavailable {
        /// Requested provider selector.
        name: String,
        /// Human-readable unavailability reason.
        reason: String,
    },
    /// The selected provider failed while creating a service.
    ProviderCreate {
        /// Requested provider selector.
        name: String,
        /// Human-readable creation failure reason.
        reason: String,
    },
    /// All configured provider candidates failed.
    NoAvailableProvider {
        /// Candidate failures in the order they were tried.
        failures: Vec<ProviderFailure>,
    },
    /// No providers are registered.
    EmptyRegistry,
}

impl ProviderRegistryError {
    /// Creates a provider-creation failure without a provider name.
    ///
    /// Provider implementations can return this error when they do not know the
    /// selector used by the registry. The registry will attach the candidate
    /// provider name before returning the error to callers.
    ///
    /// # Parameters
    /// - `reason`: Human-readable creation failure reason.
    ///
    /// # Returns
    /// Provider creation failure.
    pub fn create_failed(reason: &str) -> Self {
        Self::ProviderCreate {
            name: String::new(),
            reason: reason.to_owned(),
        }
    }

    /// Creates a provider-creation failure for a named provider.
    ///
    /// # Parameters
    /// - `name`: Provider selector or id.
    /// - `reason`: Human-readable creation failure reason.
    ///
    /// # Returns
    /// Provider creation failure.
    pub fn provider_create(name: &str, reason: &str) -> Self {
        Self::ProviderCreate {
            name: name.to_owned(),
            reason: reason.to_owned(),
        }
    }

    /// Attaches a provider name to name-bearing errors that do not already have one.
    ///
    /// # Parameters
    /// - `name`: Provider candidate name being tried by the registry.
    ///
    /// # Returns
    /// Error with provider context preserved or attached.
    pub(crate) fn with_provider_name(self, name: &str) -> Self {
        match self {
            Self::UnknownProvider { name: current } if current.trim().is_empty() => {
                Self::UnknownProvider {
                    name: name.to_owned(),
                }
            }
            Self::ProviderUnavailable {
                name: current,
                reason,
            } if current.trim().is_empty() => Self::ProviderUnavailable {
                name: name.to_owned(),
                reason,
            },
            Self::ProviderCreate {
                name: current,
                reason,
            } if current.trim().is_empty() => Self::ProviderCreate {
                name: name.to_owned(),
                reason,
            },
            error => error,
        }
    }

    /// Converts a provider error into a candidate failure.
    ///
    /// # Parameters
    /// - `name`: Provider candidate name being tried by the registry.
    ///
    /// # Returns
    /// Candidate failure preserving the most useful provider error context.
    pub(crate) fn into_provider_failure(self, name: &str) -> ProviderFailure {
        match self.with_provider_name(name) {
            Self::UnknownProvider { name } => ProviderFailure::unknown(&name),
            Self::ProviderUnavailable { name, reason } => {
                ProviderFailure::unavailable(&name, &reason)
            }
            Self::ProviderCreate { name, reason } => ProviderFailure::create_failed(&name, &reason),
            error => ProviderFailure::create_failed(name, &error.to_string()),
        }
    }
}

impl Display for ProviderRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::EmptyProviderName => formatter.write_str("provider name must not be empty"),
            Self::DuplicateProviderName { name } => {
                write!(formatter, "duplicate provider name: {name}")
            }
            Self::UnknownProvider { name } => {
                write!(formatter, "unknown provider: {name}")
            }
            Self::ProviderUnavailable { name, reason } => {
                write!(formatter, "provider '{name}' is unavailable: {reason}")
            }
            Self::ProviderCreate { name, reason } if name.is_empty() => {
                write!(formatter, "provider failed to create service: {reason}")
            }
            Self::ProviderCreate { name, reason } => {
                write!(
                    formatter,
                    "provider '{name}' failed to create service: {reason}"
                )
            }
            Self::NoAvailableProvider { failures } => {
                let messages: Vec<String> = failures.iter().map(ToString::to_string).collect();
                write!(
                    formatter,
                    "no available provider; candidate failures: {}",
                    messages.join("; "),
                )
            }
            Self::EmptyRegistry => formatter.write_str("provider registry is empty"),
        }
    }
}

impl std::error::Error for ProviderRegistryError {}
