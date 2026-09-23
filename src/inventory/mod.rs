// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Types supporting optional link-time provider discovery.

mod async_provider_inventory_entry;
mod build_async_registry;
mod build_sync_registry;
mod macros;
mod provider_registration_source;
mod sync_provider_inventory_entry;

pub use async_provider_inventory_entry::AsyncProviderInventoryEntry;
pub use build_async_registry::build_async_registry;
pub use build_async_registry::build_async_registry_with;
pub use build_sync_registry::build_sync_registry;
pub use build_sync_registry::build_sync_registry_with;
pub use provider_registration_source::ProviderRegistrationSource;
pub use sync_provider_inventory_entry::SyncProviderInventoryEntry;
