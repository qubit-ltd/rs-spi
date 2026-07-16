// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private representations for opaque validation and registration errors.

mod provider_descriptor_error_repr;
mod provider_id_error_repr;
mod provider_selection_error_repr;
mod provider_selector_error_repr;
mod registration_error_repr;

pub(crate) use provider_descriptor_error_repr::ProviderDescriptorErrorRepr;
pub(crate) use provider_id_error_repr::ProviderIdErrorRepr;
pub(crate) use provider_selection_error_repr::ProviderSelectionErrorRepr;
pub(crate) use provider_selector_error_repr::ProviderSelectorErrorRepr;
pub(crate) use registration_error_repr::RegistrationErrorRepr;
