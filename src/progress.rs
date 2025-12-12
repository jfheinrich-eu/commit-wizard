//! Progress indicators for long-running operations.
//!
//! This module provides spinner animations and other progress indicators
//! for operations that may take some time to complete.

use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Progress indicator that runs in background and animates.
///
/// The spinner displays an animated spinner character along with a message
/// to indicate progress. It runs in a separate thread and automatically
/// stops when dropped or when `stop()` is called.
///
/// # Example
///
/// ```no_run
/// use commit_wizard::progress::ProgressSpinner;
///
/// let spinner = ProgressSpinner::new("Processing files", 1, 3);
/// // ... do work ...
/// spinner.stop();
/// ```
pub struct ProgressSpinner {
    running: Arc<AtomicBool>,
    handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl Drop for ProgressSpinner {
    /// Ensures the spinner thread is stopped when the instance is dropped.
    ///
    /// This prevents the background thread from continuing to run if the
    /// spinner goes out of scope without an explicit `stop()` call.
    fn drop(&mut self) {
        self.stop();
    }
}

impl ProgressSpinner {
    /// Creates a new progress spinner with a message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display alongside the spinner
    /// * `step` - The current step number (for displaying progress like [1/3])
    /// * `total` - The total number of steps
    ///
    /// # Returns
    ///
    /// A new `ProgressSpinner` instance that starts animating immediately.
    /// If stderr is not a terminal, the spinner will not display anything.
    pub fn new(message: impl Into<String>, step: usize, total: usize) -> Self {
        let message = message.into();
        let running = Arc::new(AtomicBool::new(true));

        let msg_clone = message.clone();
        let running_clone = running.clone();

        let handle = if std::io::stderr().is_terminal() {
            Some(thread::spawn(move || {
                let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let mut idx = 0;

                while running_clone.load(Ordering::Relaxed) {
                    eprint!(
                        "\r\x1B[2K[{}/{}] {} {}",
                        step, total, spinners[idx], msg_clone
                    );
                    let _ = io::stderr().flush();

                    idx = (idx + 1) % spinners.len();
                    thread::sleep(Duration::from_millis(100));
                }

                // Clear line when done
                eprint!("\r\x1B[2K");
                let _ = io::stderr().flush();
            }))
        } else {
            None
        };

        Self {
            running,
            handle: Arc::new(Mutex::new(handle)),
        }
    }

    /// Stops the spinner animation and waits for the thread to finish.
    ///
    /// This method is idempotent and can be called multiple times safely.
    /// Subsequent calls after the first will have no effect. If the thread
    /// panicked, a warning is printed to stderr but the method does not panic.
    pub fn stop(&self) {
        // Atomically set running to false and check if we were the one to do it
        if self.running.swap(false, Ordering::Relaxed) {
            // We were running, so we need to clean up
            let mut handle_guard = self.handle.lock().unwrap();
            if let Some(handle) = handle_guard.take() {
                if let Err(e) = handle.join() {
                    eprintln!("Warning: spinner thread panicked: {:?}", e);
                }
            }
        }
        // If running was already false, another thread already stopped us - do nothing
    }
}
