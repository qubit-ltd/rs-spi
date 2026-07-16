// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ResolutionError;
use qubit_spi::{
    ProviderRegistry,
    ServiceSpec,
};

/// Empty service family used to exercise registry resolution failures.
struct EmptySpec;

impl ServiceSpec for EmptySpec {
    type Config = ();
    type Output = ();
}

/// Verifies that direct lookup exposes its normalized unknown selector.
#[test]
fn test_resolution_error_exposes_unknown_provider_selector() {
    let error =
        match ProviderRegistry::<EmptySpec>::default().resolve("missing") {
            Ok(_) => panic!("an empty registry cannot resolve a provider"),
            Err(error) => error,
        };

    let ResolutionError::UnknownProvider { selector } = &error else {
        panic!("an empty registry lookup should report an unknown provider");
    };
    assert_eq!("missing", selector.as_str());
    assert!(error.attempts().is_empty());
    assert!(error.termination().is_none());
    assert!(error.terminal_attempt().is_none());
    assert!(error.is_absence());
}
