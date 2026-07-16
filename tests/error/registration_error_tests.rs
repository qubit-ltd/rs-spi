// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::{
    ProviderError,
    RegistrationError,
};
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ServiceProvider,
    ServiceSpec,
};

/// Service family used to create a registration conflict through public APIs.
struct ConflictSpec;

impl ServiceSpec for ConflictSpec {
    type Config = ();
    type Output = ();
}

/// Provider used only to populate the conflict-test builder.
struct EmptyProvider;

impl ServiceProvider<ConflictSpec> for EmptyProvider {
    /// Creates the unit service used by registration tests.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused unit configuration.
    ///
    /// # Returns
    ///
    /// The unit service.
    fn create(&self, _config: &()) -> Result<(), ProviderError> {
        Ok(())
    }
}

/// Verifies that builder conflicts expose both providers and the selector.
#[test]
fn test_registration_error_exposes_its_variant_and_conflict_details() {
    let mut builder = ProviderRegistry::<ConflictSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            EmptyProvider,
        )
        .expect("first provider should register");
    let error = builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("spanish")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["en"])
            .expect("test alias should be valid"),
            EmptyProvider,
        )
        .expect_err("duplicate alias should be rejected");

    let RegistrationError::DuplicateSelector {
        selector,
        existing_provider,
        provider,
    } = error;
    assert_eq!("en", selector.as_ref());
    assert_eq!("english", existing_provider.as_ref());
    assert_eq!("spanish", provider.as_ref());
}
