// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider-controlled callbacks used to observe metadata sampling and lock
//! scope.

use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_spi::AsyncServiceProvider;
use qubit_spi::AsyncServiceSpec;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderFuture;
use qubit_spi::ProviderId;
use qubit_spi::ProviderMetadata;
use qubit_spi::ServiceProvider;
use qubit_spi::ServiceSpec;
use qubit_spi::SyncServiceSpec;
use qubit_spi::error::ProviderFailure;

use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;

/// Configurable callbacks deliberately reenter the public registry boundary.
pub(crate) struct RegistryContractProvider {
    pub(crate) metadata: Box<dyn Fn() -> ProviderDescriptor + Send + Sync>,
    pub(crate) on_drop: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ProviderMetadata for RegistryContractProvider {
    /// Invokes the provider-controlled metadata callback.
    fn descriptor(&self) -> ProviderDescriptor {
        (self.metadata)()
    }
}

impl ServiceProvider<StringSpec> for RegistryContractProvider {
    /// Returns an owned configuration copy for snapshot isolation assertions.
    fn create_configured(&self, config: &String) -> Result<String, ProviderFailure<TestError>> {
        Ok(config.clone())
    }
}

impl AsyncServiceProvider<StringSpec> for RegistryContractProvider {
    /// Returns an owned configuration copy when polled.
    fn create_configured<'a>(
        &'a self,
        config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        Box::pin(async move { Ok(config.clone()) })
    }
}

impl Drop for RegistryContractProvider {
    /// Reenters the registry only when this fixture explicitly requests it.
    fn drop(&mut self) {
        if let Some(callback) = &self.on_drop {
            callback();
        }
    }
}

/// Service output intentionally shares a provider-owned allocation.
pub(crate) struct SharedOutputSpec;

impl ServiceSpec for SharedOutputSpec {
    type Config = ();
    type Error = Infallible;
}

impl SyncServiceSpec for SharedOutputSpec {
    type Output = Arc<String>;
}

impl AsyncServiceSpec for SharedOutputSpec {
    type Output = Arc<String>;
}

/// Counts factory calls without promising a fresh underlying service
/// allocation.
pub(crate) struct SharedOutputProvider {
    pub(crate) output: Arc<String>,
    pub(crate) calls: Arc<AtomicUsize>,
}

impl ProviderMetadata for SharedOutputProvider {
    /// Returns stable metadata for the shared output fixture.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(ProviderId::new("shared").expect("valid fixture ID"))
    }
}

impl ServiceProvider<SharedOutputSpec> for SharedOutputProvider {
    /// Counts every factory invocation while returning the same resource.
    fn create_configured(&self, _config: &()) -> Result<Arc<String>, ProviderFailure<Infallible>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::clone(&self.output))
    }
}

impl AsyncServiceProvider<SharedOutputSpec> for SharedOutputProvider {
    /// Counts each polled factory invocation while returning the same resource.
    fn create_configured<'a>(
        &'a self,
        _config: &'a (),
    ) -> ProviderFuture<'a, Result<Arc<String>, ProviderFailure<Infallible>>> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::clone(&self.output))
        })
    }
}
