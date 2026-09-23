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

use qubit_spi::ProviderDefinition;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderMetadata;
use qubit_spi::ProviderSelection;
use qubit_spi::ServiceProvider;
use qubit_spi::declare_sync_provider_inventory;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderInventoryBuildError;
use qubit_spi::provider_descriptor;
use qubit_spi::submit_sync_provider;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_provider_definition::TestProviderDefinition;
use crate::common::test_provider_definition::define_provider;

mod macro_path_support {
    pub(crate) use crate::common::string_spec::StringSpec as OrdinarySpec;
}

declare_sync_provider_inventory! {
    pub mod discovered_providers {
        spec = StringSpec;
    }
}

declare_sync_provider_inventory! {
    pub mod transform_providers {
        spec = StringSpec;
    }
}

submit_sync_provider! {
    inventory_entry = discovered_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("discovered", priority: 10), ConfigurableProvider::success("discovered"));
}

submit_sync_provider! {
    inventory_entry = transform_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("transform-first"), ConfigurableProvider::success("original-first"));
}

submit_sync_provider! {
    inventory_entry = transform_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("transform-second"), ConfigurableProvider::success("original-second"));
}

/// Provider adapter that replaces the service result while retaining metadata.
struct ReplacingProvider {
    /// Original definition whose descriptor remains registered.
    inner: Arc<dyn ProviderDefinition<StringSpec>>,
}

impl ProviderMetadata for ReplacingProvider {
    /// Returns the original provider descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        self.inner.descriptor()
    }
}

impl ServiceProvider<StringSpec> for ReplacingProvider {
    /// Returns the adapter's replacement result instead of delegating creation.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<TestError>> {
        Ok("transformed".to_owned())
    }
}

declare_sync_provider_inventory! {
    pub mod caller_factory_providers {
        spec = StringSpec;
    }
}

/// Builds a provider through a name that must remain visible inside the macro's
/// generated factory function.
fn factory() -> TestProviderDefinition<ConfigurableProvider> {
    define_provider(
        provider_descriptor!("caller-factory"),
        ConfigurableProvider::success("caller-factory"),
    )
}

submit_sync_provider! {
    inventory_entry = caller_factory_providers::Entry;
    spec = StringSpec;
    provider = factory();
}

declare_sync_provider_inventory! {
    pub mod absolute_entry_providers {
        spec = StringSpec;
    }
}

submit_sync_provider! {
    inventory_entry = ::inventory_test_contract::inventory::sync_inventory_tests::absolute_entry_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("absolute-entry"), ConfigurableProvider::success("absolute-entry"));
}

declare_sync_provider_inventory! {
    pub mod internal_factory_name_providers {
        spec = StringSpec;
    }
}

/// Builds a provider through the macro implementation's former internal name.
fn __qubit_spi_inventory_factory() -> TestProviderDefinition<ConfigurableProvider> {
    define_provider(
        provider_descriptor!("internal-factory-name"),
        ConfigurableProvider::success("internal-factory-name"),
    )
}

submit_sync_provider! {
    inventory_entry = internal_factory_name_providers::Entry;
    spec = StringSpec;
    provider = __qubit_spi_inventory_factory();
}

declare_sync_provider_inventory! {
    pub mod duplicate_providers {
        spec = StringSpec;
    }
}

mod alpha_duplicate_submission {
    use super::ConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_sync_provider;

    submit_sync_provider! {
        inventory_entry = super::duplicate_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("duplicate"), ConfigurableProvider::success("first"));
    }
}

mod zulu_duplicate_submission {
    use super::ConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_sync_provider;

    pub(crate) const SUBMISSION_LINE: u32 = line!() + 1;
    submit_sync_provider! {
        inventory_entry = super::duplicate_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("duplicate"), ConfigurableProvider::success("second"));
    }
}

declare_sync_provider_inventory! {
    pub mod alias_collision_providers {
        spec = StringSpec;
    }
}

submit_sync_provider! {
    inventory_entry = alias_collision_providers::Entry;
    spec = StringSpec;
    provider = define_provider(
        provider_descriptor!("alias-owner")
            .with_aliases(["taken"])
            .expect("static alias should be valid"),
        ConfigurableProvider::success("first"),
    );
}

submit_sync_provider! {
    inventory_entry = alias_collision_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("taken"), ConfigurableProvider::success("second"));
}

