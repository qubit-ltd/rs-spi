// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::Cell;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_spi::AsyncProviderDefinition;
use qubit_spi::AsyncServiceProvider;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderFuture;
use qubit_spi::ProviderMetadata;
use qubit_spi::ProviderSelection;
use qubit_spi::declare_async_provider_inventory;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderInventoryBuildError;
use qubit_spi::provider_descriptor;
use qubit_spi::submit_async_provider;

use crate::common::async_configurable_provider::AsyncConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_provider_definition::TestProviderDefinition;
use crate::common::test_provider_definition::define_provider;

mod macro_path_support {
    pub(crate) use crate::common::string_spec::StringSpec as OrdinarySpec;
}

declare_async_provider_inventory! {
    pub mod discovered_providers {
        spec = StringSpec;
    }
}

declare_async_provider_inventory! {
    pub mod transform_providers {
        spec = StringSpec;
    }
}

submit_async_provider! {
    inventory_entry = discovered_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("discovered", priority: 10), AsyncConfigurableProvider::success("discovered"));
}

submit_async_provider! {
    inventory_entry = transform_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("transform-first"), AsyncConfigurableProvider::success("original-first"));
}

submit_async_provider! {
    inventory_entry = transform_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("transform-second"), AsyncConfigurableProvider::success("original-second"));
}

/// Provider adapter that replaces the service result while retaining metadata.
struct ReplacingProvider {
    /// Original definition whose descriptor remains registered.
    inner: Arc<dyn AsyncProviderDefinition<StringSpec>>,
}

impl ProviderMetadata for ReplacingProvider {
    /// Returns the original provider descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        self.inner.descriptor()
    }
}

impl AsyncServiceProvider<StringSpec> for ReplacingProvider {
    /// Returns the adapter's replacement result instead of delegating creation.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        Box::pin(async { Ok("transformed".to_owned()) })
    }
}

declare_async_provider_inventory! {
    pub mod caller_factory_providers {
        spec = StringSpec;
    }
}

/// Builds an asynchronous provider through a name that must remain visible
/// inside the macro's generated factory function.
fn factory() -> TestProviderDefinition<AsyncConfigurableProvider> {
    define_provider(
        provider_descriptor!("caller-factory"),
        AsyncConfigurableProvider::success("caller-factory"),
    )
}

submit_async_provider! {
    inventory_entry = caller_factory_providers::Entry;
    spec = StringSpec;
    provider = factory();
}

declare_async_provider_inventory! {
    pub mod absolute_entry_providers {
        spec = StringSpec;
    }
}

submit_async_provider! {
    inventory_entry = ::inventory_test_contract::inventory::async_inventory_tests::absolute_entry_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("absolute-entry"), AsyncConfigurableProvider::success("absolute-entry"));
}

declare_async_provider_inventory! {
    pub mod internal_factory_name_providers {
        spec = StringSpec;
    }
}

/// Builds an asynchronous provider through the macro implementation's former
/// internal name.
fn __qubit_spi_inventory_factory() -> TestProviderDefinition<AsyncConfigurableProvider> {
    define_provider(
        provider_descriptor!("internal-factory-name"),
        AsyncConfigurableProvider::success("internal-factory-name"),
    )
}

submit_async_provider! {
    inventory_entry = internal_factory_name_providers::Entry;
    spec = StringSpec;
    provider = __qubit_spi_inventory_factory();
}

declare_async_provider_inventory! {
    pub mod duplicate_providers {
        spec = StringSpec;
    }
}

mod alpha_duplicate_submission {
    use super::AsyncConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_async_provider;

    submit_async_provider! {
        inventory_entry = super::duplicate_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("duplicate"), AsyncConfigurableProvider::success("first"));
    }
}

mod zulu_duplicate_submission {
    use super::AsyncConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_async_provider;

    submit_async_provider! {
        inventory_entry = super::duplicate_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("duplicate"), AsyncConfigurableProvider::success("second"));
    }
}

declare_async_provider_inventory! {
    pub mod panic_providers {
        spec = StringSpec;
    }
}

/// Panics when link-time discovery invokes this asynchronous provider factory.
fn panic_factory() -> TestProviderDefinition<AsyncConfigurableProvider> {
    panic!("async factory panic must propagate")
}

submit_async_provider! {
    inventory_entry = panic_providers::Entry;
    spec = StringSpec;
    provider = panic_factory();
}

declare_async_provider_inventory! {
    pub mod descriptor_panic_providers {
        spec = StringSpec;
    }
}

/// Provider whose metadata callback panics during registry registration.
struct PanicDescriptorProvider;

impl AsyncServiceProvider<StringSpec> for PanicDescriptorProvider {
    /// Never creates a service because descriptor evaluation panics first.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        unreachable!("descriptor panic prevents provider creation")
    }
}

impl ProviderMetadata for PanicDescriptorProvider {
    /// Panics to prove registry construction preserves metadata callback
    /// panics.
    fn descriptor(&self) -> ProviderDescriptor {
        panic!("async descriptor panic must propagate")
    }
}

