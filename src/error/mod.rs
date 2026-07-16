// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors and diagnostics produced by provider validation, registration, and
//! resolution.

mod attempt_failure;
mod provider_descriptor_error;
mod provider_descriptor_error_kind;
mod provider_error;
mod provider_error_kind;
mod provider_id_error;
mod provider_id_error_kind;
mod provider_selection_error;
mod provider_selection_error_kind;
mod provider_selector_error;
mod provider_selector_error_kind;
mod registration_error;
mod registration_error_kind;
mod resolution_error;

pub use attempt_failure::AttemptFailure;
pub use provider_descriptor_error::ProviderDescriptorError;
pub use provider_descriptor_error_kind::ProviderDescriptorErrorKind;
pub use provider_error::ProviderError;
pub use provider_error_kind::ProviderErrorKind;
pub use provider_id_error::ProviderIdError;
pub use provider_id_error_kind::ProviderIdErrorKind;
pub use provider_selection_error::ProviderSelectionError;
pub use provider_selection_error_kind::ProviderSelectionErrorKind;
pub use provider_selector_error::ProviderSelectorError;
pub use provider_selector_error_kind::ProviderSelectorErrorKind;
pub use registration_error::RegistrationError;
pub use registration_error_kind::RegistrationErrorKind;
pub use resolution_error::ResolutionError;
