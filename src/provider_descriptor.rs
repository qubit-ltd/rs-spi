// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable provider registration metadata.

use std::collections::HashSet;

use crate::error::ProviderDescriptorError;
use crate::{
    ProviderId,
    ProviderSelector,
};

/// Immutable metadata that identifies and ranks a registered provider.
///
/// Construct a descriptor while assembling a [`crate::ProviderRegistry`]: its
/// ID and aliases control explicit lookup, while its priority controls
/// automatic selection order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDescriptor {
    /// Canonical identifier unique within one Registry and service family.
    id: ProviderId,
    /// Normalized alternative selectors that resolve to `id`.
    aliases: Box<[ProviderSelector]>,
    /// Descending sort key used during automatic provider selection.
    priority: i32,
}

impl ProviderDescriptor {
    /// Creates metadata for a canonical provider ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Stable canonical identity of the provider.
    ///
    /// # Returns
    ///
    /// A descriptor with no aliases and priority zero.
    #[inline]
    #[must_use]
    pub fn new(id: ProviderId) -> Self {
        Self {
            id,
            aliases: Box::new([]),
            priority: 0,
        }
    }

    /// Replaces the descriptor's aliases with normalized lookup selectors.
    ///
    /// # Arguments
    ///
    /// * `aliases` - Raw aliases trimmed, lowercased, and validated in order.
    ///
    /// # Returns
    ///
    /// This descriptor with the resulting normalized aliases.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderDescriptorError`] when an alias is invalid, duplicates
    /// another alias, or duplicates the canonical provider ID.
    pub fn with_aliases<I, T>(
        mut self,
        aliases: I,
    ) -> Result<Self, ProviderDescriptorError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut inputs = Vec::new();
        for alias in aliases {
            inputs.push(Box::<str>::from(alias.as_ref()));
        }
        self.aliases = normalize_aliases(&self.id, inputs)?;
        Ok(self)
    }

    /// Sets the priority used by automatic selection.
    ///
    /// # Arguments
    ///
    /// * `priority` - Descending automatic-selection sort key.
    ///
    /// # Returns
    ///
    /// This descriptor with the replacement priority.
    #[inline(always)]
    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Returns the canonical provider ID.
    ///
    /// # Returns
    ///
    /// The descriptor's stable provider identity.
    #[inline(always)]
    #[must_use]
    pub fn id(&self) -> &ProviderId {
        &self.id
    }

    /// Returns the normalized aliases that resolve to the canonical ID.
    ///
    /// # Returns
    ///
    /// The immutable alias slice in descriptor order.
    #[inline(always)]
    #[must_use]
    pub fn aliases(&self) -> &[ProviderSelector] {
        &self.aliases
    }

    /// Returns the priority used to order automatic selection candidates.
    ///
    /// # Returns
    ///
    /// The descending automatic-selection sort key.
    #[inline(always)]
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }
}

/// Normalizes and validates owned alias inputs for one canonical provider ID.
///
/// Keeping validation outside the generic public method avoids duplicating the
/// complete validation state machine for every caller iterator type.
///
/// # Arguments
///
/// * `id` - Canonical provider ID that aliases must not duplicate.
/// * `inputs` - Owned raw aliases in caller-supplied order.
///
/// # Returns
///
/// The validated normalized aliases in input order.
///
/// # Errors
///
/// Returns [`ProviderDescriptorError`] when an alias is invalid, duplicates the
/// canonical provider ID, or duplicates an earlier normalized alias.
fn normalize_aliases(
    id: &ProviderId,
    inputs: Vec<Box<str>>,
) -> Result<Box<[ProviderSelector]>, ProviderDescriptorError> {
    let canonical_selector = ProviderSelector::from(id);
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(inputs.len());
    for (alias_index, input) in inputs.into_iter().enumerate() {
        let alias = match ProviderSelector::parse(&input) {
            Ok(alias) => alias,
            Err(source) => {
                return Err(ProviderDescriptorError::invalid_alias(
                    alias_index,
                    source,
                ));
            }
        };
        if alias == canonical_selector {
            return Err(ProviderDescriptorError::alias_matches_id(
                alias.as_str(),
            ));
        }
        if !seen.insert(alias.clone()) {
            return Err(ProviderDescriptorError::duplicate_alias(
                alias.as_str(),
            ));
        }
        normalized.push(alias);
    }
    Ok(normalized.into_boxed_slice())
}
