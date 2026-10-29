// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use provider_alpha as _;
use provider_duplicate as _;

/// Proves that duplicate discovered providers fail while building the registry.
///
/// The duplicate registration error must identify the submitted provider and
/// retain the selector conflict.
fn main() {
    let error = service_contract::sync_providers::build_registry()
        .expect_err("duplicate alpha provider should fail registry construction");

    assert_eq!("provider-duplicate", error.source_location().crate_name());
    assert!(error
        .registration_error()
        .expect("selector conflict has a registration error")
        .to_string()
        .contains("alpha"));
    println!("provider-duplicate");
}