declare_sync_provider_inventory! {
    pub mod panic_providers {
        spec = StringSpec;
    }
}

/// Panics when link-time discovery invokes this provider factory.
fn panic_factory() -> TestProviderDefinition<ConfigurableProvider> {
    panic!("factory panic must propagate")
}

submit_sync_provider! {
    inventory_entry = panic_providers::Entry;
    spec = StringSpec;
    provider = panic_factory();
}

declare_sync_provider_inventory! {
    pub mod descriptor_panic_providers {
        spec = StringSpec;
    }
}

/// Provider whose metadata callback panics during registry registration.
struct PanicDescriptorProvider;

impl ServiceProvider<StringSpec> for PanicDescriptorProvider {
    /// Never creates a service because descriptor evaluation panics first.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<TestError>> {
        unreachable!("descriptor panic prevents provider creation")
    }
}

impl ProviderMetadata for PanicDescriptorProvider {
    /// Panics to prove registry construction preserves metadata callback
    /// panics.
    fn descriptor(&self) -> ProviderDescriptor {
        panic!("descriptor panic must propagate")
    }
}

submit_sync_provider! {
    inventory_entry = descriptor_panic_providers::Entry;
    spec = StringSpec;
    provider = PanicDescriptorProvider;
}

declare_sync_provider_inventory! {
    pub mod descriptor_count_providers {
        spec = StringSpec;
    }
}

static DESCRIPTOR_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Provider that counts descriptor sampling during inventory registration.
struct CountingDescriptorProvider;

impl ServiceProvider<StringSpec> for CountingDescriptorProvider {
    /// Never creates a service because this fixture only verifies registration.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<TestError>> {
        unreachable!("descriptor sampling test does not create services")
    }
}

impl ProviderMetadata for CountingDescriptorProvider {
    /// Returns metadata while recording each descriptor sample.
    fn descriptor(&self) -> ProviderDescriptor {
        DESCRIPTOR_CALLS.fetch_add(1, Ordering::SeqCst);
        provider_descriptor!("descriptor-count")
    }
}

submit_sync_provider! {
    inventory_entry = descriptor_count_providers::Entry;
    spec = StringSpec;
    provider = CountingDescriptorProvider;
}

declare_sync_provider_inventory! {
    pub mod local_path_providers {
        spec = self::macro_path_support::OrdinarySpec;
    }
}

declare_sync_provider_inventory! {
    pub mod self_path_providers {
        spec = self::StringSpec;
    }
}

declare_sync_provider_inventory! {
    pub mod crate_path_providers {
        spec = crate::common::string_spec::StringSpec;
    }
}

declare_sync_provider_inventory! {
    pub mod absolute_external_path_providers {
        spec = ::inventory_test_contract::ExternalStringSpec;
    }
}

mod super_path_declarations {
    use qubit_spi::declare_sync_provider_inventory;

    declare_sync_provider_inventory! {
        pub mod providers {
            spec = super::StringSpec;
        }
    }
}

declare_sync_provider_inventory! {
    pub mod ordered_providers {
        spec = StringSpec;
    }
}

mod zulu_submission {
    use super::ConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_sync_provider;

    submit_sync_provider! {
        inventory_entry = super::ordered_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("zulu", priority: 10), ConfigurableProvider::success("zulu"));
    }
}

mod alpha_submission {
    use super::ConfigurableProvider;
    use super::define_provider;
    use super::provider_descriptor;
    use super::submit_sync_provider;

    submit_sync_provider! {
        inventory_entry = super::ordered_providers::Entry;
        spec = super::StringSpec;
        provider = define_provider(provider_descriptor!("alpha", priority: -10), ConfigurableProvider::success("alpha"));
    }
}

