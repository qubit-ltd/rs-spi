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
                spec = super::$($spec)::+;
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
            ) -> Result<$crate::ProviderRegistry<$spec>, $crate::error::ProviderInventoryBuildError> {
                let entries = $crate::__private::inventory::iter::<Entry>
                    .into_iter()
                    .map(|entry| &entry.0);
                $crate::__private::build_sync_registry(entries)
            }
        }
    };
}

/// Submits a synchronous provider factory to a declared inventory.
///
/// The provider expression is evaluated only when the inventory builds its
/// registry. It must not capture local state because it is compiled into a
/// regular zero-argument factory function.
#[macro_export]
macro_rules! submit_sync_provider {
    (
        inventory_entry = $($inventory_entry:ident)::+;
        spec = $spec:path;
        provider = $provider:expr;
    ) => {
        const _: () = {
            $crate::__private::inventory::submit! {
                $($inventory_entry)::+::__new(
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
                spec = super::$($spec)::+;
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
            ) -> Result<$crate::AsyncProviderRegistry<$spec>, $crate::error::ProviderInventoryBuildError> {
                let entries = $crate::__private::inventory::iter::<Entry>
                    .into_iter()
                    .map(|entry| &entry.0);
                $crate::__private::build_async_registry(entries)
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
#[macro_export]
macro_rules! submit_async_provider {
    (
        inventory_entry = $($inventory_entry:ident)::+;
        spec = $spec:path;
        provider = $provider:expr;
    ) => {
        const _: () = {
            $crate::__private::inventory::submit! {
                $($inventory_entry)::+::__new(
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
