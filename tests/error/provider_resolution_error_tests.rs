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
    let selection = ProviderSelection::chain(["first", "second"])
        .expect("test selectors should be valid");
    let error = registry
        .resolve_selected(&selection)
        .expect_err("unmatched chain should fail resolution");

    for remaining_successes in [0, 1] {
        let mut writer = FailingWriter::new(remaining_successes);
        assert_eq!(Err(fmt::Error), write!(&mut writer, "{error}"));
    }
}
