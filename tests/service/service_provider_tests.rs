// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ServiceProvider;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderFailureKind;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;

/// Verifies that a provider returns the output handle selected by its spec.
#[test]
fn test_provider_creates_the_handle_selected_by_the_spec() {
    assert_eq!(
        "seven",
        ConfigurableProvider::success("seven")
            .create_configured(&String::new())
            .expect("test provider should create its output"),
    );
}

/// Verifies that default creation supplies the configuration default.
#[test]
fn test_provider_create_uses_config_default() {
    let output = ConfigurableProvider::echo()
        .create()
        .expect("default service creation should succeed");

    assert_eq!(String::default(), output);
}

/// Verifies that a leaf provider returns its classified error directly.
#[test]
fn test_leaf_provider_returns_typed_failure_directly() {
    let provider = ConfigurableProvider::failure(TestProviderFailure::unavailable("offline"));

    let error: ProviderFailure<TestError> = provider
        .create_configured(&String::new())
        .expect_err("leaf provider should fail directly");

    assert_eq!(ProviderFailureKind::Unavailable, error.kind());
}
