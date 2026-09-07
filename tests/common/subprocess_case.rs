// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bounded subprocess isolation for tests that would otherwise hang on
//! deadlock.

use std::env;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;

/// Runs one exact test in a child, returning true only in that child's body.
pub(crate) fn enter() -> bool {
    let thread = thread::current();
    let name = thread.name().expect("test thread must have a name");
    if env::var("SPI_CONTRACT_CHILD").as_deref() == Ok(name) {
        return true;
    }
    let mut child = Command::new(env::current_exe().expect("test executable must be available"))
        .args(["--exact", name, "--nocapture"])
        .env("SPI_CONTRACT_CHILD", name)
        .spawn()
        .expect("isolated test must start");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().expect("child status must be readable") {
            assert!(status.success(), "isolated test failed: {name}");
            return false;
        }
        if Instant::now() >= deadline {
            child.kill().expect("hung child must be terminated");
            child.wait().expect("terminated child must be reaped");
            panic!("registry contract deadlocked: {name}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}
