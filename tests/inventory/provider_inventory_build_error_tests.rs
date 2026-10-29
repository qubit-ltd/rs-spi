// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

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
    let error = ProviderInventoryBuildError::Registration {
        source,
        error: mutation.clone(),
    };

    assert_eq!(source, error.source_location());
    assert_eq!(Some(&mutation), error.registration_error());
    assert!(error.source().is_some());
}

/// Verifies that source ambiguity has no fabricated registration cause.
#[test]
fn test_inventory_build_error_reports_ambiguous_source_without_registration_error() {
    let source = ProviderRegistrationSource::new("provider-a", "backend", "src/lib.rs", 17);
    let error = ProviderInventoryBuildError::AmbiguousSource { location: source };

    assert_eq!(source, error.source_location());
    assert!(error.registration_error().is_none());
    assert!(error.source().is_none());
    assert!(error.to_string().contains("provider-a::backend, src/lib.rs:17"));
    assert!(error.to_string().contains("no factories were run"));
}
