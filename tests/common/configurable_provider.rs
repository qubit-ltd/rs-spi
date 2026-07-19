// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::{
    Arc,
    Mutex,
    atomic::{
        AtomicUsize,
        Ordering,
    },
};

use qubit_spi::ServiceProvider;
use qubit_spi::error::ProviderError;

use super::string_spec::StringSpec;

/// Provider fixture supporting success, echo, failure, and call recording.
pub(crate) struct ConfigurableProvider {
    output: Option<Box<str>>,
    error: Option<ProviderError>,
    echo_config: bool,
    calls: Option<Arc<AtomicUsize>>,
    seen_config: Option<Arc<Mutex<Option<String>>>>,
}

impl ConfigurableProvider {
    /// Creates a provider returning one stable output.
    ///
    /// # Parameters
    ///
    /// * `output` - String returned by every creation call.
    ///
    /// # Returns
    ///
    /// A successful provider fixture.
    pub(crate) fn success(output: impl Into<Box<str>>) -> Self {
        Self {
            output: Some(output.into()),
            error: None,
            echo_config: false,
            calls: None,
            seen_config: None,
        }
    }

    /// Creates a provider returning its input configuration.
    ///
    /// # Returns
    ///
    /// A successful echo provider fixture.
    pub(crate) fn echo() -> Self {
        Self {
            output: None,
            error: None,
            echo_config: true,
            calls: None,
            seen_config: None,
        }
    }

    /// Creates a provider returning one classified leaf failure.
    ///
    /// # Parameters
    ///
    /// * `error` - Provider error cloned for every creation call.
    ///
    /// # Returns
    ///
    /// A failing provider fixture.
    pub(crate) fn failure(error: ProviderError) -> Self {
        Self {
            output: None,
            error: Some(error),
            echo_config: false,
            calls: None,
            seen_config: None,
        }
    }

    /// Adds a shared invocation counter.
    ///
    /// # Parameters
    ///
    /// * `calls` - Counter incremented before each configured outcome.
    ///
    /// # Returns
    ///
    /// This provider fixture with call recording enabled.
    pub(crate) fn with_calls(mut self, calls: Arc<AtomicUsize>) -> Self {
        self.calls = Some(calls);
        self
    }

    /// Adds shared configuration recording.
    ///
    /// # Parameters
    ///
    /// * `seen_config` - Slot replaced with each received configuration.
    ///
    /// # Returns
    ///
    /// This provider fixture with configuration recording enabled.
    pub(crate) fn with_seen_config(
        mut self,
        seen_config: Arc<Mutex<Option<String>>>,
    ) -> Self {
        self.seen_config = Some(seen_config);
        self
    }
}

impl ServiceProvider<StringSpec> for ConfigurableProvider {
    /// Produces the configured output while recording invocation state.
    ///
    /// # Parameters
    ///
    /// * `config` - String configuration optionally echoed and recorded.
    ///
    /// # Returns
    ///
    /// The configured stable output or a clone of `config` for echo fixtures.
    ///
    /// # Errors
    ///
    /// Returns the configured leaf provider failure.
    fn create_configured(
        &self,
        config: &String,
    ) -> Result<String, ProviderError> {
        if let Some(calls) = &self.calls {
            calls.fetch_add(1, Ordering::SeqCst);
        }
        if let Some(seen_config) = &self.seen_config {
            match seen_config.lock() {
                Ok(mut seen_config) => *seen_config = Some(config.clone()),
                Err(poisoned) => {
                    *poisoned.into_inner() = Some(config.clone());
                }
            }
        }
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        if self.echo_config {
            return Ok(config.clone());
        }
        Ok(self.output.as_deref().unwrap_or_default().to_owned())
    }
}
