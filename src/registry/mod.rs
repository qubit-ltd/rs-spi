// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider registration, resolution, and creation fallback facades.

mod async_provider_registry;
mod async_resolving_service_provider;
mod internal;
mod provider_creation_termination;
mod provider_registry;
mod resolving_service_provider;

pub use async_provider_registry::AsyncProviderRegistry;
pub use async_resolving_service_provider::AsyncResolvingServiceProvider;
pub use provider_creation_termination::ProviderCreationTermination;
pub use provider_registry::ProviderRegistry;
pub use resolving_service_provider::ResolvingServiceProvider;
