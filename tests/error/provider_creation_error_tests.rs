// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{
    error::Error,
    fmt::Write,
};

use qubit_spi::error::ProviderFailure;
use qubit_spi::{
    FallbackPolicy,
    ProviderCreationTermination,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderSelection,
};

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::failing_writer::FailingWriter;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::{
    TestError,
    TestProviderFailure,
};
use crate::common::test_provider_definition::define_provider;

/// Verifies diagnostics for exhaustion after multiple absence failures.
#[test]
fn test_exhausted_creation_error_exposes_ordered_ambiguous_diagnostics() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(
        &registry,
        "first",
        20,
        TestProviderFailure::unavailable("first unavailable"),
    );
    register_failure(
        &registry,
        "second",
        10,
        TestProviderFailure::unsupported("second unsupported"),
    );
    let selection = ProviderSelection::auto()
        .with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("both providers should fail");

    assert_eq!(ProviderCreationTermination::Exhausted, error.termination(),);
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
    assert!(std::ptr::eq(
        error
            .attempts()
            .last()
            .expect("aggregate should have a final attempt"),
        error.decisive_attempt(),
    ));
    assert!(Error::source(&error).is_some());
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
        TestProviderFailure::invalid_configuration("bad config"),
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
        ProviderCreationTermination::StoppedByPolicy,
        error.termination(),
    );
    assert!(!error.is_absence());
    assert_eq!("invalid", error.decisive_attempt().provider_id().as_str());
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
        TestProviderFailure::unavailable("offline"),
    );
    let selection = ProviderSelection::named("remote")
        .expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect("named selection should resolve")
        .create()
        .expect_err("selected provider should fail");

    assert_eq!(ProviderCreationTermination::Exhausted, error.termination(),);
    assert!(std::ptr::eq(
        error
            .attempts()
            .last()
            .expect("aggregate should have a final attempt"),
        error.decisive_attempt(),
    ));
    assert!(Error::source(&error).is_some());
}

/// Verifies that aggregate diagnostics propagate formatter failures.
#[test]
fn test_aggregate_creation_error_propagates_formatter_failures() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(
        &registry,
        "first",
        10,
        TestProviderFailure::unavailable("first unavailable"),
    );
    register_failure(
        &registry,
        "second",
        0,
        TestProviderFailure::unavailable("second unavailable"),
    );
    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect_err("both providers should fail");

    assert!(
        write!(&mut FailingWriter::new(0), "{error}").is_err(),
        "header write failure should propagate",
    );
    assert!(
        write!(&mut FailingWriter::new(1), "{error}").is_err(),
        "attempt write failure should propagate",
    );
}

/// Registers one failing test provider.
///
/// # Parameters
///
/// * `registry` - Runtime registry receiving the provider.
/// * `id` - Canonical provider identity.
/// * `priority` - Descending automatic-selection priority.
/// * `error` - Leaf failure returned during service creation.
fn register_failure(
    registry: &ProviderRegistry<StringSpec>,
    id: &str,
    priority: i32,
    error: ProviderFailure<TestError>,
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
