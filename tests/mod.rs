// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Integration tests for the public Qubit SPI contract.

mod async_provider_definition_tests;
mod async_provider_registry_tests;
mod async_resolving_service_provider_tests;
mod async_service_provider_tests;
mod async_service_spec_tests;
mod common;
mod error;
mod fallback_policy_tests;
mod internal;
mod lib_tests;
mod missing_provider_policy_tests;
mod provider_creation_termination_tests;
mod provider_definition_tests;
mod provider_descriptor_tests;
mod provider_future_tests;
mod provider_id_tests;
mod provider_metadata_tests;
mod provider_registry_tests;
mod provider_selection_target_ref_tests;
mod provider_selection_tests;
mod provider_selector_tests;
mod resolving_service_provider_tests;
mod service_provider_tests;
mod service_spec_tests;
mod sync_service_spec_tests;
