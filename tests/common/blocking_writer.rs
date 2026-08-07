// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

/// Formatter sink that pauses once after observing a configured marker.
pub(crate) struct BlockingWriter {
    /// Accumulated formatted output.
    output: String,
    /// Marker that triggers the one-time pause.
    marker: Box<str>,
    /// Notification sent when the marker is first observed.
    entered: Option<Sender<()>>,
    /// Signal that permits formatting to continue.
    release: Receiver<()>,
}

impl BlockingWriter {
    /// Creates a writer that pauses after `marker` appears in its output.
    ///
    /// # Parameters
    ///
    /// * `marker` - Text whose first appearance triggers the pause.
    /// * `entered` - Channel notified immediately before the pause.
    /// * `release` - Channel whose next value resumes formatting.
    ///
    /// # Returns
    ///
    /// An empty writer ready to coordinate one formatting pause.
    pub(crate) fn new(
        marker: &str,
        entered: Sender<()>,
        release: Receiver<()>,
    ) -> Self {
        Self {
            output: String::new(),
            marker: marker.into(),
            entered: Some(entered),
            release,
        }
    }

    /// Returns all text written to this sink.
    ///
    /// # Returns
    ///
    /// The complete accumulated formatter output.
    pub(crate) fn into_output(self) -> String {
        self.output
    }
}

impl fmt::Write for BlockingWriter {
    /// Appends text and performs the configured one-time pause.
    ///
    /// # Parameters
    ///
    /// * `value` - Formatter output to append.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending and, when applicable, resuming from the pause.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when either coordination channel is disconnected.
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.output.push_str(value);
        if self.entered.is_some() && self.output.contains(self.marker.as_ref())
        {
            let entered = self
                .entered
                .take()
                .expect("blocking writer pauses at most once");
            entered.send(()).map_err(|_| fmt::Error)?;
            self.release.recv().map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}
