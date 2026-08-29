// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;

use crate::common::self_described_provider::SelfDescribedProvider;
use crate::common::string_spec::StringSpec;

/// Verifies Registry facades share their underlying provider catalog.
#[test]
fn test_provider_catalog_is_shared_by_registry_clones() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let clone = registry.clone();
    registry
        .register(SelfDescribedProvider::new(
            ProviderDescriptor::new(ProviderId::new("shared").expect("static ID should be valid")),
            "value",
        ))
        .expect("provider should register");

    assert_eq!(1, clone.len());
    assert_eq!("shared", clone.provider_ids()[0].as_str());
}
