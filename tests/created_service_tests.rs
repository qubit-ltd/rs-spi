// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderError;
use qubit_spi::{
    CreatedService,
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ServiceProvider,
    ServiceSpec,
};

/// Service specification used to create a small scalar test service.
struct NumberSpec;

impl ServiceSpec for NumberSpec {
    type Config = ();
    type Output = u8;
}

/// Provider returning the stable scalar used by CreatedService assertions.
struct NumberProvider;

impl ServiceProvider<NumberSpec> for NumberProvider {
    /// Creates the scalar test service.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused configuration for the scalar service.
    ///
    /// # Returns
    ///
    /// The stable value `42`.
    fn create(&self, _config: &()) -> Result<u8, ProviderError> {
        Ok(42)
    }
}

/// Creates a service result through the public registry and resolver API.
///
/// # Returns
///
/// A scalar service attributed to the canonical `memory` provider.
fn create_number() -> CreatedService<u8> {
    let mut builder = ProviderRegistry::<NumberSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            ),
            NumberProvider,
        )
        .expect("test provider should register");
    ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
        .create_named("memory", &())
        .expect("registered provider should create its service")
}

/// Verifies that a created service exposes its winning provider and service.
#[test]
fn test_created_service_contains_the_winning_provider() {
    let created = create_number();

    assert_eq!("memory", created.provider_id().as_str());
    assert_eq!(&42, created.service());
    assert_eq!(42, created.into_service());
}

/// Verifies that a created service decomposes into both owned fields.
#[test]
fn test_created_service_decomposes_into_owned_parts() {
    let created = create_number();
    let (provider_id, service) = created.into_parts();

    assert_eq!("memory", provider_id.as_str());
    assert_eq!(42, service);
}
