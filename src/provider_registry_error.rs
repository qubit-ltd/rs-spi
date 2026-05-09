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

use thiserror::Error;

use crate::{
    ProviderFailure,
    ProviderName,
};

/// Error returned by provider registries.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ProviderRegistryError {
    /// A provider id, alias, or selector is empty after trimming.
    #[error("provider name must not be empty")]
    EmptyProviderName,
    /// A provider name contains unsupported characters.
    #[error("invalid provider name '{name}': {reason}")]
    InvalidProviderName {
        /// Invalid provider name after trimming.
        name: String,
        /// Human-readable validation failure reason.
        reason: String,
    },
    /// A provider id or alias conflicts with an already registered name.
    #[error("duplicate provider name: {name}")]
    DuplicateProviderName {
        /// Conflicting provider name.
        name: ProviderName,
    },
    /// No registered provider matches the requested selector.
    #[error("unknown provider: {name}")]
    UnknownProvider {
        /// Requested provider selector.
        name: ProviderName,
    },
    /// The selected provider is not available in the current environment.
    #[error("provider '{name}' is unavailable: {reason}")]
    ProviderUnavailable {
        /// Requested provider selector.
        name: ProviderName,
        /// Human-readable unavailability reason.
        reason: String,
    },
    /// The selected provider failed while creating a service.
    #[error("provider '{name}' failed to create service: {reason}")]
    ProviderCreate {
        /// Requested provider selector.
        name: ProviderName,
        /// Human-readable creation failure reason.
        reason: String,
    },
    /// All configured provider candidates failed.
    #[error(
        "no available provider; candidate failures: {}",
        format_provider_failures(.failures)
    )]
    NoAvailableProvider {
        /// Candidate failures in the order they were tried.
        failures: Vec<ProviderFailure>,
    },
    /// No providers are registered.
    #[error("provider registry is empty")]
    EmptyRegistry,
}

/// Formats ordered fallback candidate failures.
///
/// # Parameters
/// - `failures`: Candidate failures in the order they were tried.
///
/// # Returns
/// Candidate failure messages joined by `; `.
fn format_provider_failures(failures: &[ProviderFailure]) -> String {
    failures
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}
