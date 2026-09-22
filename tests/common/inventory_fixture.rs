// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Subprocess support for the cross-crate inventory fixture.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static TARGET_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

/// Runs one fixture binary with a target directory isolated from its sources.
///
/// # Parameters
///
/// * `binary` - Consumer binary that demonstrates one linked-provider case.
///
/// # Returns
///
/// The completed Cargo process output, including its captured standard streams.
///
/// # Panics
///
/// Panics when the fixture location cannot be resolved or Cargo cannot start.
pub(crate) fn run(binary: &str) -> Output {
    let fixture_root = fixture_root();
    let target = TemporaryTargetDir::new(binary);
    Command::new("cargo")
        .args(["run", "--quiet", "--locked", "--offline", "--bin", binary])
        .current_dir(&fixture_root)
        .env("CARGO_TARGET_DIR", target.path())
        .output()
        .expect("fixture Cargo process must start")
}

/// Returns the root directory of the fixture workspace.
///
/// # Returns
///
/// The checked-in fixture workspace resolved from this crate's manifest root.
///
/// # Panics
///
/// Panics when Cargo's manifest-directory environment variable is absent.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("provider_inventory")
}

/// Removes one uniquely assigned temporary target directory when it goes out
/// of scope.
struct TemporaryTargetDir {
    path: PathBuf,
}

impl TemporaryTargetDir {
    /// Creates a target directory unique to this fixture process invocation.
    ///
    /// # Parameters
    ///
    /// * `binary` - Binary name used only to make the temporary path readable.
    ///
    /// # Returns
    ///
    /// An RAII cleanup guard for the allocated target directory.
    fn new(binary: &str) -> Self {
        let sequence = TARGET_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "qubit-spi-provider-inventory-{}-{}-{sequence}",
            std::process::id(),
            binary
        ));
        fs::create_dir(&path).expect("fixture temporary target directory must be unique");
        Self { path }
    }

    /// Returns the isolated target directory passed to the nested Cargo
    /// process.
    ///
    /// # Returns
    ///
    /// A borrowed filesystem path that Cargo may create and populate.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryTargetDir {
    /// Deletes only the temporary target directory allocated by this guard.
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).expect("fixture temporary target directory must be removable");
        }
    }
}
