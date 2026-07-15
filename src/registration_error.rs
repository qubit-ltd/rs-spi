// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Conflicts raised while registering providers.

use thiserror::Error;

use crate::internal::RegistrationErrorRepr;
use crate::RegistrationErrorKind;

/// Error returned when a provider registration conflicts with registry state.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error(transparent)]
pub struct RegistrationError(
    /// Variant-specific provider registration conflict.
    RegistrationErrorRepr,
);

impl RegistrationError {
    /// Creates an error for a selector claimed by another provider.
    ///
    /// # Arguments
    ///
    /// * `selector` - Conflicting canonical ID or alias.
    /// * `existing_provider` - Canonical provider currently owning the selector.
    /// * `provider` - Canonical provider attempting the new claim.
    ///
    /// # Returns
    ///
    /// A registry-owned duplicate-selector error.
    #[inline]
    #[must_use]
    pub(crate) fn duplicate_selector(
        selector: &str,
        existing_provider: &str,
        provider: &str,
    ) -> Self {
        Self(RegistrationErrorRepr::DuplicateSelector {
            selector: selector.into(),
            existing_provider: existing_provider.into(),
            provider: provider.into(),
        })
    }

    /// Returns the registration conflict classification.
    ///
    /// # Returns
    ///
    /// [`RegistrationErrorKind::DuplicateSelector`].
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> RegistrationErrorKind {
        RegistrationErrorKind::DuplicateSelector
    }

    /// Returns the canonical ID or alias that conflicts.
    ///
    /// # Returns
    ///
    /// The conflicting normalized selector.
    #[inline(always)]
    #[must_use]
    pub fn selector(&self) -> &str {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector { selector, .. } => selector,
        }
    }

    /// Returns the canonical provider that already owns the selector.
    ///
    /// # Returns
    ///
    /// The existing provider's canonical ID.
    #[inline(always)]
    #[must_use]
    pub fn existing_provider(&self) -> &str {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector {
                existing_provider, ..
            } => existing_provider,
        }
    }

    /// Returns the canonical provider that attempted the conflicting claim.
    ///
    /// # Returns
    ///
    /// The new provider's canonical ID.
    #[inline(always)]
    #[must_use]
    pub fn provider(&self) -> &str {
        match &self.0 {
            RegistrationErrorRepr::DuplicateSelector { provider, .. } => provider,
        }
    }
}
