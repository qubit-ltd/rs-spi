// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderRegistrationSource;

/// Verifies that a registration source exposes its complete declaration
/// location.
#[test]
fn test_registration_source_exposes_declaration_location() {
    let source = ProviderRegistrationSource::new("provider-a", "backend", "src/lib.rs", 17);

    assert_eq!("provider-a", source.crate_name());
    assert_eq!("backend", source.module_path());
    assert_eq!("src/lib.rs", source.file());
    assert_eq!(17, source.line());
}

/// Verifies that registration sources have a stable declaration-location order.
#[test]
fn test_registration_source_sorts_by_its_complete_declaration_location() {
    let earliest = ProviderRegistrationSource::new("provider-a", "backend", "src/lib.rs", 16);
    let later_line = ProviderRegistrationSource::new("provider-a", "backend", "src/lib.rs", 17);
    let later_file = ProviderRegistrationSource::new("provider-a", "backend", "src/main.rs", 1);
    let later_module = ProviderRegistrationSource::new("provider-a", "frontend", "src/lib.rs", 1);
    let later_crate = ProviderRegistrationSource::new("provider-b", "backend", "src/lib.rs", 1);

    assert!(earliest < later_line);
    assert!(later_line < later_file);
    assert!(later_file < later_module);
    assert!(later_module < later_crate);
}

/// Verifies that the source display includes every declaration-location
/// segment.
#[test]
fn test_registration_source_display_includes_the_complete_declaration_location() {
    let source = ProviderRegistrationSource::new("provider-a", "backend", "src/lib.rs", 17);
    let display = source.to_string();

    assert!(display.contains("provider-a"));
    assert!(display.contains("backend"));
    assert!(display.contains("src/lib.rs"));
    assert!(display.contains("17"));
}
