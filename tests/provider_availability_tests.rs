use qubit_spi::ProviderAvailability;

/// Test available states report availability.
#[test]
fn test_is_available_returns_true_for_available_state() {
    assert!(ProviderAvailability::Available.is_available());
}

/// Test unavailable states report unavailability and preserve the reason.
#[test]
fn test_unavailable_preserves_reason_and_reports_false() {
    let availability = ProviderAvailability::unavailable("missing binary");

    assert!(!availability.is_available());
    assert!(matches!(
        availability,
        ProviderAvailability::Unavailable { ref reason } if reason == "missing binary"
    ));
}
