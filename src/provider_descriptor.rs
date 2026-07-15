// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable provider registration metadata.

use std::collections::HashSet;

use crate::{ProviderId, ProviderSelector, RegistrationError};

/// Stable metadata used to register and select a provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDescriptor {
    id: ProviderId,
    aliases: Box<[ProviderSelector]>,
    priority: i32,
}

impl ProviderDescriptor {
    /// Creates metadata for a canonical provider ID.
    #[must_use]
    pub fn new(id: ProviderId) -> Self {
        Self {
            id,
            aliases: Box::new([]),
            priority: 0,
        }
    }

    /// Adds normalized aliases.
    ///
    /// # Errors
    ///
    /// Returns RegistrationError when an alias is invalid, duplicates another
    /// alias, or duplicates the canonical provider ID.
    pub fn with_aliases<I, T>(mut self, aliases: I) -> Result<Self, RegistrationError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let canonical_selector = ProviderSelector::parse(self.id.as_str())
            .expect("canonical provider IDs are valid selectors");
        let mut seen = HashSet::from([canonical_selector]);
        let mut normalized = Vec::new();
        for alias in aliases {
            let alias = ProviderSelector::parse(alias)?;
            if !seen.insert(alias.clone()) {
                return Err(RegistrationError::duplicate_selector(
                    alias.as_str(),
                    self.id.as_str(),
                    self.id.as_str(),
                ));
            }
            normalized.push(alias);
        }
        self.aliases = normalized.into_boxed_slice();
        Ok(self)
    }

    /// Sets the priority used by automatic selection.
    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Gets the canonical provider ID.
    #[must_use]
    pub fn id(&self) -> &ProviderId {
        &self.id
    }

    /// Gets normalized aliases.
    #[must_use]
    pub fn aliases(&self) -> &[ProviderSelector] {
        &self.aliases
    }

    /// Gets automatic-selection priority.
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }
}
