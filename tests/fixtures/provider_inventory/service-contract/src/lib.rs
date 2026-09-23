// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![deny(missing_docs)]

//! Service-family contract used by the cross-crate provider fixture.

use spi::declare_async_provider_inventory;
use spi::declare_sync_provider_inventory;

/// Service family re-exported for provider crates and consumer binaries.
pub use service_model::FixtureSpec;

declare_sync_provider_inventory! {
    pub mod sync_providers {
        spec = FixtureSpec;
    }
}

declare_async_provider_inventory! {
    pub mod async_providers {
        spec = FixtureSpec;
    }
}
