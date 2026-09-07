// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-bearing asynchronous provider definitions.

use crate::AsyncServiceProvider;
use crate::AsyncServiceSpec;
use crate::ProviderMetadata;

/// Marker combining asynchronous creation with registration metadata.
///
/// # Type Parameters
///
/// * `S` - Asynchronous service family implemented by the provider.
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, AsyncServiceSpec, AsyncServiceProvider, ProviderFuture};
/// use qubit_spi::{ProviderDescriptor, ProviderId, ProviderMetadata};
/// use qubit_spi::error::ProviderFailure;
/// struct Spec;
/// impl ServiceSpec for Spec { type Config = String; type Error = std::io::Error; }
/// impl AsyncServiceSpec for Spec { type Output = String; }
/// struct Echo;
/// impl ProviderMetadata for Echo {
///     fn descriptor(&self) -> ProviderDescriptor {
///         ProviderDescriptor::new(ProviderId::new("echo").expect("valid static ID"))
///     }
/// }
/// impl AsyncServiceProvider<Spec> for Echo {
///     fn create_configured<'a>(&'a self, config: &'a String)
///         -> ProviderFuture<'a, Result<String, ProviderFailure<std::io::Error>>> {
///         Box::pin(async move { Ok(config.clone()) })
///     }
/// }
/// let provider: std::sync::Arc<dyn qubit_spi::AsyncProviderDefinition<Spec>> = std::sync::Arc::new(Echo);
/// assert_eq!("echo", provider.descriptor().id().as_str());
/// assert_eq!("hello", futures::executor::block_on(provider.create_configured(&"hello".to_owned()))?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait AsyncProviderDefinition<S>: ProviderMetadata + AsyncServiceProvider<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    // empty
}

impl<S, T> AsyncProviderDefinition<S> for T
where
    S: AsyncServiceSpec,
    S::Config: Sync,
    T: ProviderMetadata + AsyncServiceProvider<S> + ?Sized,
{
    // empty
}
