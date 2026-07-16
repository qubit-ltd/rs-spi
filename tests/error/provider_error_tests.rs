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

/// Verifies an already owned reason is transferred without copying.
#[test]
fn test_provider_error_transfers_an_owned_reason() {
    let reason: Box<str> = "owned provider reason".into();
    let reason_pointer = reason.as_ptr();

    let error = ProviderError::unavailable(reason);

    assert_eq!(reason_pointer, error.reason().as_ptr());
    assert_eq!("owned provider reason", error.reason());
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

/// Verifies unsupported failures can retain their original causal error.
#[test]
fn test_unsupported_failure_preserves_its_source() {
    let error = ProviderError::unsupported_with_source(
        "requested capability is unavailable",
        std::io::Error::other("capability disabled"),
    );

    assert_eq!(ProviderErrorKind::Unsupported, error.kind());
    assert_eq!("requested capability is unavailable", error.reason());
    assert!(std::error::Error::source(&error).is_some());
}

/// Verifies invalid configurations can retain their causal error.
#[test]
fn test_invalid_configuration_preserves_its_source() {
    let error = ProviderError::invalid_configuration_with_source(
        "invalid provider setting",
        std::io::Error::other("value is out of range"),
    );

    assert_eq!(ProviderErrorKind::InvalidConfiguration, error.kind());
    assert_eq!("invalid provider setting", error.reason());
    assert!(std::error::Error::source(&error).is_some());
}
