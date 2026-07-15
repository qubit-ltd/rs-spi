// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ServiceSpec;

/// Service family proving that configuration may be dynamically sized.
struct UnsizedConfigSpec;

impl ServiceSpec for UnsizedConfigSpec {
    type Config = str;
    type Output = String;
}

/// Accepts only a service specification with the expected associated types.
fn assert_unsized_config_spec<S: ServiceSpec<Config = str, Output = String>>() {
}

/// Verifies unsized configuration and sized output associated types.
#[test]
fn test_service_spec_accepts_unsized_configuration_and_sized_output() {
    assert_unsized_config_spec::<UnsizedConfigSpec>();
}
