// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider contract for pluggable service implementations.

use crate::{ProviderError, ServiceSpec};

/// Factory contract for one pluggable service implementation.
///
/// Registration owns provider identity and selection metadata. A provider only
/// creates the output handle selected by its service specification. Implement
/// this trait when an application needs to provide one selectable backend for
/// a [`ServiceSpec`] family.
pub trait ServiceProvider<S>: Send + Sync + 'static
where
    S: ServiceSpec,
{
    /// Creates one service output from the supplied configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Service-family configuration declared by `S`.
    ///
    /// # Returns
    ///
    /// The complete `S::Output` handle.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the provider cannot create the requested
    /// service. Its classification must accurately reflect whether fallback is
    /// appropriate.
    fn create(&self, config: &S::Config) -> Result<S::Output, ProviderError>;
}
