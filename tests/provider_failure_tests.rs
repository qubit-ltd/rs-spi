mod support;

use std::error::Error;
use std::io;

use qubit_spi::{
    ProviderCreateError,
    ProviderFailure,
    ProviderRegistryError,
};

/// Test unknown provider failures expose their candidate name.
#[test]
fn test_unknown_provider_failure_display_and_name() {
    let failure = ProviderFailure::unknown("missing").expect("valid provider name");

    assert_eq!("missing", failure.name());
    assert_eq!("unknown provider: missing", failure.to_string());
    assert!(Error::source(&failure).is_none());
}

/// Test unavailable provider failures include the reason.
#[test]
fn test_unavailable_provider_failure_display_and_name() {
    let failure = ProviderFailure::unavailable("native", "not installed").expect("valid provider name");

    assert_eq!("native", failure.name());
    assert_eq!("provider 'native' is unavailable: not installed", failure.to_string(),);
    assert_eq!(
        "provider is unavailable: not installed",
        Error::source(&failure).expect("source should exist").to_string(),
    );
}

/// Test creation failures keep the underlying provider error.
#[test]
fn test_create_failed_provider_failure_display_and_name() {
    let failure = ProviderFailure::create_failed("native", "boom").expect("valid provider name");

    assert_eq!("native", failure.name());
    assert_eq!("native", failure.provider_name().as_str());
    assert_eq!("provider 'native' failed to create service: boom", failure.to_string(),);
    assert_eq!(
        "provider failed to create service: boom",
        Error::source(&failure).expect("source should exist").to_string(),
    );
}

/// Test candidate failures can preserve nested provider source errors.
#[test]
fn test_create_failed_provider_failure_preserves_nested_source() {
    let failure = ProviderFailure::create_failed_from_error(
        "native",
        ProviderCreateError::failed_with_source("boom", io::Error::other("root cause")),
    )
    .expect("valid provider name");
    let provider_error = Error::source(&failure).expect("provider error should exist");

    assert_eq!("provider 'native' failed to create service: boom", failure.to_string());
    assert_eq!("provider failed to create service: boom", provider_error.to_string());
    assert_eq!(
        "root cause",
        provider_error.source().expect("nested source should exist").to_string(),
    );
}

/// Test public failure constructors reject invalid provider names.
#[test]
fn test_failure_constructors_reject_invalid_provider_names() {
    let unknown_error = ProviderFailure::unknown("missing provider").expect_err("invalid name should fail");
    let unavailable_error =
        ProviderFailure::unavailable(" ", "missing").expect_err("empty name should fail before storing reason");
    let failed_error =
        ProviderFailure::create_failed("原生", "boom").expect_err("non-ASCII name should fail before storing reason");

    assert!(matches!(
        unknown_error,
        ProviderRegistryError::InvalidProviderName { ref name, .. }
            if name == "missing provider"
    ));
    assert!(matches!(unavailable_error, ProviderRegistryError::EmptyProviderName));
    assert!(matches!(
        failed_error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "原生"
    ));
}