/// Verifies that discovered providers are registered and create services.
#[test]
fn test_build_registry_discovers_and_creates_synchronous_provider() {
    let registry = discovered_providers::build_registry().expect("discovered provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("discovered").expect("static selector should be valid"))
        .expect("discovered provider should resolve");

    assert_eq!(
        "discovered",
        resolver
            .create()
            .expect("discovered provider should create its service"),
    );
}

/// Verifies that registry builders can wrap each discovered provider before
/// registration.
#[test]
fn test_build_registry_with_transforms_each_synchronous_provider() {
    let transformed = Cell::new(0);
    let registry = transform_providers::build_registry_with(|inner| {
        transformed.set(transformed.get() + 1);
        Arc::new(ReplacingProvider { inner }) as Arc<dyn ProviderDefinition<StringSpec>>
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
            resolver.create().expect("adapter should replace service result")
        );
    }
}

/// Verifies that a provider expression can call a caller function named
/// `factory` without macro-generated name capture.
#[test]
fn test_submit_provider_preserves_caller_factory_name_resolution() {
    let registry = caller_factory_providers::build_registry().expect("caller factory provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("caller-factory").expect("static selector should be valid"))
        .expect("caller factory provider should resolve");

    assert_eq!(
        "caller-factory",
        resolver
            .create()
            .expect("caller factory provider should create its service"),
    );
}

/// Verifies that a submission can name its inventory entry with a leading
/// absolute path.
#[test]
fn test_submit_provider_accepts_absolute_inventory_entry_path() {
    let registry = absolute_entry_providers::build_registry().expect("absolute entry provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("absolute-entry").expect("static selector should be valid"))
        .expect("absolute entry provider should resolve");

    assert_eq!(
        "absolute-entry",
        resolver
            .create()
            .expect("absolute entry provider should create its service"),
    );
}

/// Verifies that provider expressions cannot be captured by a macro-internal
/// factory identifier.
#[test]
fn test_submit_provider_preserves_internal_factory_name_resolution() {
    let registry =
        internal_factory_name_providers::build_registry().expect("internal factory name provider should register");
    let resolver = registry
        .resolve_selected(&ProviderSelection::named("internal-factory-name").expect("static selector should be valid"))
        .expect("internal factory name provider should resolve");

    assert_eq!(
        "internal-factory-name",
        resolver
            .create()
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
            ConfigurableProvider::success("explicit"),
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
    assert_eq!(zulu_duplicate_submission::SUBMISSION_LINE, source.line());
    assert!(error.registration_error().to_string().contains("duplicate"));
}

/// Verifies that an alias owned by one discovered provider conflicts with a
/// later provider's canonical ID.
#[test]
fn test_build_registry_reports_alias_to_id_collision() {
    let error = alias_collision_providers::build_registry()
        .expect_err("canonical ID should not reuse an earlier provider alias");
    let registration = error.registration_error();

    assert_eq!(Some("taken"), registration.selector());
    assert_eq!(Some("alias-owner"), registration.existing_provider());
    assert_eq!(Some("taken"), registration.provider());
}

/// Verifies that a discovered registry rejects later registrations after the
/// application seals it.
#[test]
fn test_build_registry_rejects_explicit_registration_after_seal() {
    let registry = discovered_providers::build_registry().expect("discovered provider should register");

    registry.seal();
    let error = registry
        .register(define_provider(
            provider_descriptor!("after-seal"),
            ConfigurableProvider::success("after-seal"),
        ))
        .expect_err("sealed inventory registry should reject explicit providers");

    assert!(error.is_sealed());
}

/// Verifies that a provider factory panic remains observable to the caller.
#[test]
fn test_build_registry_propagates_factory_panic() {
    let panic = catch_unwind(AssertUnwindSafe(panic_providers::build_registry))
        .expect_err("factory panic should escape registry construction");

    assert_eq!(Some(&"factory panic must propagate"), panic.downcast_ref::<&str>());
}

/// Verifies that a provider metadata panic remains observable to the caller.
#[test]
fn test_build_registry_propagates_descriptor_panic() {
    let panic = catch_unwind(AssertUnwindSafe(descriptor_panic_providers::build_registry))
        .expect_err("descriptor panic should escape registry construction");

    assert_eq!(Some(&"descriptor panic must propagate"), panic.downcast_ref::<&str>());
}

/// Verifies that inventory registration samples each descriptor exactly once.
#[test]
fn test_build_registry_samples_each_descriptor_once() {
    DESCRIPTOR_CALLS.store(0, Ordering::SeqCst);

    let registry = descriptor_count_providers::build_registry().expect("descriptor-count provider should register");

    assert_eq!(1, registry.len());
    assert_eq!(1, DESCRIPTOR_CALLS.load(Ordering::SeqCst));
}

/// Verifies that every supported declaration path remains anchored at the
/// declaration site.
#[test]
fn test_declare_inventory_resolves_relative_and_absolute_spec_paths() {
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
        super_path_declarations::providers::build_registry()
            .expect("super path should compile and build")
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
        resolver.create().expect("automatic provider should create its service")
    );
}
