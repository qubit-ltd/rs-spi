// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable provider registration metadata.

use std::collections::HashSet;

use crate::ProviderId;
use crate::ProviderSelector;
use crate::error::ProviderDescriptorError;

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
    /// # Parameters
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

    /// Replaces the descriptor's aliases with normalized lookup selectors.
    ///
    /// # Type Parameters
    ///
    /// * `I` - Iterator-like source of alias inputs.
    /// * `T` - Individual alias input convertible to a string reference.
    ///
    /// # Parameters
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

    /// Sets the priority used by automatic selection.
    ///
    /// # Parameters
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

    /// Reports whether static descriptor literals satisfy descriptor
    /// invariants.
    ///
    /// This hidden macro-support API accepts only already canonical aliases.
    /// [`crate::provider_descriptor!`] uses it to reject invalid static
    /// metadata during compilation.
    ///
    /// # Parameters
    ///
    /// * `id` - Canonical provider ID literal.
    /// * `aliases` - Canonical, distinct alias literals.
    ///
    /// # Returns
    ///
    /// `true` when every literal is canonical, no alias matches `id`, and no
    /// aliases duplicate one another.
    #[doc(hidden)]
    #[must_use]
    pub const fn __are_valid_static_literals(
        id: &str,
        aliases: &[&str],
    ) -> bool {
        if !ProviderId::is_canonical_token(id) {
            return false;
        }
        let mut alias_index = 0;
        while alias_index < aliases.len() {
            let alias = aliases[alias_index];
            if !ProviderId::is_canonical_token(alias)
                || static_tokens_equal(alias, id)
            {
                return false;
            }
            let mut previous_index = 0;
            while previous_index < alias_index {
                if static_tokens_equal(alias, aliases[previous_index]) {
                    return false;
                }
                previous_index += 1;
            }
            alias_index += 1;
        }
        true
    }

    /// Creates a descriptor from literals validated by
    /// [`crate::provider_descriptor!`].
    ///
    /// # Parameters
    ///
    /// * `id` - Compile-time-validated canonical provider ID.
    /// * `aliases` - Compile-time-validated canonical aliases.
    /// * `priority` - Automatic-selection priority.
    ///
    /// # Returns
    ///
    /// The descriptor represented by the validated static metadata.
    #[doc(hidden)]
    #[must_use]
    pub fn __from_static_literals(
        id: &str,
        aliases: &[&str],
        priority: i32,
    ) -> Self {
        Self::new(ProviderId::new(id).expect(
            "provider_descriptor! validates static literals at compile time",
        ))
        .with_aliases(aliases.iter().copied())
        .expect(
            "provider_descriptor! validates static literals at compile time",
        )
        .with_priority(priority)
    }
}

/// Reports whether two static token slices have identical bytes.
///
/// # Parameters
///
/// * `left` - First token slice.
/// * `right` - Second token slice.
///
/// # Returns
///
/// `true` when the slices have equal lengths and bytes; otherwise, `false`.
const fn static_tokens_equal(left: &str, right: &str) -> bool {
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    if left_bytes.len() != right_bytes.len() {
        return false;
    }
    let mut index = 0;
    while index < left_bytes.len() {
        if left_bytes[index] != right_bytes[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// Normalizes and validates owned alias inputs for one canonical provider ID.
///
/// Keeping validation outside the generic public method avoids duplicating the
/// complete validation state machine for every caller iterator type.
///
/// # Parameters
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
    let mut seen = HashSet::with_capacity(inputs.len());
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

/// Creates a [`ProviderDescriptor`] from compile-time-validated static
/// metadata.
///
/// The ID and every alias must be canonical lowercase ASCII provider tokens.
/// Aliases must also be distinct and different from the ID. The macro rejects
/// invalid static metadata during compilation and returns a descriptor with an
/// optional automatic-selection priority.
///
/// ```
/// use qubit_spi::provider_descriptor;
///
/// let descriptor = provider_descriptor!(
///     "file",
///     aliases: ["file-command"],
///     priority: 10,
/// );
///
/// assert_eq!("file", descriptor.id().as_str());
/// assert_eq!(10, descriptor.priority());
/// ```
///
/// ```compile_fail
/// use qubit_spi::provider_descriptor;
///
/// let _ = provider_descriptor!("File");
/// ```
#[macro_export]
macro_rules! provider_descriptor {
    (
        $id:literal
        $(, aliases: [$($alias:literal),* $(,)?])?
        $(, priority: $priority:expr)?
        $(,)?
    ) => {{
        const _: () = assert!(
            $crate::ProviderDescriptor::__are_valid_static_literals(
                $id,
                &[$($($alias),*)?],
            ),
            "provider_descriptor! requires canonical, distinct ID and alias literals",
        );
        $crate::ProviderDescriptor::__from_static_literals(
            $id,
            &[$($($alias),*)?],
            0 $(+ ($priority))?,
        )
    }};
}
