// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Offline dependency checks for the cross-crate inventory fixture.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Verifies that every fixture registry package is already pinned by the root
/// lockfile, so nested offline Cargo runs need no additional downloads.
#[test]
fn test_fixture_registry_dependencies_are_pinned_by_root_lockfile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = root.join("tests/fixtures/provider_inventory/Cargo.lock");
    let fixture_packages = registry_packages(&fixture);
    let root_packages = registry_packages(&root.join("Cargo.lock"));
    let missing = fixture_packages.difference(&root_packages).collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "fixture registry dependencies missing from root lockfile: {missing:?}"
    );
}

/// Extracts registry-sourced `(name, version)` package pairs from a lockfile.
///
/// # Parameters
///
/// * `lockfile` - Cargo lockfile containing package records to inspect.
///
/// # Returns
///
/// All crates.io package names and versions listed by the lockfile.
///
/// # Panics
///
/// Panics when the lockfile cannot be read or contains an incomplete registry
/// package record.
fn registry_packages(lockfile: &Path) -> BTreeSet<(String, String)> {
    fs::read_to_string(lockfile)
        .expect("Cargo lockfile should be readable")
        .split("[[package]]")
        .filter_map(|record| {
            let name = record
                .lines()
                .find_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'));
            let version = record
                .lines()
                .find_map(|line| line.strip_prefix("version = \"")?.strip_suffix('"'));
            let source = record
                .lines()
                .find_map(|line| line.strip_prefix("source = \"")?.strip_suffix('"'));

            source.filter(|value| value.starts_with("registry+")).map(|_| {
                (
                    name.expect("registry package should have a name").to_owned(),
                    version.expect("registry package should have a version").to_owned(),
                )
            })
        })
        .collect()
}
