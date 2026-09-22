// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt;

/// Identifies the source location of a provider registration submitted through
/// link-time discovery.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProviderRegistrationSource {
    crate_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
}

impl ProviderRegistrationSource {
    /// Creates a source location from its crate, module, file, and line.
    ///
    /// # Parameters
    ///
    /// * `crate_name` - Name of the crate that submitted the provider.
    /// * `module_path` - Fully-qualified module path of the submission.
    /// * `file` - Source file containing the submission.
    /// * `line` - One-based source line containing the submission.
    ///
    /// # Returns
    ///
    /// A source location retaining the supplied static declaration details.
    #[must_use]
    pub const fn new(crate_name: &'static str, module_path: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            crate_name,
            module_path,
            file,
            line,
        }
    }

    /// Returns the crate that submitted the provider.
    ///
    /// # Returns
    ///
    /// The static package name recorded for the submission.
    #[inline(always)]
    #[must_use]
    pub const fn crate_name(self) -> &'static str {
        self.crate_name
    }

    /// Returns the module path containing the provider submission.
    ///
    /// # Returns
    ///
    /// The static module path recorded for the submission.
    #[inline(always)]
    #[must_use]
    pub const fn module_path(self) -> &'static str {
        self.module_path
    }

    /// Returns the source file containing the provider submission.
    ///
    /// # Returns
    ///
    /// The static source file path recorded for the submission.
    #[inline(always)]
    #[must_use]
    pub const fn file(self) -> &'static str {
        self.file
    }

    /// Returns the one-based source line containing the provider submission.
    ///
    /// # Returns
    ///
    /// The one-based line number recorded for the submission.
    #[inline(always)]
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }
}

impl fmt::Display for ProviderRegistrationSource {
    /// Formats the complete provider-registration declaration location.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects diagnostic output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}::{}, {}:{}",
            self.crate_name, self.module_path, self.file, self.line,
        )
    }
}
