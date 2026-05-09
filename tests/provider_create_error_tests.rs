use qubit_spi::ProviderCreateError;

/// Test unavailable provider creation errors preserve their reason.
#[test]
fn test_unavailable_preserves_reason_and_display() {
    let error = ProviderCreateError::unavailable("native dependency is missing");

    assert!(matches!(
        error,
        ProviderCreateError::Unavailable { ref reason }
            if reason == "native dependency is missing"
    ));
    assert_eq!(
        "provider is unavailable: native dependency is missing",
        error.to_string(),
    );
}

/// Test failed provider creation errors preserve their reason.
#[test]
fn test_failed_preserves_reason_and_display() {
    let error = ProviderCreateError::failed("initialization failed");

    assert!(matches!(
        error,
        ProviderCreateError::Failed { ref reason } if reason == "initialization failed"
    ));
    assert_eq!(
        "provider failed to create service: initialization failed",
        error.to_string(),
    );
}
