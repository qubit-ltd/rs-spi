// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_spi::error::{
    ProviderError,
    RegistrationError,
};
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderRegistryBuilder,
    ServiceProvider,
    ServiceSpec,
};

/// Service family used by provider registry builder tests.
struct TextSpec;

impl ServiceSpec for TextSpec {
    type Config = ();
    type Output = String;
}

/// Verifies that the default builder creates an empty registry.
#[test]
fn test_default_builder_creates_an_empty_registry() {
    let registry = ProviderRegistryBuilder::<TextSpec>::default().build();

    assert!(registry.is_empty());
}

/// Provider returning the text stored in its fixture field.
struct TextProvider(
    /// Stable service text returned by this provider.
    &'static str,
);

impl ServiceProvider<TextSpec> for TextProvider {
    /// Creates the configured text service.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused unit configuration.
    ///
    /// # Returns
    ///
    /// An owned copy of the fixture text.
    fn create(&self, _config: &()) -> Result<String, ProviderError> {
        Ok(self.0.to_owned())
    }
}

/// Verifies conflict details and atomicity for a duplicate alias.
#[test]
fn test_builder_rejects_a_selector_owned_by_another_provider() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            TextProvider("hello"),
        )
        .expect("first provider should register");

    let error = builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("spanish")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            TextProvider("hola"),
        )
        .expect_err("duplicate alias should be rejected");

    let RegistrationError::DuplicateSelector {
        selector,
        existing_provider,
        provider,
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
        .register(
            ProviderDescriptor::new(
                ProviderId::new("spanish")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["es"])
            .expect("test alias should be valid"),
            TextProvider("hola"),
        )
        .expect("a rejected registration must not partially claim selectors");

    let registry = builder.build();
    assert_eq!(
        "hola",
        registry
            .resolve("es")
            .expect("registered alias should resolve")
            .create(&())
            .expect("test provider should create its service"),
    );
}

/// Verifies that a duplicate canonical ID leaves the first provider intact.
#[test]
fn test_builder_rejects_a_duplicate_canonical_id_without_mutation() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            TextProvider("hello"),
        )
        .expect("first provider should register");

    let error = builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            TextProvider("replacement"),
        )
        .expect_err("duplicate canonical ID should be rejected");

    let RegistrationError::DuplicateSelector {
        selector,
        existing_provider,
        provider,
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
            .resolve("english")
            .expect("first provider should remain registered")
            .create(&())
            .expect("test provider should create its service"),
    );
}

/// Verifies registration of an already shared provider factory.
#[test]
fn test_builder_registers_an_already_shared_provider() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    let provider: Arc<dyn ServiceProvider<TextSpec>> =
        Arc::new(TextProvider("shared"));

    builder
        .register_shared(
            ProviderDescriptor::new(
                ProviderId::new("shared")
                    .expect("test provider ID should be valid"),
            ),
            Arc::clone(&provider),
        )
        .expect("shared provider should register");

    assert_eq!(2, Arc::strong_count(&provider));
    let registry = builder.build();
    assert_eq!(
        "shared",
        registry
            .resolve("shared")
            .expect("shared provider should resolve")
            .create(&())
            .expect("shared provider should create its service"),
    );
}

/// Verifies that cloning a built registry preserves its provider catalog.
#[test]
fn test_built_registry_clone_keeps_the_same_provider_catalog() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            TextProvider("hello"),
        )
        .expect("test provider should register");
    let registry = builder.build();

    assert_eq!(
        registry.provider_ids().collect::<Vec<_>>(),
        registry.clone().provider_ids().collect::<Vec<_>>(),
    );
}

/// Verifies priority ordering with canonical ID tie breaking.
#[test]
fn test_automatic_order_uses_priority_then_canonical_id() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    for (id, priority) in [("zulu", 10), ("low", 1), ("alpha", 10)] {
        builder
            .register(
                ProviderDescriptor::new(
                    ProviderId::new(id)
                        .expect("test provider ID should be valid"),
                )
                .with_priority(priority),
                TextProvider(id),
            )
            .expect("unique test provider should register");
    }
    let registry = builder.build();

    assert_eq!(
        vec!["zulu", "low", "alpha"],
        registry
            .provider_ids()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>(),
        "enumeration must preserve registration order",
    );

    let resolver = qubit_spi::ProviderResolver::new(
        registry,
        qubit_spi::FallbackPolicy::OnAnyError,
    );
    let created = resolver
        .create(&qubit_spi::ProviderSelection::auto(), &())
        .expect("automatic resolution should select a provider");
    assert_eq!("alpha", created.provider_id().as_str());
}
