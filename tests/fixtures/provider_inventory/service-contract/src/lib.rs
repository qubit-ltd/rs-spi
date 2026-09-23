// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![deny(missing_docs)]

//! Service-family contract used by the cross-crate provider fixture.

/// Service family re-exported for provider crates and consumer binaries.
pub use service_model::FixtureSpec;

spi::declare_sync_provider_inventory! {
    pub mod sync_providers {
        spec = service_model::FixtureSpec;
    }
}

spi::declare_async_provider_inventory! {
    pub mod async_providers {
        spec = service_model::FixtureSpec;
    }
}
