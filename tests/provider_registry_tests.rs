// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{
    error::Error,
    sync::Arc,
    thread,
};

use qubit_spi::error::{
    ProviderError,
    ProviderSelectorError,
    ResolutionError,
};
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ServiceProvider,
    ServiceSpec,
};

/// Service family used by provider registry integration tests.
struct TextSpec;

impl ServiceSpec for TextSpec {
    type Config = ();
    type Output = String;
}

/// Provider returning one stable text value.
struct TextProvider;

impl ServiceProvider<TextSpec> for TextProvider {
    /// Creates the stable text service.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused unit configuration.
    ///
    /// # Returns
    ///
    /// The owned string `"hello"`.
    fn create(&self, _config: &()) -> Result<String, ProviderError> {
        Ok("hello".to_owned())
    }
}

/// Verifies case-insensitive alias lookup and provider creation.
#[test]
fn test_registry_resolves_case_insensitive_aliases() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            TextProvider,
        )
        .expect("test provider should register");
    let registry = builder.build();

    let provider = registry
        .resolve(" EN ")
        .expect("normalized alias should resolve");

    assert_eq!("english", provider.descriptor().id().as_str());
    assert_eq!(
        "hello",
        provider
            .create(&())
            .expect("test provider should create its service"),
    );
}

/// Verifies the optional lookup API for known, unknown, and invalid selectors.
#[test]
fn test_registry_find_distinguishes_known_unknown_and_invalid_selectors() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            TextProvider,
        )
        .expect("test provider should register");
    let registry = builder.build();

    assert!(registry.find(" EN ").is_some_and(|provider| {
        provider.descriptor().id().as_str() == "english"
    }),);
    assert!(registry.find("missing").is_none());
    assert!(registry.find("bad selector").is_none());
}

/// Verifies structured diagnostics for an unknown normalized selector.
#[test]
fn test_registry_reports_unknown_provider() {
    let registry = ProviderRegistry::<TextSpec>::default();

    let result = registry.resolve("missing");
    let error = match result {
        Ok(_) => panic!("missing provider must not resolve"),
        Err(error) => error,
    };

    let ResolutionError::UnknownProvider { selector } = &error else {
        panic!("missing provider should produce an unknown-provider error");
    };
    assert_eq!("missing", selector.as_str());
    assert!(Error::source(&error).is_none());
}

/// Verifies registry size and emptiness before and after registration.
#[test]
fn test_registry_length_matches_emptiness_and_registration_count() {
    let empty = ProviderRegistry::<TextSpec>::default();
    assert_eq!(0, empty.len());
    assert!(empty.is_empty());

    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english").expect("valid ID"),
            ),
            TextProvider,
        )
        .expect("unique provider should register");
    let registry = builder.build();
    assert_eq!(1, registry.len());
    assert!(!registry.is_empty());
}

/// Verifies preservation of raw invalid selector input and its parse source.
#[test]
fn test_registry_preserves_invalid_selector_input_and_source() {
    let registry = ProviderRegistry::<TextSpec>::default();

    let error = match registry.resolve(" Bad Selector ") {
        Ok(_) => panic!("invalid selector must not resolve"),
        Err(error) => error,
    };

    let ResolutionError::InvalidSelector {
        input,
        selector_index,
        source,
    } = &error
    else {
        panic!("invalid provider input should retain its parser error");
    };
    assert_eq!(" Bad Selector ", input.as_ref());
    assert_eq!(None, *selector_index);
    assert!(matches!(source, ProviderSelectorError::Invalid { .. }));
    assert!(
        Error::source(&error)
            .and_then(|source| source.downcast_ref::<ProviderSelectorError>())
            .is_some()
    );
    assert!(error.to_string().contains(" Bad Selector "));
}

/// Verifies that cloned immutable registries support concurrent resolution.
#[test]
fn test_cloned_registry_supports_concurrent_lookup_and_creation() {
    let mut builder = ProviderRegistry::<TextSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            TextProvider,
        )
        .expect("test provider should register");
    let registry = Arc::new(builder.build());

    let threads = (0..8)
        .map(|_| {
            let registry = Arc::clone(&registry);
            thread::spawn(move || {
                registry
                    .resolve("english")
                    .expect("registered provider should resolve")
                    .create(&())
                    .expect("test provider should create its service")
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert_eq!(
            "hello",
            thread.join().expect("lookup thread should not panic"),
        );
    }
}
