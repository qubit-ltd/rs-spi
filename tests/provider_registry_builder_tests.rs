// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

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
    assert_eq!(Some("english"), error.existing_provider());
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
