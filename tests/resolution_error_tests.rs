// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    ProviderRegistry,
    ProviderSelector,
    ResolutionErrorKind,
    ServiceSpec,
};

/// Empty service family used to exercise registry resolution failures.
struct EmptySpec;

impl ServiceSpec for EmptySpec {
    type Config = ();
    type Output = ();
}

/// Verifies that direct lookup exposes the unknown-provider classification.
#[test]
fn test_resolution_error_exposes_its_kind_and_attempts() {
    let error =
        match ProviderRegistry::<EmptySpec>::default().resolve("missing") {
            Ok(_) => panic!("an empty registry cannot resolve a provider"),
            Err(error) => error,
        };

    assert_eq!(ResolutionErrorKind::UnknownProvider, error.kind());
    assert_eq!(Some("missing"), error.selector_input());
    assert_eq!(None, error.selector_index());
    assert!(error.selector_error().is_none());
    assert_eq!(
        Some("missing"),
        error.requested_selector().map(ProviderSelector::as_str),
    );
    assert!(error.attempts().is_empty());
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!("unknown provider: missing", error.to_string());
}
