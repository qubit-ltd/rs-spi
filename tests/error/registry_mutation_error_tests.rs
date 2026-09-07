// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;
use qubit_spi::error::RegistryMutationError;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies that registry conflicts expose both providers and the selector.
#[test]
fn test_registry_mutation_error_exposes_its_variant_and_conflict_details() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid"))
                .with_aliases(["en"])
                .expect("test alias should be valid"),
            ConfigurableProvider::success("unused"),
        ))
        .expect("first provider should register");
    let error = registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("spanish").expect("test provider ID should be valid"))
                .with_aliases(["en"])
                .expect("test alias should be valid"),
            ConfigurableProvider::success("unused"),
        ))
        .expect_err("duplicate alias should be rejected");

    assert!(matches!(&error, RegistryMutationError::DuplicateSelector { .. },));
    assert_eq!(Some("en"), error.selector());
    assert_eq!(Some("english"), error.existing_provider());
    assert_eq!(Some("spanish"), error.provider());
}

#[test]
fn test_sealed_error_has_no_duplicate_selector_context() {
    let error = RegistryMutationError::Sealed;
    assert!(error.is_sealed());
    assert_eq!(None, error.selector());
    assert_eq!(None, error.existing_provider());
    assert_eq!(None, error.provider());
    assert_eq!("provider registry is sealed", error.to_string());
}
