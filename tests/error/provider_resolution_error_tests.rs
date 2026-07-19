// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt::{
    self,
    Write,
};

use qubit_spi::{
    ProviderRegistry,
    ProviderSelection,
};

use crate::common::failing_writer::FailingWriter;
use crate::common::string_spec::StringSpec;

/// Verifies that no-candidate diagnostics propagate formatter failures.
#[test]
fn test_no_candidates_display_propagates_formatter_failures() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let selection =
        ProviderSelection::chain_allowing_missing(["first", "second"])
            .expect("test selectors should be valid");
    let error = registry
        .resolve_selected(&selection)
        .expect_err("unmatched chain should fail resolution");

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
        .resolve_selected(
            &ProviderSelection::named("first")
                .expect("test selector should parse"),
        )
        .expect_err("unknown named selector should fail");
    let plural = registry
        .resolve_selected(
            &ProviderSelection::chain(["first", "second"])
                .expect("test selectors should parse"),
        )
        .expect_err("unknown strict chain should fail");

    assert_eq!("unknown provider selector; first", singular.to_string());
    assert_eq!(
        "unknown provider selectors; first; second",
        plural.to_string(),
    );

    for remaining_successes in [0, 1, 2] {
        let mut writer = FailingWriter::new(remaining_successes);
        assert_eq!(Err(fmt::Error), write!(&mut writer, "{plural}"));
    }
}
