mod support;

use qubit_spi::ProviderFailure;

use crate::support::test_services::TestProviderError;

/// Test unknown provider failures expose their candidate name.
#[test]
fn test_unknown_provider_failure_display_and_name() {
    let failure = ProviderFailure::<TestProviderError>::unknown("missing");

    assert_eq!("missing", failure.name());
    assert_eq!("unknown provider: missing", failure.to_string());
}

/// Test unavailable provider failures include the reason.
#[test]
fn test_unavailable_provider_failure_display_and_name() {
    let failure = ProviderFailure::<TestProviderError>::unavailable("native", "not installed");

    assert_eq!("native", failure.name());
    assert_eq!(
        "provider 'native' is unavailable: not installed",
        failure.to_string(),
    );
}

/// Test creation failures keep the underlying provider error.
#[test]
fn test_create_failed_provider_failure_display_and_name() {
    let failure = ProviderFailure::create_failed("native", TestProviderError::new("boom"));

    assert_eq!("native", failure.name());
    assert_eq!(
        "provider 'native' failed to create service: boom",
        failure.to_string(),
    );
}
