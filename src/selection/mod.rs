// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider selection targets and fallback policies.

mod fallback_policy;
mod internal;
mod missing_provider_policy;
mod provider_selection;
mod provider_selection_target_ref;

pub use fallback_policy::FallbackPolicy;
pub use missing_provider_policy::MissingProviderPolicy;
pub use provider_selection::ProviderSelection;
pub use provider_selection_target_ref::ProviderSelectionTargetRef;

pub(crate) use internal::ProviderSelectionRepr;
