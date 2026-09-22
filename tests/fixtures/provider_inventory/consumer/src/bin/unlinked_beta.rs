// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use provider_alpha as _;

/// Proves that Cargo metadata alone does not link beta into the binary.
fn main() {
    let registry =
        service_contract::sync_providers::build_registry().expect("alpha should register");
    let ids = registry.provider_ids();

    assert_eq!(
        1,
        ids.len(),
        "only explicitly linked providers should register"
    );
    assert_eq!("alpha", ids[0].as_str());
    println!("alpha");
}
