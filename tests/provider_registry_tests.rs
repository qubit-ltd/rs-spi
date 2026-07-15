// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{error::Error, sync::Arc, thread};

use qubit_spi::{
    ProviderDescriptor, ProviderError, ProviderId, ProviderRegistry, ProviderSelectorError,
    ProviderSelectorErrorKind, ResolutionErrorKind, ServiceProvider, ServiceSpec,
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
    assert_eq!(Some("missing"), error.selector_input());
    assert!(Error::source(&error).is_none());
}

#[test]
fn registry_length_matches_emptiness_and_registration_count() {
    let empty = ProviderRegistry::<TextSpec>::default();
    assert_eq!(0, empty.len());
    assert!(empty.is_empty());

    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").expect("valid ID")),
            TextProvider,
        )
        .expect("unique provider should register");
    let registry = builder.build();
    assert_eq!(1, registry.len());
    assert!(!registry.is_empty());
}

#[test]
fn registry_preserves_invalid_selector_input_and_source() {
    let registry = ProviderRegistry::<TextSpec>::default();

    let error = match registry.resolve(" Bad Selector ") {
        Ok(_) => panic!("invalid selector must not resolve"),
        Err(error) => error,
    };

    assert_eq!(ResolutionErrorKind::InvalidSelector, error.kind());
    assert_eq!(Some(" Bad Selector "), error.selector_input());
    assert_eq!(None, error.requested_selector());
    let source = Error::source(&error)
        .and_then(|source| source.downcast_ref::<ProviderSelectorError>())
        .expect("invalid selector must retain its selector error source");
    assert_eq!(ProviderSelectorErrorKind::Invalid, source.kind());
    assert!(error.to_string().contains(" Bad Selector "));
}

#[test]
fn cloned_registry_supports_concurrent_lookup_and_creation() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("english").unwrap()),
            TextProvider,
        )
        .unwrap();
    let registry = Arc::new(builder.build());

    let threads = (0..8)
        .map(|_| {
            let registry = Arc::clone(&registry);
            thread::spawn(move || registry.resolve("english").unwrap().create(&()).unwrap())
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert_eq!("hello", thread.join().unwrap());
    }
}
