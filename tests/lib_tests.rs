// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    AsyncProviderDefinition,
    AsyncProviderRegistry,
    AsyncResolvingServiceProvider,
    AsyncServiceProvider,
    FallbackPolicy,
    MissingProviderPolicy,
    ProviderCreationTermination,
    ProviderDefinition,
    ProviderDescriptor,
    ProviderFuture,
    ProviderId,
    ProviderMetadata,
    ProviderRegistry,
    ProviderSelection,
    ProviderSelectionTargetRef,
    ProviderSelector,
    ResolvingServiceProvider,
    ServiceProvider,
};

use crate::common::async_configurable_provider::AsyncConfigurableProvider;
use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::TestProviderDefinition;

/// Verifies the crate root re-exports core selection types.
#[test]
fn test_crate_root_reexports_core_selection_types() {
    let selection =
        ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never);
    assert_eq!(FallbackPolicy::Never, selection.fallback_policy());
}

/// Verifies the crate root exposes strict and lenient chain target policies.
#[test]
fn test_crate_root_reexports_selection_policy_types() {
    let selection = ProviderSelection::chain_allowing_missing(["optional"])
        .expect("static selector should be valid");

    assert!(matches!(
        selection.target(),
        ProviderSelectionTargetRef::Chain {
            missing_policy: MissingProviderPolicy::Ignore,
            ..
        }
    ));
}

/// Verifies async Registry and contract types are available from the root.
#[test]
fn test_crate_root_reexports_async_provider_types() {
    fn assert_async_provider<P>()
    where
        P: AsyncServiceProvider<StringSpec>
            + AsyncProviderDefinition<StringSpec>
            + ProviderMetadata,
    {
    }

    fn accept_future<'a>(future: ProviderFuture<'a, ()>) {
        drop(future);
    }

    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let _: Option<AsyncResolvingServiceProvider<StringSpec>> = None;
    accept_future(Box::pin(async {}));
    assert!(registry.is_empty());

    let _ = assert_async_provider::<
        TestProviderDefinition<AsyncConfigurableProvider>,
    >;
}

/// Verifies every remaining root re-export used by the synchronous SPI API.
#[test]
fn test_crate_root_reexports_sync_provider_types() {
    fn assert_sync_provider<P>()
    where
        P: ServiceProvider<StringSpec>
            + ProviderDefinition<StringSpec>
            + ProviderMetadata,
    {
    }

    let _: Option<ResolvingServiceProvider<StringSpec>> = None;
    let registry = ProviderRegistry::<StringSpec>::default();
    let selector = ProviderSelector::parse("sync")
        .expect("static selector should be valid");
    let descriptor = ProviderDescriptor::new(
        ProviderId::new("sync").expect("static ID should be valid"),
    );

    assert!(registry.is_empty());
    assert_eq!("sync", selector.as_str());
    assert_eq!("sync", descriptor.id().as_str());
    assert!(matches!(
        ProviderCreationTermination::Exhausted,
        ProviderCreationTermination::Exhausted
    ));

    let _ =
        assert_sync_provider::<TestProviderDefinition<ConfigurableProvider>>;
}
