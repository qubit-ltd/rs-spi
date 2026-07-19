// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Service specifications and synchronous or asynchronous provider contracts.

mod async_provider_definition;
mod async_service_provider;
mod async_service_spec;
mod provider_definition;
mod provider_future;
mod service_provider;
mod service_spec;
mod sync_service_spec;

pub use async_provider_definition::AsyncProviderDefinition;
pub use async_service_provider::AsyncServiceProvider;
pub use async_service_spec::AsyncServiceSpec;
pub use provider_definition::ProviderDefinition;
pub use provider_future::ProviderFuture;
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
pub use sync_service_spec::SyncServiceSpec;
