// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::AsyncServiceSpec;
use qubit_spi::ServiceSpec;

/// Asynchronous service family with a shared unsized configuration.
struct AsyncByteCountSpec;

impl ServiceSpec for AsyncByteCountSpec {
    type Config = [u8];
    type Error = std::io::Error;
}

impl AsyncServiceSpec for AsyncByteCountSpec {
    type Output = usize;
}

/// Verifies that asynchronous output and configuration constraints compose.
#[test]
fn test_async_service_spec_selects_send_output() {
    fn assert_spec<S: AsyncServiceSpec<Config = [u8], Output = usize>>() {}
    fn assert_send_static<T: Send + 'static>() {}

    assert_spec::<AsyncByteCountSpec>();
    assert_send_static::<<AsyncByteCountSpec as AsyncServiceSpec>::Output>();
}
