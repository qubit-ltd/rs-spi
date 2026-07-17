// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderCreationError;
use qubit_spi::{
    ProviderDefinition,
    ProviderDescriptor,
    ServiceProvider,
};

use super::string_spec::StringSpec;

/// Provider fixture carrying its registration metadata and service output.
pub(crate) struct SelfDescribedProvider {
    descriptor: ProviderDescriptor,
    output: Box<str>,
}

impl SelfDescribedProvider {
    /// Creates a self-described provider fixture.
    ///
    /// # Arguments
    ///
    /// * `descriptor` - Registration metadata returned to the Registry.
    /// * `output` - Stable service output returned during creation.
    ///
    /// # Returns
    ///
    /// A provider containing both registration and creation state.
    pub(crate) fn new(
        descriptor: ProviderDescriptor,
        output: impl Into<Box<str>>,
    ) -> Self {
        Self {
            descriptor,
            output: output.into(),
        }
    }
}

impl ServiceProvider<StringSpec> for SelfDescribedProvider {
    /// Creates the provider's stable string service.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused string configuration.
    ///
    /// # Returns
    ///
    /// An owned copy of the configured output.
    fn create_configured(
        &self,
        _config: &String,
    ) -> Result<String, ProviderCreationError> {
        Ok(self.output.to_string())
    }
}

impl ProviderDefinition<StringSpec> for SelfDescribedProvider {
    /// Returns the descriptor carried by this Provider Definition.
    ///
    /// # Returns
    ///
    /// A descriptor snapshot for Registry ownership.
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}
