// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Types supporting optional link-time provider discovery.

mod build_sync_registry;
mod macros;
mod provider_registration_source;
mod sync_provider_inventory_entry;

pub use build_sync_registry::build_sync_registry;
pub use provider_registration_source::ProviderRegistrationSource;
pub use sync_provider_inventory_entry::SyncProviderInventoryEntry;
