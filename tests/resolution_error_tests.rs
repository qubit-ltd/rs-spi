// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{ProviderRegistry, ResolutionErrorKind, ServiceSpec};

/// Empty service family used to exercise registry resolution failures.
struct EmptySpec;

impl ServiceSpec for EmptySpec {
    type Config = ();
    type Output = ();
}

/// Verifies that direct lookup exposes the unknown-provider classification.
#[test]
fn test_resolution_error_exposes_its_kind_and_attempts() {
    let error = match ProviderRegistry::<EmptySpec>::default().resolve("missing") {
        Ok(_) => panic!("an empty registry cannot resolve a provider"),
        Err(error) => error,
    };

    assert_eq!(ResolutionErrorKind::UnknownProvider, error.kind());
    assert!(error.attempts().is_empty());
}
