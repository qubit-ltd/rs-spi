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
    /// Tries providers in deterministic automatic order until resolution stops.
    #[default]
    Auto,
    /// Uses exactly one normalized selector.
    Named(
        /// Selector that must resolve to the single provider used.
        ProviderSelector,
    ),
    /// Tries normalized selectors in the supplied order.
    Chain(
        /// Ordered selectors tried until a provider succeeds or fallback stops.
        Box<[ProviderSelector]>,
    ),
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

/// Controls which provider creation failures permit trying another candidate.
///
/// This policy applies to automatic and chained selection after a provider
/// factory returns an error. Named selection always uses exactly one provider
/// and never falls back.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FallbackPolicy {
    /// Continues only after an unsupported or unavailable provider.
    #[default]
    OnAbsence,
    /// Continues after every provider creation failure.
    OnAnyError,
}
