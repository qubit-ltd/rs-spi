// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Duplicate provider implementation and inventory submission.

use service_contract::FixtureSpec;
use service_contract::sync_providers::Entry;
use spi::submit_sync_provider;

mod internal;

use internal::DuplicateProvider;

submit_sync_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = DuplicateProvider;
}
