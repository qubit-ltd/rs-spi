// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use futures::executor::block_on;
use qubit_spi::AsyncServiceProvider;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderFailureKind;

use crate::common::async_configurable_provider::AsyncConfigurableProvider;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;

/// Verifies explicit and default asynchronous provider creation.
#[test]
fn test_async_provider_creates_with_explicit_and_default_config() {
    let provider = AsyncConfigurableProvider::echo();

    assert_eq!(
        "explicit",
        block_on(provider.create_configured(&"explicit".to_owned()))
            .expect("explicit async creation should succeed"),
    );
    assert_eq!(
        String::default(),
        block_on(provider.create())
            .expect("default async creation should succeed"),
    );
}

/// Verifies that asynchronous leaf providers return their typed failure
/// directly.
#[test]
fn test_async_leaf_provider_returns_typed_failure_directly() {
    let provider = AsyncConfigurableProvider::failure(
        TestProviderFailure::unavailable("offline"),
    );
    let config = String::new();
    let future = provider.create_configured(&config);

    fn assert_send<T: Send>(_: &T) {}
    assert_send(&future);
    let error: ProviderFailure<TestError> =
        block_on(future).expect_err("async leaf provider should fail");
    assert_eq!(ProviderFailureKind::Unavailable, error.kind());
}

/// Verifies stable asynchronous provider output.
#[test]
fn test_async_provider_returns_stable_output() {
    let provider = AsyncConfigurableProvider::success("stable");
    assert_eq!(
        "stable",
        block_on(provider.create_configured(&String::new()))
            .expect("stable async creation should succeed"),
    );
}
