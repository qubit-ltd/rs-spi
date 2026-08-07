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

/// Verifies resolved candidates remain a point-in-time snapshot.
#[test]
fn test_resolved_candidates_do_not_observe_later_registrations() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(SelfDescribedProvider::new(
            ProviderDescriptor::new(
                ProviderId::new("first").expect("static ID should be valid"),
            ),
            "first",
        ))
        .expect("first provider should register");
    let snapshot = registry.resolve().expect("first snapshot should resolve");

    registry
        .register(SelfDescribedProvider::new(
            ProviderDescriptor::new(
                ProviderId::new("second").expect("static ID should be valid"),
            )
            .with_priority(100),
            "second",
        ))
        .expect("second provider should register");

    assert_eq!(
        "first",
        snapshot
            .create_configured(&String::new())
            .expect("snapshot should create from its original candidate"),
    );
    assert_eq!(
        "second",
        registry
            .resolve()
            .expect("new snapshot should resolve")
            .create_configured(&String::new())
            .expect("new snapshot should use the higher priority provider"),
    );
}
