// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous service provider resolution.

#[path = "async_resolving_service_provider/async_resolving_service_provider.rs"]
mod implementation;

pub use implementation::AsyncResolvingServiceProvider;
