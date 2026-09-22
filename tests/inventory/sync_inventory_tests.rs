// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
// =============================================================================

use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

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

declare_sync_provider_inventory! {
    pub mod discovered_providers {
        spec = StringSpec;
    }
}

submit_sync_provider! {
    inventory_entry = discovered_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("discovered", priority: 10), ConfigurableProvider::success("discovered"));
}

declare_sync_provider_inventory! {
    pub mod duplicate_providers {
        spec = StringSpec;
    }
}

submit_sync_provider! {
    inventory_entry = duplicate_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("duplicate"), ConfigurableProvider::success("first"));
}

submit_sync_provider! {
    inventory_entry = duplicate_providers::Entry;
    spec = StringSpec;
    provider = define_provider(provider_descriptor!("duplicate"), ConfigurableProvider::success("second"));
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
    fn descriptor(&self) -> qubit_spi::ProviderDescriptor {
        panic!("descriptor panic must propagate")
    }
}

submit_sync_provider! {
    inventory_entry = descriptor_panic_providers::Entry;
    spec = StringSpec;
    provider = PanicDescriptorProvider;
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
        provider = define_provider(provider_descriptor!("zulu"), ConfigurableProvider::success("zulu"));
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
        provider = define_provider(provider_descriptor!("alpha"), ConfigurableProvider::success("alpha"));
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

    assert!(matches!(error, ProviderInventoryBuildError::Registration { .. }));
    assert_eq!(module_path!(), error.source_location().module_path());
    assert!(error.registration_error().to_string().contains("duplicate"));
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

/// Verifies that source ordering determines the order of successful
/// registrations.
#[test]
fn test_build_registry_sorts_entries_by_submission_source() {
    let registry = ordered_providers::build_registry().expect("ordered providers should register");

    assert_eq!(
        vec!["alpha", "zulu"],
        registry.provider_ids().iter().map(|id| id.as_str()).collect::<Vec<_>>(),
    );
}
