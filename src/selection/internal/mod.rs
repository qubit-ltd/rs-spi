// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private invariant-preserving storage for provider selections.

mod provider_selection;
mod provider_selection_repr;

pub(super) use provider_selection::parse_chain_selectors;
pub(crate) use provider_selection_repr::ProviderSelectionRepr;
