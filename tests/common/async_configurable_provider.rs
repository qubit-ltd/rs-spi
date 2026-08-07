// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::AsyncServiceProvider;
use qubit_spi::ProviderFuture;
use qubit_spi::error::ProviderFailure;

use super::string_spec::StringSpec;
use super::test_error::TestError;

/// Asynchronous provider fixture supporting stable, echo, and failing outcomes.
pub(crate) struct AsyncConfigurableProvider {
    output: Option<Box<str>>,
    error: Option<ProviderFailure<TestError>>,
    echo_config: bool,
}

impl AsyncConfigurableProvider {
    /// Creates an asynchronous provider returning one stable output.
    pub(crate) fn success(output: impl Into<Box<str>>) -> Self {
        Self {
            output: Some(output.into()),
            error: None,
            echo_config: false,
        }
    }

    /// Creates an asynchronous provider returning its configuration.
    pub(crate) const fn echo() -> Self {
        Self {
            output: None,
            error: None,
            echo_config: true,
        }
    }

    /// Creates an asynchronous provider returning one classified failure.
    pub(crate) fn failure(error: ProviderFailure<TestError>) -> Self {
        Self {
            output: None,
            error: Some(error),
            echo_config: false,
        }
    }
}

impl AsyncServiceProvider<StringSpec> for AsyncConfigurableProvider {
    /// Produces the configured asynchronous outcome.
    fn create_configured<'a>(
        &'a self,
        config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        Box::pin(async move {
            if let Some(error) = &self.error {
                return Err(error.clone());
            }
            Ok(if self.echo_config {
                config.clone()
            } else {
                self.output.as_deref().unwrap_or_default().to_owned()
            })
        })
    }
}
