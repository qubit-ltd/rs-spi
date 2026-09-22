use provider_alpha as _;
use provider_duplicate as _;

/// Proves that duplicate discovered providers fail while building the registry.
fn main() {
    let error = service_contract::sync_providers::build_registry()
        .expect_err("duplicate alpha provider should fail registry construction");

    assert_eq!("provider-duplicate", error.source_location().crate_name());
    assert!(error.registration_error().to_string().contains("alpha"));
    println!("provider-duplicate");
}
