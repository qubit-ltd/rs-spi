// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Self-described providers accepted by a provider registry.

use crate::ProviderMetadata;
use crate::ServiceProvider;
use crate::SyncServiceSpec;

/// Marker combining synchronous creation with registration metadata.
///
/// Every type implementing both [`ProviderMetadata`] and
/// [`ServiceProvider<S>`] automatically implements this trait.
///
/// # Type Parameters
///
/// * `S` - Synchronous service family implemented by the provider.
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
///         Ok(config.clone())
///     }
/// }
/// let provider: std::sync::Arc<dyn qubit_spi::ProviderDefinition<Spec>> = std::sync::Arc::new(Echo);
/// assert_eq!("echo", provider.descriptor().id().as_str());
/// assert_eq!("hello", provider.create_configured(&"hello".to_owned())?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait ProviderDefinition<S>: ProviderMetadata + ServiceProvider<S>
where
    S: SyncServiceSpec,
{
    // empty
}

impl<S, T> ProviderDefinition<S> for T
where
    S: SyncServiceSpec,
    T: ProviderMetadata + ServiceProvider<S> + ?Sized,
{
    // empty
}
