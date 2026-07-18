// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Formatter sink that fails after a configured number of writes.

use std::fmt;

/// Formatter sink used to exercise Display error propagation.
pub(crate) struct FailingWriter {
    /// Number of writes that may succeed before the sink rejects output.
    remaining_successes: usize,
}

impl FailingWriter {
    /// Creates a writer with a fixed successful-write allowance.
    ///
    /// # Parameters
    ///
    /// * `remaining_successes` - Writes accepted before returning
    ///   [`fmt::Error`].
    ///
    /// # Returns
    ///
    /// A formatter sink with the requested failure point.
    #[inline]
    #[must_use]
    pub(crate) const fn new(remaining_successes: usize) -> Self {
        Self {
            remaining_successes,
        }
    }
}

impl fmt::Write for FailingWriter {
    /// Accepts output until the configured successful-write count is exhausted.
    ///
    /// # Parameters
    ///
    /// * `value` - Formatted text supplied by the Display implementation.
    ///
    /// # Returns
    ///
    /// `Ok(())` while the allowance remains.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] after the configured number of successful writes.
    fn write_str(&mut self, _value: &str) -> fmt::Result {
        if self.remaining_successes == 0 {
            Err(fmt::Error)
        } else {
            self.remaining_successes -= 1;
            Ok(())
        }
    }
}