submit_async_provider! {
    inventory_entry = descriptor_panic_providers::Entry;
    spec = StringSpec;
    provider = PanicDescriptorProvider;
}

declare_async_provider_inventory! {
    pub mod lifecycle_providers {
        spec = StringSpec;
    }
}

static DESCRIPTOR_CALLS: AtomicUsize = AtomicUsize::new(0);
static CREATE_CONFIGURED_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Provider that records metadata and creation calls during inventory builds.
struct LifecycleProvider;

impl AsyncServiceProvider<StringSpec> for LifecycleProvider {
    /// Records service creation before returning a successful asynchronous
    /// output.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        CREATE_CONFIGURED_CALLS.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok("lifecycle".to_owned()) })
    }
}

impl ProviderMetadata for LifecycleProvider {
    /// Returns metadata while recording each descriptor sample.
    fn descriptor(&self) -> ProviderDescriptor {
        DESCRIPTOR_CALLS.fetch_add(1, Ordering::SeqCst);
        provider_descriptor!("lifecycle")
    }
}

submit_async_provider! {
    inventory_entry = lifecycle_providers::Entry;
    spec = StringSpec;
    provider = LifecycleProvider;
}

declare_async_provider_inventory! {
    pub mod root_path_providers {
        spec = StringSpec;
    }
}

declare_async_provider_inventory! {
    pub mod local_path_providers {
        spec = self::macro_path_support::OrdinarySpec;
    }
}

declare_async_provider_inventory! {
    pub mod self_path_providers {
        spec = self::StringSpec;
    }
}

declare_async_provider_inventory! {
    pub mod crate_path_providers {
        spec = crate::common::string_spec::StringSpec;
    }
}

declare_async_provider_inventory! {
    pub mod absolute_external_path_providers {
        spec = ::inventory_test_contract::ExternalStringSpec;
    }
}

mod nested_path_declarations {
    use qubit_spi::declare_async_provider_inventory;

    declare_async_provider_inventory! {
        pub mod providers {
            spec = super::StringSpec;
        }
    }
}

declare_async_provider_inventory! {
    pub mod ordered_providers {
        spec = StringSpec;
    }
}

mod zulu_submission {
    use super::AsyncConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_async_provider;

    submit_async_provider! {
        inventory_entry = super::ordered_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("zulu", priority: 10), AsyncConfigurableProvider::success("zulu"));
    }
}

mod alpha_submission {
    use super::AsyncConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_async_provider;

    submit_async_provider! {
        inventory_entry = super::ordered_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("alpha", priority: -10), AsyncConfigurableProvider::success("alpha"));
    }
}

