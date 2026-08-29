// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt;
use std::fmt::Write;

use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;

use crate::common::failing_writer::FailingWriter;
use crate::common::string_spec::StringSpec;

/// Verifies that no-candidate diagnostics propagate formatter failures.
#[test]
fn test_no_candidates_display_propagates_formatter_failures() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let selection =
        ProviderSelection::chain_allowing_missing(["first", "second"]).expect("test selectors should be valid");
    let error = registry
        .resolve_selected(&selection)
        .expect_err("unmatched chain should fail resolution");

    assert!(error.is_no_candidates());
    assert!(!error.is_unknown_providers());
    assert!(!error.is_empty_registry());
    assert_eq!(
        ["first", "second"],
        error
            .selectors()
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .as_slice(),
    );

    for remaining_successes in [0, 1, 2] {
        let mut writer = FailingWriter::new(remaining_successes);
        assert_eq!(Err(fmt::Error), write!(&mut writer, "{error}"));
    }
}

/// Verifies singular and plural unknown-provider diagnostics.
#[test]
fn test_unknown_providers_display_preserves_selector_order() {
    let registry = ProviderRegistry::<StringSpec>::default();

    let singular = registry
        .resolve_selected(&ProviderSelection::named("first").expect("test selector should parse"))
        .expect_err("unknown named selector should fail");
    let plural = registry
        .resolve_selected(&ProviderSelection::chain(["first", "second"]).expect("test selectors should parse"))
        .expect_err("unknown strict chain should fail");

    assert_eq!("unknown provider selector; first", singular.to_string());
    assert!(singular.is_unknown_providers());
    assert!(!singular.is_no_candidates());
    assert!(!singular.is_empty_registry());
    assert_eq!("first", singular.selectors().unwrap()[0].as_str());
    assert_eq!("unknown provider selectors; first; second", plural.to_string(),);

    for remaining_successes in [0, 1, 2] {
        let mut writer = FailingWriter::new(remaining_successes);
        assert_eq!(Err(fmt::Error), write!(&mut writer, "{plural}"));
    }
}

/// Verifies empty Registry diagnostics expose no selector collection.
#[test]
fn test_empty_registry_error_has_no_selectors() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let error = registry
        .resolve()
        .expect_err("automatic resolution from an empty Registry should fail");

    assert!(error.is_empty_registry());
    assert!(!error.is_unknown_providers());
    assert!(!error.is_no_candidates());
    assert!(error.selectors().is_none());
}
