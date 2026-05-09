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

use crate::{
    ProviderAvailability,
    ProviderRegistryError,
};

/// Factory contract for one service implementation.
///
/// A provider gives a registry stable names, optional aliases, availability
/// checks, a priority used by automatic selection, and a factory method for
/// creating one service instance. The associated `Service` type may be a trait
/// object, such as `dyn MyService`, which allows a registry to select an
/// implementation at runtime while keeping the registry itself strongly typed.
///
/// # Examples
///
/// Implement a provider for a trait-object service and create it through a
/// registry:
///
/// ```rust
/// use std::fmt::Debug;
///
/// use qubit_spi::{
///     ProviderRegistry,
///     ProviderRegistryError,
///     ServiceProvider,
/// };
///
/// trait Encoder: Debug + Send + Sync {
///     fn encode(&self, value: &str) -> String;
/// }
///
/// #[derive(Debug)]
/// struct PlainEncoder;
///
/// impl Encoder for PlainEncoder {
///     fn encode(&self, value: &str) -> String {
///         value.to_owned()
///     }
/// }
///
/// #[derive(Debug)]
/// struct PlainEncoderProvider;
///
/// impl ServiceProvider for PlainEncoderProvider {
///     type Config = ();
///     type Service = dyn Encoder;
///
///     fn id(&self) -> &'static str {
///         "plain"
///     }
///
///     fn aliases(&self) -> &'static [&'static str] {
///         &["identity"]
///     }
///
///     fn create(&self, _config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
///         Ok(Box::new(PlainEncoder))
///     }
/// }
///
/// let mut registry = ProviderRegistry::<dyn Encoder, ()>::new();
/// registry
///     .register(PlainEncoderProvider)
///     .expect("provider id and aliases should be unique");
///
/// let encoder = registry
///     .create("identity", &())
///     .expect("registered provider should create an encoder");
/// assert_eq!("payload", encoder.encode("payload"));
/// ```
///
/// Use `priority` and `availability` to let automatic selection skip an
/// unavailable preferred backend:
///
/// ```rust
/// use std::fmt::Debug;
///
/// use qubit_spi::{
///     ProviderAvailability,
///     ProviderRegistry,
///     ProviderRegistryError,
///     ServiceProvider,
/// };
///
/// #[derive(Debug)]
/// struct CacheConfig {
///     remote_enabled: bool,
/// }
///
/// trait Cache: Debug + Send + Sync {
///     fn backend(&self) -> &'static str;
/// }
///
/// #[derive(Debug)]
/// struct NamedCache(&'static str);
///
/// impl Cache for NamedCache {
///     fn backend(&self) -> &'static str {
///         self.0
///     }
/// }
///
/// #[derive(Debug)]
/// struct MemoryCacheProvider;
///
/// impl ServiceProvider for MemoryCacheProvider {
///     type Config = CacheConfig;
///     type Service = dyn Cache;
///
///     fn id(&self) -> &'static str {
///         "memory"
///     }
///
///     fn priority(&self) -> i32 {
///         10
///     }
///
///     fn create(&self, _config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
///         Ok(Box::new(NamedCache("memory")))
///     }
/// }
///
/// #[derive(Debug)]
/// struct RemoteCacheProvider;
///
/// impl ServiceProvider for RemoteCacheProvider {
///     type Config = CacheConfig;
///     type Service = dyn Cache;
///
///     fn id(&self) -> &'static str {
///         "remote"
///     }
///
///     fn priority(&self) -> i32 {
///         20
///     }
///
///     fn availability(&self, config: &Self::Config) -> ProviderAvailability {
///         if config.remote_enabled {
///             ProviderAvailability::Available
///         } else {
///             ProviderAvailability::unavailable("remote cache is disabled")
///         }
///     }
///
///     fn create(&self, _config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
///         Ok(Box::new(NamedCache("remote")))
///     }
/// }
///
/// let mut registry = ProviderRegistry::<dyn Cache, CacheConfig>::new();
/// registry
///     .register(MemoryCacheProvider)
///     .expect("memory provider should register");
/// registry
///     .register(RemoteCacheProvider)
///     .expect("remote provider should register");
///
/// let cache = registry
///     .create_auto(&CacheConfig {
///         remote_enabled: false,
///     })
///     .expect("memory cache should be selected as fallback");
/// assert_eq!("memory", cache.backend());
/// ```
pub trait ServiceProvider: Debug + Send + Sync {
    /// Configuration type passed to provider checks and factories.
    type Config;

    /// Service type created by this provider.
    type Service: ?Sized + 'static;

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
    /// Returns [`ProviderRegistryError`] when initialization fails. Provider
    /// implementations should use [`ProviderRegistryError::create_failed`] to
    /// translate backend-specific errors into registry errors.
    fn create(&self, config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError>;
}
