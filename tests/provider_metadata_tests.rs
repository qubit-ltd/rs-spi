// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderMetadata,
};

use crate::common::self_described_provider::SelfDescribedProvider;

/// Verifies that provider metadata returns an owned descriptor snapshot.
#[test]
fn test_provider_metadata_exposes_registration_descriptor() {
    let provider = SelfDescribedProvider::new(
        ProviderDescriptor::new(
            ProviderId::new("metadata")
                .expect("test provider ID should be valid"),
        ),
        "output",
    );

    assert_eq!("metadata", provider.descriptor().id().as_str());
}
