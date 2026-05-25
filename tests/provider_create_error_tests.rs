use std::error::Error;
use std::io;

use qubit_spi::ProviderCreateError;

/// Test unavailable provider creation errors preserve their reason.
#[test]
fn test_unavailable_preserves_reason_and_display() {
    let error = ProviderCreateError::unavailable("native dependency is missing");

    assert!(matches!(
        error,
        ProviderCreateError::Unavailable { ref reason, .. }
            if reason == "native dependency is missing"
    ));
    assert_eq!("native dependency is missing", error.reason());
    assert_eq!(
        "provider is unavailable: native dependency is missing",
        error.to_string(),
    );
    assert!(Error::source(&error).is_none());
}

/// Test failed provider creation errors preserve their reason.
#[test]
fn test_failed_preserves_reason_and_display() {
    let error = ProviderCreateError::failed("initialization failed");

    assert!(matches!(
        error,
        ProviderCreateError::Failed { ref reason, .. } if reason == "initialization failed"
    ));
    assert_eq!("initialization failed", error.reason());
    assert_eq!(
        "provider failed to create service: initialization failed",
        error.to_string(),
    );
    assert!(Error::source(&error).is_none());
}

/// Test provider creation errors can preserve a lower-level source error.
#[test]
fn test_failed_with_source_preserves_source_error() {
    let error = ProviderCreateError::failed_with_source("initialization failed", io::Error::other("boom"));

    assert_eq!("initialization failed", error.reason());
    assert_eq!("boom", Error::source(&error).expect("source should exist").to_string());
}

/// Test unavailable provider errors can preserve a lower-level source error.
#[test]
fn test_unavailable_with_source_preserves_source_error() {
    let error =
        ProviderCreateError::unavailable_with_source("native dependency is missing", io::Error::other("not found"));

    assert_eq!("native dependency is missing", error.reason());
    assert_eq!(
        "not found",
        Error::source(&error).expect("source should exist").to_string(),
    );
}
