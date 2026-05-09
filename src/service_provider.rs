/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Provider contract for pluggable service implementations.

use std::fmt::Debug;

use crate::ProviderAvailability;

/// Factory contract for one service implementation.
///
/// A provider gives a registry stable names, optional aliases, availability
/// checks, a priority used by automatic selection, and a factory method for
/// creating one service instance. The associated `Service` type may be a trait
/// object, such as `dyn MyService`, which allows a registry to select an
/// implementation at runtime while keeping the registry itself strongly typed.
pub trait ServiceProvider: Debug + Send + Sync {
    /// Configuration type passed to provider checks and factories.
    type Config;

    /// Service type created by this provider.
    type Service: ?Sized + 'static;

    /// Provider-specific creation error.
    type Error;

    /// Gets the canonical provider identifier.
    ///
    /// # Returns
    /// Stable provider identifier. Identifiers should be lowercase ASCII.
    fn id(&self) -> &'static str;

    /// Gets additional names accepted for this provider.
    ///
    /// # Returns
    /// Alias names. Registry matching is case-insensitive.
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    /// Gets provider priority used by automatic selection.
    ///
    /// # Returns
    /// Priority value. Larger values are preferred.
    fn priority(&self) -> i32 {
        0
    }

    /// Checks whether this provider can create a service.
    ///
    /// # Parameters
    /// - `config`: Service configuration used for provider-specific checks.
    ///
    /// # Returns
    /// Provider availability in the current runtime environment.
    fn availability(&self, _config: &Self::Config) -> ProviderAvailability {
        ProviderAvailability::Available
    }

    /// Creates a service instance.
    ///
    /// # Parameters
    /// - `config`: Service configuration used to initialize the implementation.
    ///
    /// # Returns
    /// Boxed service implementation.
    ///
    /// # Errors
    /// Returns provider-specific errors when initialization fails.
    fn create(&self, config: &Self::Config) -> Result<Box<Self::Service>, Self::Error>;
}
