mod support;

use std::error::Error;
use std::io;

use qubit_spi::{ProviderCreateError, ProviderFailure, ProviderName, ProviderRegistryError};

/// Creates a provider name used by error assertions.
fn name(value: &str) -> ProviderName {
    ProviderName::new(value).expect("test provider name should be valid")
}

/// Test provider-name validation errors are readable.
#[test]
fn test_empty_provider_name_error_display() {
    let error = ProviderRegistryError::EmptyProviderName;

    assert_eq!("provider name must not be empty", error.to_string());
}

/// Test invalid provider-name errors include the invalid value and reason.
#[test]
fn test_invalid_provider_name_error_display() {
    let error = ProviderRegistryError::InvalidProviderName {
        name: "bad name".to_owned(),
        reason: "provider names may contain only ASCII letters, digits, '_' or '-'".to_owned(),
    };

    assert_eq!(
        "invalid provider name 'bad name': provider names may contain only ASCII letters, digits, '_' or '-'",
        error.to_string(),
    );
}

/// Test duplicate provider-name errors include the duplicated name.
#[test]
fn test_duplicate_provider_name_error_display() {
    let error = ProviderRegistryError::DuplicateProviderName {
        name: name("native"),
    };

    assert_eq!("duplicate provider name: native", error.to_string());
}

/// Test duplicate selection-candidate errors include the duplicated name.
#[test]
fn test_duplicate_provider_candidate_error_display() {
    let error = ProviderRegistryError::DuplicateProviderCandidate {
        name: name("native"),
    };

    assert_eq!(
        "duplicate provider candidate in selection: native",
        error.to_string(),
    );
}

/// Test unknown provider errors include the requested selector.
#[test]
fn test_unknown_provider_error_display() {
    let error = ProviderRegistryError::UnknownProvider {
        name: name("missing"),
    };

    assert_eq!("unknown provider: missing", error.to_string());
}

/// Test unavailable provider errors include the requested selector and reason.
#[test]
fn test_provider_unavailable_error_display() {
    let error = ProviderRegistryError::ProviderUnavailable {
        name: name("native"),
        source: ProviderCreateError::unavailable("not installed"),
    };

    assert_eq!(
        "provider 'native' is unavailable: not installed",
        error.to_string(),
    );
    assert_eq!(
        "provider is unavailable: not installed",
        Error::source(&error)
            .expect("source should exist")
            .to_string(),
    );
}

/// Test provider creation errors include the requested selector and source
/// message.
#[test]
fn test_provider_create_error_display() {
    let error = ProviderRegistryError::ProviderCreate {
        name: name("native"),
        source: ProviderCreateError::failed("boom"),
    };

    assert_eq!(
        "provider 'native' failed to create service: boom",
        error.to_string(),
    );
    assert_eq!(
        "provider failed to create service: boom",
        Error::source(&error)
            .expect("source should exist")
            .to_string(),
    );
}

/// Test provider creation errors keep nested source errors.
#[test]
fn test_provider_create_error_preserves_nested_source() {
    let error = ProviderRegistryError::ProviderCreate {
        name: name("native"),
        source: ProviderCreateError::failed_with_source("boom", io::Error::other("root cause")),
    };
    let provider_error = Error::source(&error).expect("provider source should exist");

    assert_eq!(
        "provider 'native' failed to create service: boom",
        error.to_string()
    );
    assert_eq!(
        "root cause",
        provider_error
            .source()
            .expect("nested source should exist")
            .to_string(),
    );
}

/// Test aggregate selection errors summarize all failed candidates.
#[test]
fn test_no_available_provider_error_display() {
    let error = ProviderRegistryError::NoAvailableProvider {
        failures: vec![
            ProviderFailure::unknown("missing").expect("valid provider name"),
            ProviderFailure::unavailable("native", "not installed").expect("valid provider name"),
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
