// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::thread;

use qubit_spi::error::{
    ProviderResolutionError,
    RegistrationError,
};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderSelection,
};

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies that a registry accepts a self-described provider after creation.
#[test]
fn test_registry_registers_a_self_described_provider_at_runtime() {
    let registry = ProviderRegistry::<StringSpec>::default();

    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("hello"),
        ))
        .expect("runtime registration should succeed");

    assert_eq!(1, registry.len());
    assert_eq!("english", registry.provider_ids()[0].as_str());
}

/// Verifies that a failed registration leaves every selector unclaimed.
#[test]
fn test_registry_rejects_conflicts_without_partial_mutation() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            ConfigurableProvider::success("hello"),
        ))
        .expect("first provider should register");

    let error = registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("spanish")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["es", "en"])
            .expect("test aliases should be valid"),
            ConfigurableProvider::success("hola"),
        ))
        .expect_err("duplicate alias should be rejected");

    assert!(matches!(error, RegistrationError::DuplicateSelector { .. }));
    assert_eq!(
        vec!["english"],
        registry
            .provider_ids()
            .iter()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>()
    );
    let selection =
        ProviderSelection::named("es").expect("test selector should be valid");
    assert!(matches!(
        registry.resolve_selected(&selection),
        Err(ProviderResolutionError::UnknownProvider { .. }),
    ));
}

/// Verifies that cloned handles observe providers registered later.
#[test]
fn test_registry_clones_share_later_registrations() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let clone = registry.clone();

    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("hello"),
        ))
        .expect("runtime registration should succeed");

    assert_eq!(
        vec!["english"],
        clone
            .provider_ids()
            .iter()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>()
    );
}

/// Verifies reading and replacing the registry's default selection.
#[test]
fn test_registry_uses_and_updates_default_selection() {
    let registry = ProviderRegistry::<StringSpec>::default();
    assert_eq!(ProviderSelection::auto(), registry.default_selection());

    let selection = ProviderSelection::named("english")
        .expect("test selector should be valid")
        .with_fallback_policy(FallbackPolicy::OnAnyError);
    registry.set_default_selection(selection.clone());

    assert_eq!(selection, registry.default_selection());
}

/// Verifies default resolution and registry debug metadata snapshots.
#[test]
fn test_registry_resolves_configured_default_and_formats_snapshot() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("hello"),
        ))
        .expect("test provider should register");
    registry.set_default_selection(
        ProviderSelection::named("english")
            .expect("test selector should be valid"),
    );

    let output = registry
        .resolve()
        .expect("default selection should resolve")
        .create()
        .expect("default provider should create its service");
    let debug = format!("{registry:?}");

    assert_eq!("hello", output);
    assert!(debug.contains("ProviderRegistry"));
    assert!(debug.contains("english"));
    assert!(debug.contains("default_selection"));
}

/// Verifies that descriptor snapshots retain successful registration order.
#[test]
fn test_registry_preserves_registration_order_in_descriptor_snapshots() {
    let registry = ProviderRegistry::<StringSpec>::default();
    for id in ["third", "first", "second"] {
        registry
            .register(define_provider(
                ProviderDescriptor::new(
                    ProviderId::new(id)
                        .expect("test provider ID should be valid"),
                ),
                ConfigurableProvider::success("hello"),
            ))
            .expect("unique provider should register");
    }

    assert_eq!(
        vec!["third", "first", "second"],
        registry
            .descriptors()
            .iter()
            .map(|descriptor| descriptor.id().as_str())
            .collect::<Vec<_>>(),
    );
}

/// Verifies concurrent registration and owned metadata snapshots.
#[test]
fn test_registry_supports_concurrent_registration_and_snapshot_reads() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let threads = (0..8)
        .map(|index| {
            let registry = registry.clone();
            thread::spawn(move || {
                let id = format!("provider-{index}");
                registry
                    .register(define_provider(
                        ProviderDescriptor::new(
                            ProviderId::new(&id)
                                .expect("test provider ID should be valid"),
                        ),
                        ConfigurableProvider::success("hello"),
                    ))
                    .expect("unique provider should register");
                registry.descriptors()
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert!(
            !thread
                .join()
                .expect("registration thread should not panic")
                .is_empty()
        );
    }
    let mut ids = registry
        .provider_ids()
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!(
        (0..8)
            .map(|index| format!("provider-{index}"))
            .collect::<Vec<_>>(),
        ids,
    );
}

/// Verifies registry size and emptiness before and after registration.
#[test]
fn test_registry_length_matches_emptiness_and_registration_count() {
    let empty = ProviderRegistry::<StringSpec>::default();
    assert_eq!(0, empty.len());
    assert!(empty.is_empty());

    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english").expect("valid ID"),
            ),
            ConfigurableProvider::success("hello"),
        ))
        .expect("unique provider should register");
    assert_eq!(1, registry.len());
    assert!(!registry.is_empty());
}
