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
    AsyncServiceSpec,
    FallbackPolicy,
    MissingProviderPolicy,
    ProviderFuture,
    ProviderMetadata,
    ProviderSelection,
    ProviderSelectionTargetRef,
    ServiceSpec,
    SyncServiceSpec,
};

/// Minimal service marker used for compile-time API checks.
struct SurfaceSpec;

impl ServiceSpec for SurfaceSpec {
    type Config = ();
}

impl SyncServiceSpec for SurfaceSpec {
    type Output = ();
}

impl AsyncServiceSpec for SurfaceSpec {
    type Output = ();
}

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
        P: AsyncServiceProvider<SurfaceSpec>
            + AsyncProviderDefinition<SurfaceSpec>
            + ProviderMetadata,
    {
    }

    fn accept_future<'a>(future: ProviderFuture<'a, ()>) {
        drop(future);
    }

    let registry = AsyncProviderRegistry::<SurfaceSpec>::default();
    let _: Option<AsyncResolvingServiceProvider<SurfaceSpec>> = None;
    accept_future(Box::pin(async {}));
    assert!(registry.is_empty());

    let _ = assert_async_provider::<NeverProvider>;
}

/// Compile-time provider used only to instantiate trait bounds.
struct NeverProvider;

impl ProviderMetadata for NeverProvider {
    fn descriptor(&self) -> qubit_spi::ProviderDescriptor {
        qubit_spi::ProviderDescriptor::new(
            qubit_spi::ProviderId::new("never")
                .expect("static provider ID should be valid"),
        )
    }
}

impl AsyncServiceProvider<SurfaceSpec> for NeverProvider {
    fn create_configured<'a>(
        &'a self,
        _config: &'a (),
    ) -> ProviderFuture<'a, Result<(), qubit_spi::error::ProviderError>> {
        Box::pin(async { Ok(()) })
    }
}
