// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
};

use crate::common::self_described_provider::SelfDescribedProvider;
use crate::common::string_spec::StringSpec;

/// Verifies that registration obtains metadata from the Provider itself.
#[test]
fn test_definition_supplies_registration_descriptor() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(SelfDescribedProvider::new(
            ProviderDescriptor::new(
                ProviderId::new("english")
                    .expect("test provider ID should be valid"),
            ),
            "hello",
        ))
        .expect("self-described provider should register");

    assert_eq!(1, registry.len());
    assert_eq!("english", registry.provider_ids()[0].as_str());
}
