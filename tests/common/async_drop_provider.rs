// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Observable async invocation, cancellation, and pre-future panic behavior.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;

use qubit_spi::AsyncServiceProvider;
use qubit_spi::ProviderFuture;
use qubit_spi::error::ProviderFailure;

use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;

/// Action taken when a leaf factory is invoked by the resolver.
pub(crate) enum AsyncAction {
    Pending,
    Absent,
    Panic,
    Success,
}

/// Records leaf invocation independently from polling its returned future.
pub(crate) struct AsyncDropProvider {
    pub(crate) id: &'static str,
    pub(crate) action: AsyncAction,
    pub(crate) calls: Arc<Mutex<Vec<&'static str>>>,
    pub(crate) drops: Arc<AtomicUsize>,
}

/// A never-ready future whose destruction is directly observable.
struct PendingOutput {
    drops: Arc<AtomicUsize>,
}

impl Future for PendingOutput {
    type Output = Result<String, ProviderFailure<TestError>>;

    /// Remains pending until its owner cancels the operation.
    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for PendingOutput {
    /// Records cancellation of this active provider future.
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

impl AsyncServiceProvider<StringSpec> for AsyncDropProvider {
    /// Records invocation before constructing the selected outcome future.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        self.calls.lock().expect("event lock must be available").push(self.id);
        match self.action {
            AsyncAction::Pending => Box::pin(PendingOutput {
                drops: Arc::clone(&self.drops),
            }),
            AsyncAction::Absent => Box::pin(async { Err(TestProviderFailure::unavailable("absent")) }),
            AsyncAction::Panic => panic!("leaf factory panics before returning its future"),
            AsyncAction::Success => Box::pin(async { Ok("later".to_owned()) }),
        }
    }
}
