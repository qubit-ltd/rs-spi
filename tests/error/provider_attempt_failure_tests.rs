// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::ProviderCreationTermination;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderFailureKind;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;
use crate::common::test_provider_definition::define_provider;

/// Verifies attempt accessors, display text, and standard error chaining.
#[test]
fn test_provider_attempt_failure_exposes_public_diagnostics() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("remote").expect("test provider ID should be valid")),
            ConfigurableProvider::failure(TestProviderFailure::unavailable_with_source(
                "runtime is absent",
                std::io::Error::new(std::io::ErrorKind::NotFound, "runtime unavailable"),
            )),
        ))
        .expect("test provider should register");
    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect_err("test provider should fail");
    let attempt = &error.attempts()[0];

    assert_eq!("remote", attempt.provider_id().as_str());
    assert_eq!(ProviderFailureKind::Unavailable, attempt.failure().kind());
    assert_eq!("runtime is absent", attempt.failure().error().reason());
    assert!(attempt.to_string().contains("remote"));
    assert!(attempt.to_string().contains("runtime is absent"));
    let classified = Error::source(attempt)
        .and_then(|cause| cause.downcast_ref::<ProviderFailure<TestError>>())
        .expect("attempt source must be the classified failure");
    assert_eq!(ProviderFailureKind::Unavailable, classified.kind());
    let domain = Error::source(classified)
        .and_then(|cause| cause.downcast_ref::<TestError>())
        .expect("classified failure must retain the domain error");
    assert_eq!("runtime is absent", domain.reason());
    let io_cause = Error::source(domain)
        .and_then(|cause| cause.downcast_ref::<std::io::Error>())
        .expect("domain error must retain its IO cause");
    assert_eq!(std::io::ErrorKind::NotFound, io_cause.kind());
    assert_eq!("runtime unavailable", io_cause.to_string());
    assert!(Error::source(io_cause).is_none());
}

/// Verifies aggregate and attempt diagnostics transfer their owned parts
/// intact.
#[test]
fn test_provider_attempt_failure_into_parts_preserves_identity_and_failure() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("remote").expect("test provider ID should be valid")),
            ConfigurableProvider::failure(TestProviderFailure::unavailable("offline")),
        ))
        .expect("test provider should register");
    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect_err("test provider should fail");

    let (attempts, termination) = error.into_parts();
    let mut attempts = attempts.into_vec();
    let attempt = attempts.pop().expect("aggregate should retain the attempted provider");
    let (provider_id, failure) = attempt.into_parts();

    assert!(attempts.is_empty());
    assert_eq!(ProviderCreationTermination::Exhausted, termination);
    assert_eq!("remote", provider_id.as_str());
    assert_eq!(ProviderFailureKind::Unavailable, failure.kind());
    assert_eq!("offline", failure.error().reason());
}
