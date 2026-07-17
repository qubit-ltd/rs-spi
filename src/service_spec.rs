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
/// by provider factories. Define one marker type implementing this trait for
/// each service family that needs independently typed provider registration.
pub trait ServiceSpec: 'static {
    /// Configuration type passed to provider factories.
    ///
    /// This may be unsized when providers operate on a dynamically sized
    /// configuration view.
    type Config: ?Sized;

    /// Complete output handle returned by provider factories.
    ///
    /// This is the service value returned directly after successful creation.
    type Output;
}
