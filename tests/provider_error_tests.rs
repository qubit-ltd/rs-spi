// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::{
    ProviderError,
    ProviderErrorKind,
};

/// Verifies provider error classification, reason, and causal source retention.
#[test]
fn test_provider_error_preserves_kind_reason_and_source() {
    let error = ProviderError::unavailable_with_source(
        "file executable is absent",
        std::io::Error::other("ENOENT"),
    );

    assert_eq!(ProviderErrorKind::Unavailable, error.kind());
    assert_eq!("file executable is absent", error.reason());
    assert!(std::error::Error::source(&error).is_some());
}

/// Verifies initialization failures retain their original causal error.
#[test]
fn test_initialization_failure_preserves_its_source() {
    let error = ProviderError::initialization_failed_with_source(
        "runtime bootstrap failed",
        std::io::Error::other("runtime unavailable"),
    );

    assert_eq!(ProviderErrorKind::InitializationFailed, error.kind());
    assert_eq!("runtime bootstrap failed", error.reason());
    assert!(std::error::Error::source(&error).is_some());
}
