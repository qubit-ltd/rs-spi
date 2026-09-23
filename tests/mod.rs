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
pub struct ExternalStringSpec;

#[cfg(feature = "inventory")]
impl qubit_spi::ServiceSpec for ExternalStringSpec {
    type Config = String;
    type Error = std::convert::Infallible;
}

#[cfg(feature = "inventory")]
impl qubit_spi::SyncServiceSpec for ExternalStringSpec {
    type Output = String;
}

#[cfg(feature = "inventory")]
impl qubit_spi::AsyncServiceSpec for ExternalStringSpec {
    type Output = String;
}

mod common;
mod error;
#[cfg(feature = "inventory")]
mod inventory;
mod lib_tests;
mod provider;
mod registry;
mod selection;
mod service;