/// Verifies that discovered asynchronous providers are registered and create
/// services.
#[test]
fn test_build_registry_discovers_and_creates_asynchronous_provider() {
    let registry = discovered_providers::build_registry().expect("discovered provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("discovered").expect("static selector should be valid"))
        .expect("discovered provider should resolve");

    assert_eq!(
        "discovered",
        futures::executor::block_on(resolver.create()).expect("discovered provider should create its service"),
    );
}

/// Verifies that asynchronous registry builders can wrap each provider before
/// registration.
#[test]
fn test_build_registry_with_transforms_each_asynchronous_provider() {
    let transformed = Cell::new(0);
    let registry = transform_providers::build_registry_with(|inner| {
        transformed.set(transformed.get() + 1);
        Arc::new(ReplacingProvider { inner }) as Arc<dyn AsyncProviderDefinition<StringSpec>>
    })
    .expect("transformed provider should register");

    assert_eq!(2, transformed.get());
    assert_eq!(2, registry.len());
    for selector in ["transform-first", "transform-second"] {
        let resolver = registry
            .resolve_selected(&ProviderSelection::named(selector).expect("static selector should be valid"))
            .expect("transformed provider should resolve");
        assert_eq!(
            "transformed",
            futures::executor::block_on(resolver.create()).expect("adapter should replace service result"),
        );
    }
}

/// Verifies that an asynchronous provider expression can call a caller
/// function named `factory` without macro-generated name capture.
#[test]
fn test_submit_provider_preserves_caller_factory_name_resolution() {
    let registry = caller_factory_providers::build_registry().expect("caller factory provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("caller-factory").expect("static selector should be valid"))
        .expect("caller factory provider should resolve");

    assert_eq!(
        "caller-factory",
        futures::executor::block_on(resolver.create()).expect("caller factory provider should create its service"),
    );
}

/// Verifies that an asynchronous submission can name its inventory entry with
/// a leading absolute path.
#[test]
fn test_submit_provider_accepts_absolute_inventory_entry_path() {
    let registry = absolute_entry_providers::build_registry().expect("absolute entry provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("absolute-entry").expect("static selector should be valid"))
        .expect("absolute entry provider should resolve");

    assert_eq!(
        "absolute-entry",
        futures::executor::block_on(resolver.create()).expect("absolute entry provider should create its service"),
    );
}

/// Verifies that asynchronous provider expressions cannot be captured by a
/// macro-internal factory identifier.
#[test]
fn test_submit_provider_preserves_internal_factory_name_resolution() {
    let registry =
        internal_factory_name_providers::build_registry().expect("internal factory name provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("internal-factory-name").expect("static selector should be valid"))
        .expect("internal factory name provider should resolve");

    assert_eq!(
        "internal-factory-name",
        futures::executor::block_on(resolver.create())
            .expect("internal factory name provider should create its service"),
    );
}

/// Verifies that a successfully built registry remains mutable for application
/// providers.
#[test]
fn test_build_registry_returns_unsealed_registry_that_accepts_explicit_provider() {
    let registry = discovered_providers::build_registry().expect("discovered provider should register");

    assert!(!registry.is_sealed());
    registry
        .register(define_provider(
            provider_descriptor!("explicit"),
            AsyncConfigurableProvider::success("explicit"),
        ))
        .expect("application provider should register after discovery");

    assert_eq!(
        vec!["discovered", "explicit"],
        registry.provider_ids().iter().map(|id| id.as_str()).collect::<Vec<_>>(),
    );
}

/// Verifies that duplicate discovered providers retain the failing submission
/// source.
#[test]
fn test_build_registry_reports_source_for_duplicate_provider_id() {
    let error = duplicate_providers::build_registry().expect_err("duplicate provider ID should be rejected");
    let source = error.source_location();

    assert!(matches!(error, ProviderInventoryBuildError::Registration { .. }));
    assert_eq!(
        concat!(module_path!(), "::zulu_duplicate_submission"),
        source.module_path()
    );
    assert_eq!(env!("CARGO_PKG_NAME"), source.crate_name());
    assert_eq!(file!(), source.file());
    assert!(error.registration_error().to_string().contains("duplicate"));
}

/// Verifies that an asynchronous provider factory panic remains observable to
/// the caller.
#[test]
fn test_build_registry_propagates_factory_panic() {
    let panic = catch_unwind(AssertUnwindSafe(panic_providers::build_registry))
        .expect_err("factory panic should escape registry construction");

    assert_eq!(
        Some(&"async factory panic must propagate"),
        panic.downcast_ref::<&str>()
    );
}

/// Verifies that asynchronous provider metadata panic remains observable to the
/// caller.
#[test]
fn test_build_registry_propagates_descriptor_panic() {
    let panic = catch_unwind(AssertUnwindSafe(descriptor_panic_providers::build_registry))
        .expect_err("descriptor panic should escape registry construction");

    assert_eq!(
        Some(&"async descriptor panic must propagate"),
        panic.downcast_ref::<&str>()
    );
}

/// Verifies that construction samples metadata once and does not invoke service
/// creation.
#[test]
fn test_build_registry_samples_each_descriptor_once_without_creating_services() {
    DESCRIPTOR_CALLS.store(0, Ordering::SeqCst);
    CREATE_CONFIGURED_CALLS.store(0, Ordering::SeqCst);

    let registry = lifecycle_providers::build_registry().expect("lifecycle provider should register");

    assert_eq!(1, registry.len());
    assert_eq!(1, DESCRIPTOR_CALLS.load(Ordering::SeqCst));
    assert_eq!(0, CREATE_CONFIGURED_CALLS.load(Ordering::SeqCst));
}

/// Verifies that every supported declaration path resolves at the declaration
/// site.
#[test]
fn test_declare_inventory_resolves_relative_and_absolute_spec_paths() {
    assert!(
        root_path_providers::build_registry()
            .expect("root path should compile and build")
            .is_empty()
    );
    assert!(
        local_path_providers::build_registry()
            .expect("local multi-segment path should compile and build")
            .is_empty()
    );
    assert!(
        self_path_providers::build_registry()
            .expect("self path should compile and build")
            .is_empty()
    );
    assert!(
        crate_path_providers::build_registry()
            .expect("crate path should compile and build")
            .is_empty()
    );
    assert!(
        nested_path_declarations::providers::build_registry()
            .expect("nested super path should compile and build")
            .is_empty()
    );
    assert!(
        absolute_external_path_providers::build_registry()
            .expect("absolute external path should compile and build")
            .is_empty()
    );
}

/// Verifies that source ordering determines the order of successful
/// registrations.
#[test]
fn test_build_registry_sorts_entries_by_submission_source() {
    let registry = ordered_providers::build_registry().expect("ordered providers should register");

    assert_eq!(
        vec!["alpha", "zulu"],
        registry.provider_ids().iter().map(|id| id.as_str()).collect::<Vec<_>>(),
    );
    let resolver = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve the higher-priority provider");

    assert_eq!(
        "zulu",
        futures::executor::block_on(resolver.create()).expect("automatic provider should create its service"),
    );
}
