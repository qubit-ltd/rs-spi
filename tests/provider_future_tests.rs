// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use futures::executor::block_on;
use qubit_spi::ProviderFuture;

/// Verifies that provider futures are sendable boxed futures.
#[test]
fn test_provider_future_is_send_and_awaitable() {
    fn assert_send<T: Send>(_: &T) {}

    let future: ProviderFuture<'static, usize> = Box::pin(async { 7 });
    assert_send(&future);
    assert_eq!(7, block_on(future));
}
