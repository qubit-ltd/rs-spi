// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderId;

#[test]
fn provider_id_requires_canonical_values_but_accepts_uri_style_values() {
    assert!(ProviderId::new("git+ssh").is_ok());
    assert!(ProviderId::new("vendor.v2").is_ok());
    assert!(ProviderId::new("File").is_err());
    assert!(ProviderId::new(" file").is_err());
}
