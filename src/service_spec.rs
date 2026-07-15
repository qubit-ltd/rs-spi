// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Service specification binding configuration and output handles.

/// Type-level description of one pluggable service family.
///
/// A service specification gives a registry one type parameter that carries the
/// configuration type accepted by providers and the complete handle returned
/// by provider factories.
pub trait ServiceSpec: 'static {
    /// Configuration type passed to provider factories.
    type Config: ?Sized;

    /// Complete output handle returned by providers.
    type Output;
}
