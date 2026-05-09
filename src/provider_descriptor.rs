/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Provider metadata captured by registries.

use crate::{
    ProviderName,
    ProviderRegistryError,
};

/// Stable provider metadata used for registration and selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDescriptor {
    /// Canonical provider id.
    id: ProviderName,
    /// Additional accepted provider aliases.
    aliases: Vec<ProviderName>,
    /// Priority used by automatic selection.
    priority: i32,
}

impl ProviderDescriptor {
    /// Creates a descriptor with no aliases and zero priority.
    ///
    /// # Parameters
    /// - `id`: Provider id.
    ///
    /// # Returns
    /// Provider descriptor.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when `id` is not a valid provider name.
    pub fn new(id: &str) -> Result<Self, ProviderRegistryError> {
        Ok(Self {
            id: ProviderName::new(id)?,
            aliases: Vec::new(),
            priority: 0,
        })
    }

    /// Sets provider aliases.
    ///
    /// # Parameters
    /// - `aliases`: Provider aliases.
    ///
    /// # Returns
    /// Updated provider descriptor.
    ///
    /// # Errors
    /// Returns [`ProviderRegistryError`] when any alias is not a valid provider
    /// name.
    pub fn with_aliases(mut self, aliases: &[&str]) -> Result<Self, ProviderRegistryError> {
        self.aliases = aliases
            .iter()
            .map(|alias| ProviderName::new(alias))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self)
    }

    /// Sets provider priority.
    ///
    /// # Parameters
    /// - `priority`: Provider priority. Larger values are preferred.
    ///
    /// # Returns
    /// Updated provider descriptor.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Gets the canonical provider id.
    ///
    /// # Returns
    /// Provider id.
    pub fn id(&self) -> &ProviderName {
        &self.id
    }

    /// Gets provider aliases.
    ///
    /// # Returns
    /// Provider aliases.
    pub fn aliases(&self) -> &[ProviderName] {
        &self.aliases
    }

    /// Gets provider aliases as string slices.
    ///
    /// # Returns
    /// Provider aliases as normalized string slices.
    pub fn aliases_as_str(&self) -> Vec<&str> {
        self.aliases.iter().map(ProviderName::as_str).collect()
    }

    /// Gets provider priority.
    ///
    /// # Returns
    /// Provider priority.
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Iterates over the provider id followed by aliases.
    ///
    /// # Returns
    /// Iterator over all normalized names exposed by this descriptor.
    pub(crate) fn names(&self) -> impl Iterator<Item = &ProviderName> {
        std::iter::once(&self.id).chain(self.aliases.iter())
    }
}
