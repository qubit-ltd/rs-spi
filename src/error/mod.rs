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
//! use qubit_spi::error::ResolutionError;
//! use qubit_spi::ResolutionTermination;
//!
//! let _ = ResolutionError::NoProviderSucceeded {
//!     attempts: Box::new([]),
//!     termination: ResolutionTermination::Exhausted,
//! };
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
