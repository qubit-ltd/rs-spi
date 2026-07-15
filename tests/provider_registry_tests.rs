// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    ProviderDescriptor, ProviderError, ProviderId, ProviderRegistry, ResolutionErrorKind,
    ServiceProvider, ServiceSpec,
};

struct TextSpec;

impl ServiceSpec for TextSpec {
    type Config = ();
    type Output = String;
}

struct TextProvider;

impl ServiceProvider<TextSpec> for TextProvider {
    fn create(&self, _config: &()) -> Result<String, ProviderError> {
        Ok("hello".to_owned())
    }
}

#[test]
fn registry_resolves_case_insensitive_aliases() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").unwrap())
                .with_aliases(["en"])
                .unwrap(),
            TextProvider,
        )
        .unwrap();
    let registry = builder.build();

    let provider = registry.resolve(" EN ").unwrap();

    assert_eq!("english", provider.descriptor().id().as_str());
    assert_eq!("hello", provider.create(&()).unwrap());
}

#[test]
fn registry_reports_unknown_provider() {
    let registry = ProviderRegistry::<TextSpec>::default();

    let result = registry.resolve("missing");
    let error = match result {
        Ok(_) => panic!("missing provider must not resolve"),
        Err(error) => error,
    };

    assert_eq!(ResolutionErrorKind::UnknownProvider, error.kind());
}
