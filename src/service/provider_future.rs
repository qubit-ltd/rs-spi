// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Runtime-independent future type returned by asynchronous providers.

use std::future::Future;
use std::pin::Pin;

/// Sendable boxed future used by asynchronous provider APIs.
///
/// # Type Parameters
///
/// * `'a` - Maximum lifetime of data borrowed by the future.
/// * `T` - Value produced when the future completes.
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
