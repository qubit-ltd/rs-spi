mod support;

use qubit_spi::{
    ProviderFailure,
    ProviderRegistryError,
};

/// Test unknown provider failures expose their candidate name.
#[test]
fn test_unknown_provider_failure_display_and_name() {
    let failure = ProviderFailure::unknown("missing").expect("valid provider name");

    assert_eq!("missing", failure.name());
    assert_eq!("unknown provider: missing", failure.to_string());
}

/// Test unavailable provider failures include the reason.
#[test]
fn test_unavailable_provider_failure_display_and_name() {
    let failure =
        ProviderFailure::unavailable("native", "not installed").expect("valid provider name");

    assert_eq!("native", failure.name());
    assert_eq!(
        "provider 'native' is unavailable: not installed",
        failure.to_string(),
    );
}

/// Test creation failures keep the underlying provider error.
#[test]
fn test_create_failed_provider_failure_display_and_name() {
    let failure = ProviderFailure::create_failed("native", "boom").expect("valid provider name");

    assert_eq!("native", failure.name());
    assert_eq!("native", failure.provider_name().as_str());
    assert_eq!(
        "provider 'native' failed to create service: boom",
        failure.to_string(),
    );
}

/// Test public failure constructors reject invalid provider names.
#[test]
fn test_failure_constructors_reject_invalid_provider_names() {
    let unknown_error =
        ProviderFailure::unknown("missing provider").expect_err("invalid name should fail");
    let unavailable_error = ProviderFailure::unavailable(" ", "missing")
        .expect_err("empty name should fail before storing reason");
    let failed_error = ProviderFailure::create_failed("原生", "boom")
        .expect_err("non-ASCII name should fail before storing reason");

    assert!(matches!(
        unknown_error,
        ProviderRegistryError::InvalidProviderName { ref name, .. }
            if name == "missing provider"
    ));
    assert!(matches!(
        unavailable_error,
        ProviderRegistryError::EmptyProviderName
    ));
    assert!(matches!(
        failed_error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "原生"
    ));
}
