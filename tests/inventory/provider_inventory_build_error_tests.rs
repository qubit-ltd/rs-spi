// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderRegistrationSource;
use qubit_spi::error::ProviderInventoryBuildError;
use qubit_spi::error::RegistryMutationError;

/// Verifies that an inventory registration error retains its source and cause.
#[test]
fn test_inventory_build_error_retains_registration_source_and_cause() {
    let source = ProviderRegistrationSource::new("provider-a", "backend", "src/lib.rs", 17);
    let mutation = RegistryMutationError::DuplicateSelector {
        selector: "shared".into(),
        existing_provider: "existing".into(),
        provider: "attempted".into(),
    };
    let error = ProviderInventoryBuildError::registration(source, mutation);

    assert_eq!(source, error.source_location());
    assert_eq!(Some("shared"), error.registration_error().selector());
}
