// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_spi::error::RegistrationError;
use qubit_spi::{
    ProviderDefinition,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderRegistryBuilder,
    ProviderSelection,
    ServiceProvider,
};

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies that the default builder creates an empty registry.
#[test]
fn test_default_builder_creates_an_empty_registry() {
    let registry = ProviderRegistryBuilder::<StringSpec>::default().build();

    assert!(registry.is_empty());
}

/// Verifies conflict details and atomicity for a duplicate alias.
#[test]
fn test_builder_rejects_a_selector_owned_by_another_provider() {
    let mut builder = ProviderRegistry::<StringSpec>::builder();
    builder
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

    let error = builder
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("spanish")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            ConfigurableProvider::success("hola"),
        ))
        .expect_err("duplicate alias should be rejected");

    let RegistrationError::DuplicateSelector {
        selector,
        existing_provider,
        provider,
        ..
    } = &error
    else {
        panic!("expected a duplicate selector error");
    };
    assert_eq!("en", selector.as_ref());
    assert_eq!("english", existing_provider.as_ref());
    assert_eq!("spanish", provider.as_ref());
    assert_eq!(
        "provider selector en claimed by spanish is already owned by english",
        error.to_string(),
    );

    builder
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("spanish")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["es"])
            .expect("test alias should be valid"),
            ConfigurableProvider::success("hola"),
        ))
        .expect("a rejected registration must not partially claim selectors");

    let registry = builder.build();
    assert_eq!(
        "hola",
        registry
            .resolve(
                &ProviderSelection::named("es")
                    .expect("test selector should be valid"),
            )
            .expect("registered alias should resolve")
            .create_default()
            .expect("test provider should create its service"),
    );
}

/// Verifies that a duplicate canonical ID leaves the first provider intact.
#[test]
fn test_builder_rejects_a_duplicate_canonical_id_without_mutation() {
    let mut builder = ProviderRegistry::<StringSpec>::builder();
    builder
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("hello"),
        ))
        .expect("first provider should register");

    let error = builder
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("replacement"),
        ))
        .expect_err("duplicate canonical ID should be rejected");

    let RegistrationError::DuplicateSelector {
        selector,
        existing_provider,
        provider,
        ..
    } = &error
    else {
        panic!("expected a duplicate selector error");
    };
    assert_eq!("english", selector.as_ref());
    assert_eq!("english", existing_provider.as_ref());
    assert_eq!("english", provider.as_ref());
    let registry = builder.build();
    assert_eq!(
        "hello",
        registry
            .resolve(
                &ProviderSelection::named("english")
                    .expect("test selector should be valid"),
            )
            .expect("first provider should remain registered")
            .create_default()
            .expect("test provider should create its service"),
    );
}

/// Verifies registration of an already shared provider factory.
#[test]
fn test_builder_registers_an_already_shared_provider() {
    let mut builder = ProviderRegistry::<StringSpec>::builder();
    let provider: Arc<dyn ProviderDefinition<StringSpec>> =
        Arc::new(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("shared")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("shared"),
        ));

    builder
        .register_shared(Arc::clone(&provider))
        .expect("shared provider should register");

    assert_eq!(2, Arc::strong_count(&provider));
    let registry = builder.build();
    assert_eq!(
        "shared",
        registry
            .resolve(
                &ProviderSelection::named("shared")
                    .expect("test selector should be valid"),
            )
            .expect("shared provider should resolve")
            .create_default()
            .expect("shared provider should create its service"),
    );
}

/// Verifies that cloning a built registry preserves its provider catalog.
#[test]
fn test_built_registry_clone_keeps_the_same_provider_catalog() {
    let mut builder = ProviderRegistry::<StringSpec>::builder();
    builder
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::success("hello"),
        ))
        .expect("test provider should register");
    let registry = builder.build();

    assert_eq!(registry.provider_ids(), registry.clone().provider_ids(),);
}

/// Verifies priority ordering with canonical ID tie breaking.
#[test]
fn test_automatic_order_uses_priority_then_canonical_id() {
    let mut builder = ProviderRegistry::<StringSpec>::builder();
    for (id, priority) in [("zulu", 10), ("low", 1), ("alpha", 10)] {
        builder
            .register(define_provider(
                ProviderDescriptor::new(
                    ProviderId::new(id)
                        .expect("test provider ID should be valid"),
                )
                .with_priority(priority),
                ConfigurableProvider::success(id),
            ))
            .expect("unique test provider should register");
    }
    let registry = builder.build();

    assert_eq!(
        vec!["zulu", "low", "alpha"],
        registry
            .provider_ids()
            .iter()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>(),
        "enumeration must preserve registration order",
    );

    let created = registry
        .resolve(
            &ProviderSelection::auto()
                .with_fallback_policy(qubit_spi::FallbackPolicy::OnAnyError),
        )
        .expect("automatic selection should resolve")
        .create_default()
        .expect("automatic resolution should select a provider");
    assert_eq!("alpha", created);
}
