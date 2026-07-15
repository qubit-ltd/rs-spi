// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage and representation types used by the public SPI contract.

mod attempt_failure_repr;
mod builder_entry;
mod provider_descriptor_error_repr;
mod provider_id_error_repr;
mod provider_selection_error_repr;
mod provider_selection_repr;
mod provider_selector_error_repr;
mod registration_error_repr;
mod registry_entry;
mod registry_inner;
mod resolution_error_repr;

pub(crate) use attempt_failure_repr::AttemptFailureRepr;
pub(crate) use builder_entry::BuilderEntry;
pub(crate) use provider_descriptor_error_repr::ProviderDescriptorErrorRepr;
pub(crate) use provider_id_error_repr::ProviderIdErrorRepr;
pub(crate) use provider_selection_error_repr::ProviderSelectionErrorRepr;
pub(crate) use provider_selection_repr::ProviderSelectionRepr;
pub(crate) use provider_selector_error_repr::ProviderSelectorErrorRepr;
pub(crate) use registration_error_repr::RegistrationErrorRepr;
pub(crate) use registry_entry::RegistryEntry;
pub(crate) use registry_inner::RegistryInner;
pub(crate) use resolution_error_repr::ResolutionErrorRepr;
