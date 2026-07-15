// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable provider registration metadata.

use std::collections::HashSet;

use crate::{ProviderDescriptorError, ProviderId, ProviderSelector};

/// Immutable metadata that identifies and ranks a registered provider.
///
/// Construct a descriptor while assembling a [`crate::ProviderRegistry`]: its
/// ID and aliases control explicit lookup, while its priority controls
/// automatic selection order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDescriptor {
    /// Canonical, globally unique identifier of the provider.
    id: ProviderId,
    /// Normalized alternative selectors that resolve to `id`.
    aliases: Box<[ProviderSelector]>,
    /// Descending sort key used during automatic provider selection.
    priority: i32,
}

impl ProviderDescriptor {
    /// Creates metadata for a canonical provider ID.
    ///
    /// `id` is the provider's stable identity. The returned descriptor has no
    /// aliases and priority zero, and can be refined with the builder-style
    /// methods before registration.
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
    /// Each item in `aliases` is trimmed, lowercased, and validated. On
    /// success, returns this descriptor with the resulting aliases.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderDescriptorError`] when an alias is invalid, duplicates
    /// another alias, or duplicates the canonical provider ID.
    pub fn with_aliases<I, T>(mut self, aliases: I) -> Result<Self, ProviderDescriptorError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let canonical_selector = ProviderSelector::parse(self.id.as_str())
            .expect("canonical provider IDs are valid selectors");
        let mut seen = HashSet::new();
        let mut normalized = Vec::new();
        for (alias_index, alias) in aliases.into_iter().enumerate() {
            let input = alias.as_ref();
            let alias = ProviderSelector::parse(input).map_err(|source| {
                ProviderDescriptorError::invalid_alias(alias_index, input, source)
            })?;
            if alias == canonical_selector {
                return Err(ProviderDescriptorError::alias_matches_id(alias.as_str()));
            }
            if !seen.insert(alias.clone()) {
                return Err(ProviderDescriptorError::duplicate_alias(alias.as_str()));
            }
            normalized.push(alias);
        }
        self.aliases = normalized.into_boxed_slice();
        Ok(self)
    }

    /// Sets the priority used by automatic selection.
    ///
    /// `priority` is ordered descending; providers with equal priorities are
    /// ordered by canonical ID. Returns the updated descriptor.
    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Returns the canonical provider ID.
    #[must_use]
    pub fn id(&self) -> &ProviderId {
        &self.id
    }

    /// Returns the normalized aliases that resolve to the canonical ID.
    #[must_use]
    pub fn aliases(&self) -> &[ProviderSelector] {
        &self.aliases
    }

    /// Returns the priority used to order automatic selection candidates.
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }
}
