// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Macros for declaring and submitting provider inventories.

/// Declares one synchronous provider inventory for a concrete service family.
///
/// The generated module exposes a `build_registry` function and a hidden entry
/// type used by the `submit_sync_provider!` macro.
///
/// A single-segment `Spec` is resolved in the declaring module. For a local
/// multi-segment path, use `self::model::Spec`, `super::model::Spec`, or
/// `crate::model::Spec`. A plain multi-segment path such as
/// `service_model::Spec` is resolved through the external crate prelude; use
/// `::service_model::Spec` when an absolute external path is preferred.
///
/// # Examples
///
/// ```
/// # mod example {
/// # use std::convert::Infallible;
/// # use qubit_spi::{ServiceSpec, SyncServiceSpec};
/// # pub struct ExampleSpec;
/// # impl ServiceSpec for ExampleSpec {
/// #     type Config = ();
/// #     type Error = Infallible;
/// # }
/// # impl SyncServiceSpec for ExampleSpec {
/// #     type Output = ();
/// # }
/// qubit_spi::declare_sync_provider_inventory! {
///     pub mod example_providers {
///         spec = ExampleSpec;
///     }
/// }
///
/// # pub fn run() {
/// #     assert!(example_providers::build_registry().unwrap().is_empty());
/// # }
/// # }
/// # example::run();
/// ```
#[allow(clippy::crate_in_macro_def)] // Preserves the declaration crate's explicit `crate::` path.
#[macro_export]
macro_rules! declare_sync_provider_inventory {
    (
        $visibility:vis mod $module:ident {
            spec = self::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_sync_provider_inventory! {
            @build $visibility mod $module {
                spec = super::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = super::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_sync_provider_inventory! {
            @build $visibility mod $module {
                spec = super::super::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = crate::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_sync_provider_inventory! {
            @build $visibility mod $module {
                spec = crate::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = ::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_sync_provider_inventory! {
            @build $visibility mod $module {
                spec = ::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = $spec:ident;
        }
    ) => {
        $crate::declare_sync_provider_inventory! {
            @build $visibility mod $module {
                spec = super::$spec;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = $($spec:ident)::+;
        }
    ) => {
        $crate::declare_sync_provider_inventory! {
            @build $visibility mod $module {
                spec = $($spec)::+;
            }
        }
    };
    (
        @build $visibility:vis mod $module:ident {
            spec = $spec:path;
        }
    ) => {
        #[doc = "Link-time provider inventory for one concrete service family."]
        $visibility mod $module {
            #[doc(hidden)]
            pub struct Entry($crate::__private::SyncProviderInventoryEntry<$spec>);

            impl Entry {
                /// Creates an inventory entry submitted by a provider crate.
                #[doc(hidden)]
                #[must_use]
                pub const fn __new(
                    factory: fn() -> ::std::sync::Arc<dyn $crate::ProviderDefinition<$spec>>,
                    source: $crate::ProviderRegistrationSource,
                ) -> Self {
                    Self($crate::__private::SyncProviderInventoryEntry::new(factory, source))
                }
            }

            $crate::__private::inventory::collect!(Entry);

            /// Builds a new unsealed registry from providers linked into this inventory.
            ///
            /// # Returns
            ///
            /// An unsealed registry ready for application-specific registrations.
            ///
            /// # Errors
            ///
            /// Returns an error that identifies the submitted provider when registration
            /// conflicts with an earlier discovered provider.
            ///
            /// # Panics
            ///
            /// Propagates panics raised by provider factories or descriptor callbacks.
            pub fn build_registry(
            ) -> Result<
                $crate::ProviderRegistry<$spec>,
                $crate::error::ProviderInventoryBuildError,
            > {
                let entries = $crate::__private::inventory::iter::<Entry>
                    .into_iter()
                    .map(|entry| &entry.0);
                $crate::__private::build_sync_registry(entries)
            }

            /// Builds a registry after transforming each discovered provider.
            ///
            /// The transform runs in stable declaration-source order after
            /// each provider factory and before registration. Service-family
            /// registries can use it to install validation or other adapters.
            ///
            /// # Parameters
            ///
            /// * `transform` - Adapter applied to each provider before it is
            ///   inserted into the registry.
            ///
            /// # Returns
            ///
            /// An unsealed registry containing every successfully transformed
            /// provider.
            ///
            /// # Errors
            ///
            /// Returns an error identifying the source of a provider whose
            /// transformed descriptor conflicts with an earlier entry.
            ///
            /// # Panics
            ///
            /// Propagates panics raised by provider factories, the transform,
            /// or provider descriptor callbacks.
            pub fn build_registry_with(
                transform: impl FnMut(
                    ::std::sync::Arc<dyn $crate::ProviderDefinition<$spec>>,
                ) -> ::std::sync::Arc<dyn $crate::ProviderDefinition<$spec>>,
            ) -> Result<
                $crate::ProviderRegistry<$spec>,
                $crate::error::ProviderInventoryBuildError,
            > {
                let entries = $crate::__private::inventory::iter::<Entry>
                    .into_iter()
                    .map(|entry| &entry.0);
                $crate::__private::build_sync_registry_with(entries, transform)
            }
        }
    };
}

/// Submits a synchronous provider factory to a declared inventory.
///
/// The provider expression is evaluated only when the inventory builds its
/// registry. It must not capture local state because it is compiled into a
/// zero-argument factory.
///
/// # Examples
///
/// ```
/// # mod example {
/// # use std::convert::Infallible;
/// # use qubit_spi::error::ProviderFailure;
/// # use qubit_spi::{ProviderMetadata, ServiceProvider, ServiceSpec, SyncServiceSpec};
/// # pub struct ExampleSpec;
/// # impl ServiceSpec for ExampleSpec {
/// #     type Config = ();
/// #     type Error = Infallible;
/// # }
/// # impl SyncServiceSpec for ExampleSpec {
/// #     type Output = ();
/// # }
/// # struct ExampleProvider;
/// # impl ProviderMetadata for ExampleProvider {
/// #     fn descriptor(&self) -> qubit_spi::ProviderDescriptor {
/// #         qubit_spi::provider_descriptor!("example")
/// #     }
/// # }
/// # impl ServiceProvider<ExampleSpec> for ExampleProvider {
/// #     fn create_configured(&self, _config: &()) -> Result<(), ProviderFailure<Infallible>> {
/// #         Ok(())
/// #     }
/// # }
/// # qubit_spi::declare_sync_provider_inventory! {
/// #     pub mod example_providers {
/// #         spec = ExampleSpec;
/// #     }
/// # }
/// qubit_spi::submit_sync_provider! {
///     inventory_entry = example_providers::Entry;
///     spec = ExampleSpec;
///     provider = ExampleProvider;
/// }
///
/// # pub fn run() {
/// #     assert_eq!(1, example_providers::build_registry().unwrap().len());
/// # }
/// # }
/// # example::run();
/// ```
#[macro_export]
macro_rules! submit_sync_provider {
    (
        inventory_entry = $inventory_entry:path;
        spec = $spec:path;
        provider = $provider:expr;
    ) => {
        const _: () = {
            $crate::__private::inventory::submit! {
                <$inventory_entry>::__new(
                    || -> ::std::sync::Arc<dyn $crate::ProviderDefinition<$spec>> {
                        ::std::sync::Arc::new($provider)
                    },
                    $crate::ProviderRegistrationSource::new(
                        env!("CARGO_PKG_NAME"),
                        module_path!(),
                        file!(),
                        line!(),
                    ),
                )
            }
        };
    };
}

/// Declares one asynchronous provider inventory for a concrete service family.
///
/// The generated module exposes a `build_registry` function and a hidden entry
/// type used by the `submit_async_provider!` macro.
///
/// A single-segment `Spec` is resolved in the declaring module. For a local
/// multi-segment path, use `self::model::Spec`, `super::model::Spec`, or
/// `crate::model::Spec`. A plain multi-segment path such as
/// `service_model::Spec` is resolved through the external crate prelude; use
/// `::service_model::Spec` when an absolute external path is preferred.
///
/// # Examples
///
/// ```
/// # mod example {
/// # use std::convert::Infallible;
/// # use qubit_spi::{AsyncServiceSpec, ServiceSpec};
/// # pub struct ExampleSpec;
/// # impl ServiceSpec for ExampleSpec {
/// #     type Config = ();
/// #     type Error = Infallible;
/// # }
/// # impl AsyncServiceSpec for ExampleSpec {
/// #     type Output = ();
/// # }
/// qubit_spi::declare_async_provider_inventory! {
///     pub mod example_providers {
///         spec = ExampleSpec;
///     }
/// }
///
/// # pub fn run() {
/// #     assert!(example_providers::build_registry().unwrap().is_empty());
/// # }
/// # }
/// # example::run();
/// ```
#[allow(clippy::crate_in_macro_def)] // Preserves the declaration crate's explicit `crate::` path.
#[macro_export]
macro_rules! declare_async_provider_inventory {
    (
        $visibility:vis mod $module:ident {
            spec = self::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_async_provider_inventory! {
            @build $visibility mod $module {
                spec = super::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = super::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_async_provider_inventory! {
            @build $visibility mod $module {
                spec = super::super::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = crate::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_async_provider_inventory! {
            @build $visibility mod $module {
                spec = crate::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = ::$($spec:ident)::+;
        }
    ) => {
        $crate::declare_async_provider_inventory! {
            @build $visibility mod $module {
                spec = ::$($spec)::+;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = $spec:ident;
        }
    ) => {
        $crate::declare_async_provider_inventory! {
            @build $visibility mod $module {
                spec = super::$spec;
            }
        }
    };
    (
        $visibility:vis mod $module:ident {
            spec = $($spec:ident)::+;
        }
    ) => {
        $crate::declare_async_provider_inventory! {
            @build $visibility mod $module {
                spec = $($spec)::+;
            }
        }
    };
    (
        @build $visibility:vis mod $module:ident {
            spec = $spec:path;
        }
    ) => {
        #[doc = "Link-time provider inventory for one concrete service family."]
        $visibility mod $module {
            #[doc(hidden)]
            pub struct Entry($crate::__private::AsyncProviderInventoryEntry<$spec>);

            impl Entry {
                /// Creates an inventory entry submitted by a provider crate.
                #[doc(hidden)]
                #[must_use]
                pub const fn __new(
                    factory: fn() -> ::std::sync::Arc<dyn $crate::AsyncProviderDefinition<$spec>>,
                    source: $crate::ProviderRegistrationSource,
                ) -> Self {
                    Self($crate::__private::AsyncProviderInventoryEntry::new(factory, source))
                }
            }

            $crate::__private::inventory::collect!(Entry);

            /// Builds a new unsealed registry from providers linked into this inventory.
            ///
            /// # Returns
            ///
            /// An unsealed registry ready for application-specific registrations.
            ///
            /// # Errors
            ///
            /// Returns an error that identifies the submitted provider when registration
            /// conflicts with an earlier discovered provider.
            ///
            /// # Panics
            ///
            /// Propagates panics raised by provider factories or descriptor callbacks.
            pub fn build_registry(
            ) -> Result<
                $crate::AsyncProviderRegistry<$spec>,
                $crate::error::ProviderInventoryBuildError,
            > {
                let entries = $crate::__private::inventory::iter::<Entry>
                    .into_iter()
                    .map(|entry| &entry.0);
                $crate::__private::build_async_registry(entries)
            }

            /// Builds a registry after transforming each discovered provider.
            ///
            /// The transform runs in stable declaration-source order after
            /// each provider factory and before registration. Service-family
            /// registries can use it to install validation or other adapters.
            /// This method never creates asynchronous services.
            ///
            /// # Parameters
            ///
            /// * `transform` - Adapter applied to each provider before it is
            ///   inserted into the registry.
            ///
            /// # Returns
            ///
            /// An unsealed registry containing every successfully transformed
            /// provider.
            ///
            /// # Errors
            ///
            /// Returns an error identifying the source of a provider whose
            /// transformed descriptor conflicts with an earlier entry.
            ///
            /// # Panics
            ///
            /// Propagates panics raised by provider factories, the transform,
            /// or provider descriptor callbacks.
            pub fn build_registry_with(
                transform: impl FnMut(
                    ::std::sync::Arc<dyn $crate::AsyncProviderDefinition<$spec>>,
                ) -> ::std::sync::Arc<dyn $crate::AsyncProviderDefinition<$spec>>,
            ) -> Result<
                $crate::AsyncProviderRegistry<$spec>,
                $crate::error::ProviderInventoryBuildError,
            > {
                let entries = $crate::__private::inventory::iter::<Entry>
                    .into_iter()
                    .map(|entry| &entry.0);
                $crate::__private::build_async_registry_with(entries, transform)
            }
        }
    };
}

/// Submits an asynchronous provider factory to a declared inventory.
///
/// The provider expression is evaluated only when the inventory builds its
/// registry. It must not capture local state because it is compiled into a
/// regular zero-argument factory function. Service creation remains deferred
/// until application code awaits the returned resolver future.
///
/// # Examples
///
/// ```
/// # mod example {
/// # use std::convert::Infallible;
/// # use qubit_spi::error::ProviderFailure;
/// # use qubit_spi::{AsyncServiceProvider, AsyncServiceSpec, ProviderFuture, ProviderMetadata, ServiceSpec};
/// # pub struct ExampleSpec;
/// # impl ServiceSpec for ExampleSpec {
/// #     type Config = ();
/// #     type Error = Infallible;
/// # }
/// # impl AsyncServiceSpec for ExampleSpec {
/// #     type Output = ();
/// # }
/// # struct ExampleProvider;
/// # impl ProviderMetadata for ExampleProvider {
/// #     fn descriptor(&self) -> qubit_spi::ProviderDescriptor {
/// #         qubit_spi::provider_descriptor!("example")
/// #     }
/// # }
/// # impl AsyncServiceProvider<ExampleSpec> for ExampleProvider {
/// #     fn create_configured<'a>(
/// #         &'a self,
/// #         _config: &'a (),
/// #     ) -> ProviderFuture<'a, Result<(), ProviderFailure<Infallible>>> {
/// #         Box::pin(async { Ok(()) })
/// #     }
/// # }
/// # qubit_spi::declare_async_provider_inventory! {
/// #     pub mod example_providers {
/// #         spec = ExampleSpec;
/// #     }
/// # }
/// qubit_spi::submit_async_provider! {
///     inventory_entry = example_providers::Entry;
///     spec = ExampleSpec;
///     provider = ExampleProvider;
/// }
///
/// # pub fn run() {
/// #     assert_eq!(1, example_providers::build_registry().unwrap().len());
/// # }
/// # }
/// # example::run();
/// ```
#[macro_export]
macro_rules! submit_async_provider {
    (
        inventory_entry = $inventory_entry:path;
        spec = $spec:path;
        provider = $provider:expr;
    ) => {
        const _: () = {
            $crate::__private::inventory::submit! {
                <$inventory_entry>::__new(
                    || -> ::std::sync::Arc<dyn $crate::AsyncProviderDefinition<$spec>> {
                        ::std::sync::Arc::new($provider)
                    },
                    $crate::ProviderRegistrationSource::new(
                        env!("CARGO_PKG_NAME"),
                        module_path!(),
                        file!(),
                        line!(),
                    ),
                )
            }
        };
    };
}
