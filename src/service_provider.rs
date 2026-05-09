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
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistryError,
    ServiceSpec,
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
///     ProviderCreateError,
///     ProviderDescriptor,
///     ProviderRegistry,
///     ProviderRegistryError,
///     ServiceProvider,
///     ServiceSpec,
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
/// #[derive(Debug)]
/// struct EncoderSpec;
///
/// impl ServiceSpec for EncoderSpec {
///     type Config = ();
///     type Output = Box<dyn Encoder>;
/// }
///
/// impl ServiceProvider<EncoderSpec> for PlainEncoderProvider {
///     fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
///         ProviderDescriptor::new("plain")?.with_aliases(&["identity"])
///     }
///
///     fn create(&self, _config: &()) -> Result<Box<dyn Encoder>, ProviderCreateError> {
///         Ok(Box::new(PlainEncoder))
///     }
/// }
///
/// let mut registry = ProviderRegistry::<EncoderSpec>::new();
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
///     ProviderCreateError,
///     ProviderDescriptor,
///     ProviderAvailability,
///     ProviderRegistry,
///     ProviderRegistryError,
///     ServiceProvider,
///     ServiceSpec,
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
/// #[derive(Debug)]
/// struct CacheSpec;
///
/// impl ServiceSpec for CacheSpec {
///     type Config = CacheConfig;
///     type Output = Box<dyn Cache>;
/// }
///
/// impl ServiceProvider<CacheSpec> for MemoryCacheProvider {
///     fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
///         Ok(ProviderDescriptor::new("memory")?.with_priority(10))
///     }
///
///     fn create(&self, _config: &CacheConfig) -> Result<Box<dyn Cache>, ProviderCreateError> {
///         Ok(Box::new(NamedCache("memory")))
///     }
/// }
///
/// #[derive(Debug)]
/// struct RemoteCacheProvider;
///
/// impl ServiceProvider<CacheSpec> for RemoteCacheProvider {
///     fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
///         Ok(ProviderDescriptor::new("remote")?.with_priority(20))
///     }
///
///     fn availability(&self, config: &CacheConfig) -> ProviderAvailability {
///         if config.remote_enabled {
///             ProviderAvailability::Available
///         } else {
///             ProviderAvailability::unavailable("remote cache is disabled")
///         }
///     }
///
///     fn create(&self, _config: &CacheConfig) -> Result<Box<dyn Cache>, ProviderCreateError> {
///         Ok(Box::new(NamedCache("remote")))
///     }
/// }
///
/// let mut registry = ProviderRegistry::<CacheSpec>::new();
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
pub trait ServiceProvider<Spec>: Debug + Send + Sync
where
    Spec: ServiceSpec,
{
    /// Gets stable provider metadata.
    ///
    /// # Returns
    /// Provider descriptor used by registries for name lookup and automatic
    /// selection.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when the provider id or aliases are
    /// invalid.
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError>;

    /// Checks whether this provider can create a service.
    ///
    /// # Parameters
    /// - `config`: Service configuration used for provider-specific checks.
    ///
    /// # Returns
    /// Provider availability in the current runtime environment.
    fn availability(&self, _config: &Spec::Config) -> ProviderAvailability {
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
    /// Returns [`ProviderCreateError`] when initialization fails. Registries
    /// translate this provider-level error into [`ProviderRegistryError`] with
    /// provider-name context.
    fn create(&self, config: &Spec::Config) -> Result<Spec::Output, ProviderCreateError>;
}
