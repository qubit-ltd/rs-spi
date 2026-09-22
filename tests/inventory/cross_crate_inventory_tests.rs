// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Cross-crate linking checks for provider inventory macros.

use std::path::Path;

use crate::common::inventory_fixture;

/// Verifies that an explicit provider-crate link anchor discovers its provider.
#[test]
fn test_linked_sync_discovers_provider_from_another_crate() {
    assert_fixture_succeeds("linked_sync", "alpha\n");
}

/// Verifies that an unreferenced Cargo dependency does not enter the registry.
#[test]
fn test_unlinked_beta_is_not_discovered() {
    assert_fixture_succeeds("unlinked_beta", "alpha\n");
}

/// Verifies that duplicate providers fail during registry construction.
#[test]
fn test_duplicate_provider_reports_its_submission_source() {
    assert_fixture_succeeds("duplicate_sync", "provider-duplicate\n");
}

/// Verifies that an asynchronously discovered provider can create a service.
#[test]
fn test_linked_async_provider_creates_service() {
    assert_fixture_succeeds("linked_async", "async\n");
}

/// Verifies that declaration and submission macros work through a renamed SPI
/// dependency.
#[test]
fn test_renamed_spi_dependency_preserves_macro_hygiene() {
    assert_fixture_succeeds("renamed_spi", "alpha,beta\n");
}

/// Runs one fixture binary and checks that neither it nor Cargo pollutes
/// fixture sources.
///
/// # Parameters
///
/// * `binary` - Consumer binary encoding one independent linking graph.
/// * `expected_stdout` - Exact output produced after the binary validates its
///   contract.
fn assert_fixture_succeeds(binary: &str, expected_stdout: &str) {
    let output = inventory_fixture::run(binary);

    assert!(
        output.status.success(),
        "{binary} fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(expected_stdout, String::from_utf8_lossy(&output.stdout));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/provider_inventory/target")
            .exists(),
        "fixture sources must not retain a Cargo target directory"
    );
}
