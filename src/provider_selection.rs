// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Explicit provider selection inputs and fallback policy.

use crate::{ProviderSelector, RegistrationError};

/// Explicit request for the providers a resolver may try.
///
/// Use this type to choose automatic selection, one named provider, or an
/// ordered fallback chain at each service-creation call.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ProviderSelection {
    /// Uses every registered provider in deterministic automatic order.
    #[default]
    Auto,
    /// Uses exactly one normalized selector.
    Named(ProviderSelector),
    /// Tries normalized selectors in the supplied order.
    Chain(Box<[ProviderSelector]>),
}

impl ProviderSelection {
    /// Creates a one-provider selection from configuration input.
    ///
    /// `value` is normalized as a provider selector. Returns a `Named`
    /// selection containing that selector.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when `value` cannot form a valid selector.
    pub fn named(value: impl AsRef<str>) -> Result<Self, RegistrationError> {
        Ok(Self::Named(ProviderSelector::parse(value)?))
    }

    /// Creates an ordered candidate chain from configuration input.
    ///
    /// Each item in `values` is normalized as a provider selector. Returns a
    /// `Chain` preserving the resulting selector order.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when any selector is invalid or `values`
    /// produces no selectors.
    pub fn chain<I, T>(values: I) -> Result<Self, RegistrationError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let selectors = values
            .into_iter()
            .map(ProviderSelector::parse)
            .collect::<Result<Vec<_>, _>>()?;
        if selectors.is_empty() {
            return Err(RegistrationError::empty_identifier());
        }
        Ok(Self::Chain(selectors.into_boxed_slice()))
    }
}

/// Controls which provider failures permit a resolver to try a fallback.
///
/// Choose [`FallbackPolicy::OnAbsence`] for conservative fallback across
/// optional backends, or [`FallbackPolicy::OnAnyError`] for best-effort chains.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FallbackPolicy {
    /// Continues only after a missing, unsupported, or unavailable provider.
    #[default]
    OnAbsence,
    /// Continues after every provider failure.
    OnAnyError,
}
