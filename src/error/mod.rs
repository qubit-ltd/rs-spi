// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors and diagnostics produced by provider validation, registration, and
//! resolution.
//!
//! # Removed parallel error classifications
//!
//! Validation and registration errors are matched directly. The former
//! parallel Kind types are intentionally unavailable:
//!
//! ```compile_fail
//! use qubit_spi::error::ProviderDescriptorErrorKind;
//! # fn main() {}
//! ```
//! ```compile_fail
//! use qubit_spi::error::ProviderIdErrorKind;
//! # fn main() {}
//! ```
//! ```compile_fail
//! use qubit_spi::error::ProviderSelectionErrorKind;
//! # fn main() {}
//! ```
//! ```compile_fail
//! use qubit_spi::error::ProviderSelectorErrorKind;
//! # fn main() {}
//! ```
//! ```compile_fail
//! use qubit_spi::error::RegistrationErrorKind;
//! # fn main() {}
//! ```

mod attempt_failure;
mod provider_descriptor_error;
mod provider_error;
mod provider_error_kind;
mod provider_id_error;
mod provider_selection_error;
mod provider_selector_error;
mod registration_error;
mod resolution_error;

pub use attempt_failure::AttemptFailure;
pub use provider_descriptor_error::ProviderDescriptorError;
pub use provider_error::ProviderError;
pub use provider_error_kind::ProviderErrorKind;
pub use provider_id_error::ProviderIdError;
pub use provider_selection_error::ProviderSelectionError;
pub use provider_selector_error::ProviderSelectorError;
pub use registration_error::RegistrationError;
pub use resolution_error::ResolutionError;
