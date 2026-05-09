/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Selection policy for provider resolution.

use crate::{
    ProviderName,
    ProviderRegistryError,
};

/// Provider candidates used by registry selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderSelection {
    /// Select providers automatically by registry priority.
    Auto,
    /// Try a primary provider followed by explicit fallback providers.
    Named {
        /// Primary provider candidate.
        primary: ProviderName,
        /// Ordered fallback provider candidates.
        fallbacks: Vec<ProviderName>,
    },
}

impl ProviderSelection {
    /// Creates an automatic provider selection.
    ///
    /// # Returns
    /// Automatic provider selection.
    pub fn auto() -> Self {
        Self::Auto
    }

    /// Creates a named provider selection without fallbacks.
    ///
    /// # Parameters
    /// - `primary`: Primary provider name.
    ///
    /// # Returns
    /// Named provider selection.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when `primary` is not a valid provider
    /// name.
    pub fn named(primary: &str) -> Result<Self, ProviderRegistryError> {
        Ok(Self::Named {
            primary: ProviderName::new(primary)?,
            fallbacks: Vec::new(),
        })
    }

    /// Creates a named provider selection from borrowed fallback names.
    ///
    /// # Parameters
    /// - `primary`: Primary provider name.
    /// - `fallbacks`: Ordered fallback provider names.
    ///
    /// # Returns
    /// Named provider selection.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when `primary` or any fallback is not a
    /// valid provider name.
    pub fn from_names(primary: &str, fallbacks: &[&str]) -> Result<Self, ProviderRegistryError> {
        Ok(Self::Named {
            primary: ProviderName::new(primary)?,
            fallbacks: normalize_borrowed_names(fallbacks)?,
        })
    }

    /// Creates a named provider selection from owned fallback names.
    ///
    /// # Parameters
    /// - `primary`: Primary provider name.
    /// - `fallbacks`: Ordered fallback provider names.
    ///
    /// # Returns
    /// Named provider selection.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when `primary` or any fallback is not a
    /// valid provider name.
    pub fn new(primary: &str, fallbacks: &[String]) -> Result<Self, ProviderRegistryError> {
        Ok(Self::Named {
            primary: ProviderName::new(primary)?,
            fallbacks: normalize_owned_names(fallbacks)?,
        })
    }

    /// Tells whether this selection requests automatic selection.
    ///
    /// # Returns
    /// `true` when this selection is [`ProviderSelection::Auto`].
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Gets the primary provider for named selections.
    ///
    /// # Returns
    /// `Some` primary provider for named selections, or `None` for automatic
    /// selection.
    pub fn primary(&self) -> Option<&ProviderName> {
        match self {
            Self::Auto => None,
            Self::Named { primary, .. } => Some(primary),
        }
    }

    /// Gets ordered fallback provider names.
    ///
    /// # Returns
    /// Fallback provider names, or an empty slice for automatic selection.
    pub fn fallbacks(&self) -> &[ProviderName] {
        match self {
            Self::Auto => &[],
            Self::Named { fallbacks, .. } => fallbacks,
        }
    }

    /// Builds the candidate provider names to try.
    ///
    /// # Parameters
    /// - `auto_candidates`: Registry-provided automatic candidate order.
    ///
    /// # Returns
    /// Automatic candidates when this selection is automatic, otherwise the
    /// primary provider followed by explicit fallbacks.
    pub(crate) fn candidates(&self, auto_candidates: Vec<ProviderName>) -> Vec<ProviderName> {
        match self {
            Self::Auto => auto_candidates,
            Self::Named { primary, fallbacks } => {
                let mut candidates = Vec::with_capacity(fallbacks.len() + 1);
                candidates.push(primary.clone());
                candidates.extend(fallbacks.iter().cloned());
                candidates
            }
        }
    }
}

impl Default for ProviderSelection {
    /// Creates an automatic provider selection.
    fn default() -> Self {
        Self::Auto
    }
}

/// Normalizes owned provider names.
///
/// # Parameters
/// - `names`: Raw provider names.
///
/// # Returns
/// Normalized provider names.
///
/// # Errors
/// Returns [`ProviderRegistryError`] when any provider name is invalid.
fn normalize_owned_names(names: &[String]) -> Result<Vec<ProviderName>, ProviderRegistryError> {
    names
        .iter()
        .map(String::as_str)
        .map(ProviderName::new)
        .collect()
}

/// Normalizes borrowed provider names.
///
/// # Parameters
/// - `names`: Raw provider names.
///
/// # Returns
/// Normalized provider names.
///
/// # Errors
/// Returns [`ProviderRegistryError`] when any provider name is invalid.
fn normalize_borrowed_names(names: &[&str]) -> Result<Vec<ProviderName>, ProviderRegistryError> {
    names.iter().copied().map(ProviderName::new).collect()
}
