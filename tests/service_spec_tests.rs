// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ServiceSpec;

struct UnsizedConfigSpec;

impl ServiceSpec for UnsizedConfigSpec {
    type Config = str;
    type Output = String;
}

#[test]
fn service_spec_accepts_unsized_configuration_and_sized_output() {
    fn output_type<S: ServiceSpec<Config = str, Output = String>>() {}

    output_type::<UnsizedConfigSpec>();
}
