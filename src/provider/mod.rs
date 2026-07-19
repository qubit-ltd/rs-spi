// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider identity and registration metadata contracts.

mod provider_descriptor;
mod provider_id;
mod provider_metadata;
mod provider_selector;

pub use provider_descriptor::ProviderDescriptor;
pub use provider_id::ProviderId;
pub use provider_metadata::ProviderMetadata;
pub use provider_selector::ProviderSelector;
