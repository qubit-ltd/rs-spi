// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Selection policy for provider resolution.

use std::collections::HashSet;

use crate::{ProviderName, ProviderRegistryError};

/// Provider candidates used by registry selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderSelection {
    /// Select providers automatically by registry priority.
    Auto,
    /// Try a primary provider followed by explicit fallback providers.
    ///
    /// Constructor methods reject repeated normalized candidate names.
    /// Registries also validate manually constructed named selections
    /// before execution.
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
    #[inline]
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
    #[inline]
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
    /// Returns [`ProviderRegistryError`] when `primary` or any fallback is not
    /// a valid provider name, or when candidate names are duplicated.
    #[inline]
    pub fn from_names(primary: &str, fallbacks: &[&str]) -> Result<Self, ProviderRegistryError> {
        let primary = ProviderName::new(primary)?;
        let fallbacks = normalize_borrowed_names(fallbacks)?;
        validate_unique_candidate_names(&primary, &fallbacks)?;
        Ok(Self::Named { primary, fallbacks })
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
    /// Returns [`ProviderRegistryError`] when `primary` or any fallback is not
    /// a valid provider name, or when candidate names are duplicated.
    #[inline]
    pub fn from_owned_names(
        primary: &str,
        fallbacks: &[String],
    ) -> Result<Self, ProviderRegistryError> {
        let primary = ProviderName::new(primary)?;
        let fallbacks = normalize_owned_names(fallbacks)?;
        validate_unique_candidate_names(&primary, &fallbacks)?;
        Ok(Self::Named { primary, fallbacks })
    }

    /// Tells whether this selection requests automatic selection.
    ///
    /// # Returns
    /// `true` when this selection is [`ProviderSelection::Auto`].
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Gets the primary provider for named selections.
    ///
    /// # Returns
    /// `Some` primary provider for named selections, or `None` for automatic
    /// selection.
    #[inline]
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
    #[inline]
    pub fn fallbacks(&self) -> &[ProviderName] {
        match self {
            Self::Auto => &[],
            Self::Named { fallbacks, .. } => fallbacks,
        }
    }

    /// Validates that named selections do not repeat provider names.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError::DuplicateProviderCandidate`] when the
    /// primary provider or fallback list repeats a normalized name.
    pub(crate) fn validate_unique_names(&self) -> Result<(), ProviderRegistryError> {
        match self {
            Self::Auto => Ok(()),
            Self::Named { primary, fallbacks } => {
                validate_unique_candidate_names(primary, fallbacks)
            }
        }
    }
}

impl Default for ProviderSelection {
    /// Creates an automatic provider selection.
    #[inline]
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

/// Validates that candidate names are unique within one named selection.
///
/// # Parameters
/// - `primary`: Primary provider name.
/// - `fallbacks`: Ordered fallback provider names.
///
/// # Errors
/// Returns [`ProviderRegistryError::DuplicateProviderCandidate`] when a
/// fallback duplicates the primary provider or an earlier fallback.
fn validate_unique_candidate_names(
    primary: &ProviderName,
    fallbacks: &[ProviderName],
) -> Result<(), ProviderRegistryError> {
    let mut names = HashSet::with_capacity(fallbacks.len() + 1);
    names.insert(primary.clone());
    for fallback in fallbacks {
        if !names.insert(fallback.clone()) {
            return Err(ProviderRegistryError::DuplicateProviderCandidate {
                name: fallback.clone(),
            });
        }
    }
    Ok(())
}
