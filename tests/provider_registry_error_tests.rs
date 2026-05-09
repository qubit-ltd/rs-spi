mod support;

use qubit_spi::{
    ProviderFailure,
    ProviderRegistryError,
};

/// Test provider-name validation errors are readable.
#[test]
fn test_empty_provider_name_error_display() {
    let error = ProviderRegistryError::EmptyProviderName;

    assert_eq!("provider name must not be empty", error.to_string());
}

/// Test duplicate provider-name errors include the duplicated name.
#[test]
fn test_duplicate_provider_name_error_display() {
    let error = ProviderRegistryError::DuplicateProviderName {
        name: "native".to_owned(),
    };

    assert_eq!("duplicate provider name: native", error.to_string());
}

/// Test unknown provider errors include the requested selector.
#[test]
fn test_unknown_provider_error_display() {
    let error = ProviderRegistryError::UnknownProvider {
        name: "missing".to_owned(),
    };

    assert_eq!("unknown provider: missing", error.to_string());
}

/// Test unavailable provider errors include the requested selector and reason.
#[test]
fn test_provider_unavailable_error_display() {
    let error = ProviderRegistryError::ProviderUnavailable {
        name: "native".to_owned(),
        reason: "not installed".to_owned(),
    };

    assert_eq!(
        "provider 'native' is unavailable: not installed",
        error.to_string(),
    );
}

/// Test provider creation errors include the requested selector and source message.
#[test]
fn test_provider_create_error_display() {
    let error = ProviderRegistryError::ProviderCreate {
        name: "native".to_owned(),
        reason: "boom".to_owned(),
    };

    assert_eq!(
        "provider 'native' failed to create service: boom",
        error.to_string(),
    );
}

/// Test unnamed provider creation errors are readable before registry context is attached.
#[test]
fn test_create_failed_error_display_without_provider_name() {
    let error = ProviderRegistryError::create_failed("boom");

    assert_eq!("provider failed to create service: boom", error.to_string());
}

/// Test the named provider creation helper preserves the provider name.
#[test]
fn test_provider_create_helper_display() {
    let error = ProviderRegistryError::provider_create("native", "boom");

    assert_eq!(
        "provider 'native' failed to create service: boom",
        error.to_string(),
    );
}

/// Test aggregate selection errors summarize all failed candidates.
#[test]
fn test_no_available_provider_error_display() {
    let error = ProviderRegistryError::NoAvailableProvider {
        failures: vec![
            ProviderFailure::unknown("missing"),
            ProviderFailure::unavailable("native", "not installed"),
        ],
    };

    assert_eq!(
        "no available provider; candidate failures: unknown provider: missing; provider 'native' is unavailable: not installed",
        error.to_string(),
    );
}

/// Test empty registries have a distinct error.
#[test]
fn test_empty_registry_error_display() {
    let error = ProviderRegistryError::EmptyRegistry;

    assert_eq!("provider registry is empty", error.to_string());
}
