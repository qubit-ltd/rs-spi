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

use std::error::Error;
use std::fmt::{
    Display,
    Formatter,
    Result as FmtResult,
};

use crate::ProviderFailure;

/// Error returned by provider registries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderRegistryError<E> {
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
        /// Provider-specific creation error.
        error: E,
    },
    /// All configured provider candidates failed.
    NoAvailableProvider {
        /// Candidate failures in the order they were tried.
        failures: Vec<ProviderFailure<E>>,
    },
    /// No providers are registered.
    EmptyRegistry,
}

impl<E> Display for ProviderRegistryError<E>
where
    E: Display,
{
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
            Self::ProviderCreate { name, error } => {
                write!(
                    formatter,
                    "provider '{name}' failed to create service: {error}"
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

impl<E> Error for ProviderRegistryError<E> where E: Error + 'static {}
