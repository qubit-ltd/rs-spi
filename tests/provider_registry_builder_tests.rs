// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_spi::{
    ProviderDescriptor, ProviderError, ProviderId, ProviderRegistry, RegistrationErrorKind,
    ServiceProvider, ServiceSpec,
};

struct TextSpec;

impl ServiceSpec for TextSpec {
    type Config = ();
    type Output = String;
}

struct TextProvider(&'static str);

impl ServiceProvider<TextSpec> for TextProvider {
    fn create(&self, _config: &()) -> Result<String, ProviderError> {
        Ok(self.0.to_owned())
    }
}

#[test]
fn builder_rejects_a_selector_owned_by_another_provider() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").unwrap())
                .with_aliases(["en"])
                .unwrap(),
            TextProvider("hello"),
        )
        .unwrap();

    let error = builder
        .register(
            ProviderDescriptor::new(ProviderId::new("spanish").unwrap())
                .with_aliases(["en"])
                .unwrap(),
            TextProvider("hola"),
        )
        .unwrap_err();

    assert_eq!(RegistrationErrorKind::DuplicateSelector, error.kind());
    assert_eq!(Some("en"), error.identifier());
    assert_eq!(Some("english"), error.existing_provider());
    assert_eq!(Some("spanish"), error.provider());
    assert_eq!(
        "provider selector en claimed by spanish is already owned by english",
        error.to_string(),
    );

    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("spanish").unwrap())
                .with_aliases(["es"])
                .unwrap(),
            TextProvider("hola"),
        )
        .expect("a rejected registration must not partially claim selectors");

    let registry = builder.build();
    assert_eq!("hola", registry.resolve("es").unwrap().create(&()).unwrap());
}

#[test]
fn builder_rejects_a_duplicate_canonical_id_without_mutation() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").unwrap()),
            TextProvider("hello"),
        )
        .unwrap();

    let error = builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").unwrap()),
            TextProvider("replacement"),
        )
        .unwrap_err();

    assert_eq!(Some("english"), error.identifier());
    assert_eq!(Some("english"), error.existing_provider());
    assert_eq!(Some("english"), error.provider());
    let registry = builder.build();
    assert_eq!(
        "hello",
        registry.resolve("english").unwrap().create(&()).unwrap()
    );
}

#[test]
fn builder_registers_an_already_shared_provider() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    let provider: Arc<dyn ServiceProvider<TextSpec>> = Arc::new(TextProvider("shared"));

    builder
        .register_shared(
            ProviderDescriptor::new(ProviderId::new("shared").unwrap()),
            Arc::clone(&provider),
        )
        .unwrap();

    assert_eq!(2, Arc::strong_count(&provider));
    let registry = builder.build();
    assert_eq!(
        "shared",
        registry.resolve("shared").unwrap().create(&()).unwrap()
    );
}

#[test]
fn built_registry_clone_keeps_the_same_provider_catalog() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").unwrap()),
            TextProvider("hello"),
        )
        .unwrap();
    let registry = builder.build();

    assert_eq!(
        registry.provider_ids().collect::<Vec<_>>(),
        registry.clone().provider_ids().collect::<Vec<_>>(),
    );
}

#[test]
fn automatic_order_uses_priority_then_canonical_id() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    for (id, priority) in [("zulu", 10), ("low", 1), ("alpha", 10)] {
        builder
            .register(
                ProviderDescriptor::new(ProviderId::new(id).unwrap()).with_priority(priority),
                TextProvider(id),
            )
            .unwrap();
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

    let resolver =
        qubit_spi::ProviderResolver::new(registry, qubit_spi::FallbackPolicy::OnAnyError);
    let created = resolver
        .create(&qubit_spi::ProviderSelection::Auto, &())
        .unwrap();
    assert_eq!("alpha", created.provider_id().as_str());
}
