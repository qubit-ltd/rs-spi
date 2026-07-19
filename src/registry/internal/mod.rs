// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private catalog, candidate, and fallback state used by registry facades.

mod fallback_state;
mod provider_catalog;
mod registry_entry;
mod registry_inner;
mod resolved_candidates;

pub(crate) use fallback_state::FallbackState;
pub(crate) use provider_catalog::ProviderCatalog;
pub(crate) use registry_entry::RegistryEntry;
pub(crate) use registry_inner::RegistryInner;
pub(crate) use resolved_candidates::ResolvedCandidates;
