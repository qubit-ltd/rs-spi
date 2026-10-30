// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous provider implementation and inventory submission.

use service_contract::FixtureSpec;
use service_contract::async_providers::Entry;
use spi::submit_async_provider;

mod async_provider;

use self::async_provider::AsyncProvider;

submit_async_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = AsyncProvider;
}
