// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reasons provider creation ended without producing a service.

/// Describes why candidate traversal ended unsuccessfully.
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, SyncServiceSpec};
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = String;
///     type Error = std::io::Error;
/// }
/// impl SyncServiceSpec for Spec { type Output = String; }
/// use qubit_spi::{ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider};
/// use qubit_spi::error::ProviderFailure;
/// struct Echo;
/// impl ProviderMetadata for Echo {
///     fn descriptor(&self) -> ProviderDescriptor {
///         ProviderDescriptor::new(ProviderId::new("echo").expect("valid static ID"))
///     }
/// }
/// impl ServiceProvider<Spec> for Echo {
///     fn create_configured(&self, config: &String) -> Result<String, ProviderFailure<std::io::Error>> {
///         Err(ProviderFailure::unavailable(std::io::Error::other(config.clone())))
///     }
/// }
/// let registry = qubit_spi::ProviderRegistry::<Spec>::default();
/// registry.register(Echo)?;
/// let error = registry.resolve()?.create_configured(&"offline".to_owned()).expect_err("provider is unavailable");
/// // A failed single-candidate selection is exhausted even with no fallback.
/// assert_eq!(qubit_spi::ProviderCreationTermination::Exhausted, error.termination());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderCreationTermination {
    /// Every candidate admitted by the selection was attempted.
    Exhausted,
    /// Fallback policy rejected continuing after the terminal failure.
    StoppedByPolicy,
}
