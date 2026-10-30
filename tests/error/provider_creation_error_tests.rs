// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::fmt::Write;

use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderCreationTermination;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;
use qubit_spi::error::ProviderAttemptFailure;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderFailureKind;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::failing_writer::FailingWriter;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;
use crate::common::test_provider_definition::define_provider;

/// Verifies diagnostics for exhaustion after multiple absence failures.
#[test]
fn test_exhausted_creation_error_exposes_ordered_ambiguous_diagnostics() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(
        &registry,
        "first",
        20,
        TestProviderFailure::unavailable_with_source(
            "first unavailable",
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "first cause"),
        ),
    );
    register_failure(
        &registry,
        "second",
        10,
        TestProviderFailure::unavailable_with_source(
            "second unavailable",
            std::io::Error::new(std::io::ErrorKind::NotFound, "second cause"),
        ),
    );
    let selection = ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("both providers should fail");

    assert_eq!(ProviderCreationTermination::Exhausted, error.termination(),);
    assert!(error.is_absence(), "absence failures should exhaust the fallback chain");
    let source = Error::source(&error).expect("aggregate must expose a source");
    let attempt = source
        .downcast_ref::<ProviderAttemptFailure<TestError>>()
        .expect("aggregate source must be a provider attempt");
    assert!(std::ptr::eq(error.decisive_attempt(), attempt));
    assert_eq!("second", attempt.provider_id().as_str());
    let classified = Error::source(attempt)
        .and_then(|cause| cause.downcast_ref::<ProviderFailure<TestError>>())
        .expect("attempt source must retain the classified failure");
    assert_eq!(ProviderFailureKind::Unavailable, classified.kind());
    let domain = Error::source(classified)
        .and_then(|cause| cause.downcast_ref::<TestError>())
        .expect("classified failure must retain the domain error");
    assert_eq!("second unavailable", domain.reason());
    let io_cause = Error::source(domain)
        .and_then(|cause| cause.downcast_ref::<std::io::Error>())
        .expect("domain error must retain its IO cause");
    assert_eq!(std::io::ErrorKind::NotFound, io_cause.kind());
    assert_eq!("second cause", io_cause.to_string());
    assert!(Error::source(io_cause).is_none());
    let display = error.to_string();
    assert!(
        display.contains("no provider succeeded after 2 attempt(s)"),
        "the aggregate diagnostic should report exhaustion after two attempts",
    );
    assert!(
        display.contains("attempt 1: provider first"),
        "the aggregate diagnostic should preserve the first provider attempt",
    );
    assert!(
        display.contains("attempt 2: provider second"),
        "the aggregate diagnostic should preserve the second provider attempt",
    );
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
            ProviderDescriptor::new(ProviderId::new("fallback").expect("test provider ID should be valid"))
                .with_priority(10),
            ConfigurableProvider::success("fallback"),
        ))
        .expect("fallback provider should register");

    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect_err("invalid configuration should stop absence fallback");

    assert_eq!(ProviderCreationTermination::StoppedByPolicy, error.termination(),);
    assert!(
        !error.is_absence(),
        "invalid configuration should not be classified as absence"
    );
    assert_eq!("invalid", error.decisive_attempt().provider_id().as_str());
    let source = Error::source(&error).expect("stopped error should expose its source");
    let attempt = source
        .downcast_ref::<ProviderAttemptFailure<TestError>>()
        .expect("stopped aggregate source should be a provider attempt");
    assert!(std::ptr::eq(error.decisive_attempt(), attempt));
    let mut display = String::new();
    write!(&mut display, "{error}").expect("display formatting should succeed");
    assert!(
        display.contains("provider creation stopped by fallback policy after 1 attempt(s)"),
        "the diagnostic should explain why fallback stopped after one attempt",
    );
}

/// Verifies that singleton exhaustion has one unambiguous source.
#[test]
fn test_singleton_exhaustion_exposes_decisive_source() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_failure(&registry, "remote", 0, TestProviderFailure::unavailable("offline"));
    let selection = ProviderSelection::named("remote").expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect("named selection should resolve")
        .create()
        .expect_err("selected provider should fail");

    assert_eq!(ProviderCreationTermination::Exhausted, error.termination(),);
    let source = Error::source(&error).expect("singleton aggregate should expose its source");
    let attempt = source
        .downcast_ref::<ProviderAttemptFailure<TestError>>()
        .expect("singleton aggregate source should be a provider attempt");
    assert!(std::ptr::eq(error.decisive_attempt(), attempt));
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
            ProviderDescriptor::new(ProviderId::new(id).expect("test provider ID should be valid"))
                .with_priority(priority),
            ConfigurableProvider::failure(error),
        ))
        .expect("unique test provider should register");
}
