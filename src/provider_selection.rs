/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Selection policy for default provider resolution.

/// Default and fallback provider names used by registry selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSelection {
    /// Preferred provider name or automatic-selection keyword.
    default_name: String,
    /// Ordered fallback provider names.
    fallbacks: Vec<String>,
    /// Keyword that enables automatic provider selection.
    auto_name: String,
}

impl ProviderSelection {
    /// Creates an automatic provider selection.
    ///
    /// # Returns
    /// Selection whose default provider name is `auto`.
    pub fn auto() -> Self {
        Self::default()
    }

    /// Creates a provider selection from owned fallback names.
    ///
    /// # Parameters
    /// - `default_name`: Preferred provider name, or the auto keyword.
    /// - `fallbacks`: Ordered fallback provider names.
    ///
    /// # Returns
    /// Selection with trimmed names and empty fallback entries removed.
    pub fn new(default_name: &str, fallbacks: &[String]) -> Self {
        Self {
            default_name: normalize_name(default_name),
            fallbacks: normalize_owned_names(fallbacks),
            auto_name: "auto".to_owned(),
        }
    }

    /// Creates a provider selection from borrowed fallback names.
    ///
    /// # Parameters
    /// - `default_name`: Preferred provider name, or the auto keyword.
    /// - `fallbacks`: Ordered fallback provider names.
    ///
    /// # Returns
    /// Selection with trimmed names and empty fallback entries removed.
    pub fn from_names(default_name: &str, fallbacks: &[&str]) -> Self {
        Self {
            default_name: normalize_name(default_name),
            fallbacks: normalize_borrowed_names(fallbacks),
            auto_name: "auto".to_owned(),
        }
    }

    /// Sets the keyword that enables automatic provider selection.
    ///
    /// Empty keywords are normalized back to `auto`.
    ///
    /// # Parameters
    /// - `auto_name`: Keyword used to request automatic provider selection.
    ///
    /// # Returns
    /// Updated selection.
    pub fn with_auto_name(mut self, auto_name: &str) -> Self {
        let normalized = normalize_name(auto_name);
        if normalized.is_empty() {
            self.auto_name = "auto".to_owned();
        } else {
            self.auto_name = normalized;
        }
        self
    }

    /// Gets the preferred provider name.
    ///
    /// # Returns
    /// Default provider name after trimming.
    pub fn default_name(&self) -> &str {
        &self.default_name
    }

    /// Gets ordered fallback provider names.
    ///
    /// # Returns
    /// Fallback names after trimming and empty-entry removal.
    pub fn fallbacks(&self) -> &[String] {
        &self.fallbacks
    }

    /// Gets the automatic-selection keyword.
    ///
    /// # Returns
    /// Auto keyword.
    pub fn auto_name(&self) -> &str {
        &self.auto_name
    }

    /// Tells whether the default selector requests automatic selection.
    ///
    /// Empty defaults are treated as automatic selection.
    ///
    /// # Returns
    /// `true` when the default name is empty or equals the auto keyword
    /// case-insensitively.
    pub fn is_auto_default(&self) -> bool {
        self.default_name.is_empty() || self.default_name.eq_ignore_ascii_case(&self.auto_name)
    }

    /// Builds the candidate provider names to try.
    ///
    /// # Parameters
    /// - `auto_candidates`: Registry-provided automatic candidate order.
    ///
    /// # Returns
    /// Automatic candidates when this selection is auto, otherwise the explicit
    /// default followed by configured fallbacks.
    pub(crate) fn candidates(&self, auto_candidates: Vec<String>) -> Vec<String> {
        if self.is_auto_default() {
            return auto_candidates;
        }
        let mut candidates = Vec::with_capacity(self.fallbacks.len() + 1);
        candidates.push(self.default_name.clone());
        candidates.extend(self.fallbacks.iter().cloned());
        candidates
    }
}

impl Default for ProviderSelection {
    /// Creates an automatic provider selection.
    fn default() -> Self {
        Self {
            default_name: "auto".to_owned(),
            fallbacks: Vec::new(),
            auto_name: "auto".to_owned(),
        }
    }
}

/// Trims one provider name.
///
/// # Parameters
/// - `name`: Raw provider name.
///
/// # Returns
/// Trimmed provider name.
fn normalize_name(name: &str) -> String {
    name.trim().to_owned()
}

/// Normalizes owned provider names.
///
/// # Parameters
/// - `names`: Raw provider names.
///
/// # Returns
/// Trimmed provider names with empty entries removed.
fn normalize_owned_names(names: &[String]) -> Vec<String> {
    names
        .iter()
        .map(String::as_str)
        .map(normalize_name)
        .filter(|name| !name.is_empty())
        .collect()
}

/// Normalizes borrowed provider names.
///
/// # Parameters
/// - `names`: Raw provider names.
///
/// # Returns
/// Trimmed provider names with empty entries removed.
fn normalize_borrowed_names(names: &[&str]) -> Vec<String> {
    names
        .iter()
        .copied()
        .map(normalize_name)
        .filter(|name| !name.is_empty())
        .collect()
}
