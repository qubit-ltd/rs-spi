// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policies controlling fallback after provider creation failures.

#[path = "fallback_policy/fallback_policy.rs"]
mod implementation;

pub use implementation::FallbackPolicy;
