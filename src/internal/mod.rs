// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private registry and selection storage used by the public SPI contract.

mod builder_entry;
mod provider_selection_repr;
mod registry_entry;
mod registry_inner;

pub(crate) use builder_entry::BuilderEntry;
pub(crate) use provider_selection_repr::ProviderSelectionRepr;
pub(crate) use registry_entry::RegistryEntry;
pub(crate) use registry_inner::RegistryInner;
