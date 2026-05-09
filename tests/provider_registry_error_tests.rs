mod support;

use qubit_spi::{
    ProviderFailure,
    ProviderRegistryError,
};

use crate::support::test_services::TestProviderError;

/// Test provider-name validation errors are readable.
#[test]
fn test_empty_provider_name_error_display() {
    let error = ProviderRegistryError::<TestProviderError>::EmptyProviderName;

    assert_eq!("provider name must not be empty", error.to_string());
}

/// Test duplicate provider-name errors include the duplicated name.
#[test]
fn test_duplicate_provider_name_error_display() {
    let error = ProviderRegistryError::<TestProviderError>::DuplicateProviderName {
        name: "native".to_owned(),
    };

    assert_eq!("duplicate provider name: native", error.to_string());
}

/// Test unknown provider errors include the requested selector.
#[test]
fn test_unknown_provider_error_display() {
    let error = ProviderRegistryError::<TestProviderError>::UnknownProvider {
        name: "missing".to_owned(),
    };

    assert_eq!("unknown provider: missing", error.to_string());
}

/// Test unavailable provider errors include the requested selector and reason.
#[test]
fn test_provider_unavailable_error_display() {
    let error = ProviderRegistryError::<TestProviderError>::ProviderUnavailable {
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
        error: TestProviderError::new("boom"),
    };

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
            ProviderFailure::<TestProviderError>::unknown("missing"),
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
    let error = ProviderRegistryError::<TestProviderError>::EmptyRegistry;

    assert_eq!("provider registry is empty", error.to_string());
}
