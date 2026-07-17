// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::error::{
    ProviderError,
    ProviderErrorKind,
};
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderSelection,
    ServiceProvider,
};

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies attempt accessors, display text, and standard error chaining.
#[test]
fn test_provider_attempt_failure_exposes_public_diagnostics() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            ),
            ConfigurableProvider::failure(
                ProviderError::unavailable_with_source(
                    "runtime is absent",
                    std::io::Error::other("ENOENT"),
                ),
            ),
        ))
        .expect("test provider should register");
    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect_err("test provider should fail");
    let attempt = &error.attempts()[0];

    assert_eq!("remote", attempt.provider_id().as_str());
    assert_eq!(ProviderErrorKind::Unavailable, attempt.error().kind());
    assert_eq!("runtime is absent", attempt.error().reason());
    assert!(attempt.to_string().contains("remote"));
    assert!(attempt.to_string().contains("runtime is absent"));
    assert!(Error::source(attempt).is_some());
    assert!(Error::source(attempt).and_then(Error::source).is_some());
}
