// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ServiceSpec;
use qubit_spi::SyncServiceSpec;

/// Service family proving that configuration may be dynamically sized.
struct UnsizedConfigSpec;

impl ServiceSpec for UnsizedConfigSpec {
    type Config = str;
    type Error = std::io::Error;
}

/// Service family proving that synchronous output is a separate capability.
struct StringOutputSpec;

impl ServiceSpec for StringOutputSpec {
    type Config = str;
    type Error = std::io::Error;
}

impl SyncServiceSpec for StringOutputSpec {
    type Output = String;
}

/// Accepts only a base service specification with unsized configuration.
fn assert_unsized_config_spec<S: ServiceSpec<Config = str>>() {}

/// Accepts only a synchronous service specification with string output.
fn assert_string_output_spec<S: SyncServiceSpec<Output = String>>() {}

/// Verifies unsized configuration and sized output associated types.
#[test]
fn test_service_spec_accepts_unsized_configuration_and_sized_output() {
    assert_unsized_config_spec::<UnsizedConfigSpec>();
    assert_unsized_config_spec::<StringOutputSpec>();
    assert_string_output_spec::<StringOutputSpec>();
}
