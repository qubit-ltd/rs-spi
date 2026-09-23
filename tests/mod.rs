// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Integration tests for the public Qubit SPI contract.

#[cfg(feature = "inventory")]
extern crate self as inventory_test_contract;

#[cfg(feature = "inventory")]
pub(crate) use common::external_string_spec::ExternalStringSpec;

mod common;
mod error;
#[cfg(feature = "inventory")]
mod inventory;
mod lib_tests;
mod provider;
mod registry;
mod selection;
mod service;
