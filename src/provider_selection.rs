// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Explicit provider selection inputs and fallback policy.

use crate::{ProviderSelector, RegistrationError};

/// Requested provider candidates.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ProviderSelection {
    /// Use every registered provider in deterministic automatic order.
    #[default]
    Auto,
    /// Use one named provider.
    Named(ProviderSelector),
    /// Try named providers in the supplied order.
    Chain(Box<[ProviderSelector]>),
}

impl ProviderSelection {
    /// Creates a named selection from configuration input.
    ///
    /// # Errors
    ///
    /// Returns RegistrationError when the selector is invalid.
    pub fn named(value: impl AsRef<str>) -> Result<Self, RegistrationError> {
        Ok(Self::Named(ProviderSelector::parse(value)?))
    }

    /// Creates an ordered candidate chain from configuration input.
    ///
    /// # Errors
    ///
    /// Returns RegistrationError when any selector is invalid or no selectors
    /// are supplied.
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

/// Controls which failures permit the resolver to continue to a fallback.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FallbackPolicy {
    /// Continue only after a missing, unsupported, or unavailable provider.
    #[default]
    OnAbsence,
    /// Continue after every provider failure.
    OnAnyError,
}
