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

/// Verifies a registered entry retains its descriptor snapshot.
#[test]
fn test_registry_entry_retains_provider_descriptor() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(SelfDescribedProvider::new(
            ProviderDescriptor::new(
                ProviderId::new("entry").expect("static ID should be valid"),
            )
            .with_priority(41),
            "value",
        ))
        .expect("provider should register");

    let descriptors = registry.descriptors();
    assert_eq!("entry", descriptors[0].id().as_str());
    assert_eq!(41, descriptors[0].priority());
}
