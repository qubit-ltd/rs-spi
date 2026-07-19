// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors and diagnostics produced by provider validation, registration,
//! selection, and service creation.
//!
//! Match validation failures directly and retain a wildcard arm for future
//! variants:
//!
//! ```rust
//! use qubit_spi::ProviderSelector;
//! use qubit_spi::error::ProviderSelectorError;
//!
//! let error = ProviderSelector::parse("bad selector")
//!     .expect_err("the selector contains whitespace");
//! match error {
//!     ProviderSelectorError::Invalid { normalized, .. } => {
//!         assert_eq!("bad selector", normalized.as_ref());
//!     }
//!     ProviderSelectorError::Empty { .. } => unreachable!(),
//!     _ => {}
//! }
//! ```
//!
//! Error values with correlated fields are produced by this crate rather than
//! assembled downstream:
//!
//! ```compile_fail
//! use qubit_spi::error::ProviderCreationError;
//! use qubit_spi::ProviderCreationTermination;
//!
//! let _ = ProviderCreationError {
//!     attempts: Box::new([]),
//!     termination: ProviderCreationTermination::Exhausted,
//! };
//! ```

mod provider_attempt_failure;
mod provider_creation_error;
mod provider_descriptor_error;
mod provider_error;
mod provider_error_kind;
mod provider_id_error;
mod provider_resolution_error;
mod provider_selection_build_error;
mod provider_selector_error;
mod registration_error;

pub use provider_attempt_failure::ProviderAttemptFailure;
pub use provider_creation_error::ProviderCreationError;
pub use provider_descriptor_error::ProviderDescriptorError;
pub use provider_error::ProviderError;
pub use provider_error_kind::ProviderErrorKind;
pub use provider_id_error::ProviderIdError;
pub use provider_resolution_error::ProviderResolutionError;
pub use provider_selection_build_error::ProviderSelectionBuildError;
pub use provider_selector_error::ProviderSelectorError;
pub use registration_error::RegistrationError;
