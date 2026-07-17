// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::error::{
    ProviderCreationError,
    ProviderError,
};
use qubit_spi::{
    FallbackPolicy,
    ProviderCreationTermination,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderSelection,
    ServiceProvider,
};

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies that a leaf provider failure retains its complete source chain.
#[test]
fn test_provider_error_converts_to_creation_error_without_losing_source() {
    let leaf = ProviderError::unavailable_with_source(
        "runtime is absent",
        std::io::Error::other("ENOENT"),
    );
    let error = ProviderCreationError::from(leaf);

    assert!(matches!(error, ProviderCreationError::Provider(_)));
    assert!(Error::source(&error).is_some());
    assert!(Error::source(&error).and_then(Error::source).is_some());
    assert!(error.is_absence());
    assert!(error.attempts().is_empty());
    assert!(error.termination().is_none());
    assert!(error.attempts().last().is_none());
    assert!(error.decisive_attempt().is_none());
    assert_eq!("provider Unavailable: runtime is absent", error.to_string());

    let invalid = ProviderCreationError::from(
        ProviderError::invalid_configuration("invalid setting"),
    );
    assert!(!invalid.is_absence());
}

/// Verifies diagnostics for exhaustion after multiple absence failures.
#[test]
fn test_exhausted_creation_error_exposes_ordered_ambiguous_diagnostics() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(
        &registry,
        "first",
        20,
        ProviderError::unavailable("first unavailable"),
    );
    register_failure(
        &registry,
        "second",
        10,
        ProviderError::unsupported("second unsupported"),
    );
    let selection = ProviderSelection::auto()
        .with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("both providers should fail");

    assert_eq!(
        Some(ProviderCreationTermination::Exhausted),
        error.termination(),
    );
    assert!(error.is_absence());
    assert_eq!(
        "second",
        error
            .attempts()
            .last()
            .expect("aggregate should have a terminal attempt")
            .provider_id()
            .as_str()
    );
    assert!(error.decisive_attempt().is_none());
    assert!(Error::source(&error).is_none());
    let display = error.to_string();
    assert!(display.contains("no provider succeeded after 2 attempt(s)"));
    assert!(display.contains("attempt 1: provider first"));
    assert!(display.contains("attempt 2: provider second"));
}

/// Verifies diagnostics when fallback policy stops after one decisive failure.
#[test]
fn test_policy_stopped_creation_error_exposes_decisive_source() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(
        &registry,
        "invalid",
        20,
        ProviderError::invalid_configuration("bad config"),
    );
    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("fallback")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(10),
            ConfigurableProvider::success("fallback"),
        ))
        .expect("fallback provider should register");

    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect_err("invalid configuration should stop absence fallback");

    assert_eq!(
        Some(ProviderCreationTermination::StoppedByPolicy),
        error.termination(),
    );
    assert!(!error.is_absence());
    assert_eq!(
        "invalid",
        error
            .decisive_attempt()
            .expect("policy stop should have one decisive attempt")
            .provider_id()
            .as_str()
    );
    assert!(Error::source(&error).is_some());
    assert!(error.to_string().contains(
        "provider creation stopped by fallback policy after 1 attempt(s)"
    ));
}

/// Verifies that singleton exhaustion has one unambiguous source.
#[test]
fn test_singleton_exhaustion_exposes_decisive_source() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(
        &registry,
        "remote",
        0,
        ProviderError::unavailable("offline"),
    );
    let selection = ProviderSelection::named("remote")
        .expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect("named selection should resolve")
        .create()
        .expect_err("selected provider should fail");

    assert_eq!(
        Some(ProviderCreationTermination::Exhausted),
        error.termination(),
    );
    assert!(error.decisive_attempt().is_some());
    assert!(Error::source(&error).is_some());
}

/// Registers one failing test provider.
///
/// # Arguments
///
/// * `registry` - Runtime registry receiving the provider.
/// * `id` - Canonical provider identity.
/// * `priority` - Descending automatic-selection priority.
/// * `error` - Leaf failure returned during service creation.
fn register_failure(
    registry: &ProviderRegistry<StringSpec>,
    id: &str,
    priority: i32,
    error: ProviderError,
) {
    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new(id).expect("test provider ID should be valid"),
            )
            .with_priority(priority),
            ConfigurableProvider::failure(error),
        ))
        .expect("unique test provider should register");
}
