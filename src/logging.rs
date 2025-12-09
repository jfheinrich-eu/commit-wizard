//! Logging configuration for commit-wizard.
//!
//! This module provides structured logging with file output support.
//! Logs are written to `~/.local/share/commit-wizard/commit-wizard.log` by default (XDG-compliant).
//! If the data directory is not writable (e.g., on macOS, Windows, or without proper permissions),
//! the tool will automatically fall back to writing logs to `./commit-wizard.log` in the current directory.
//! Users can also explicitly request local logging with the `--log-local` flag.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{error, info, set_logger, set_max_level, warn, LevelFilter, Log, Metadata, Record};

/// Local log file name
const LOCAL_LOG_FILE: &str = "commit-wizard.log";

/// Returns the default log file path in the user's data directory (XDG-compliant).
fn default_log_path() -> PathBuf {
    if let Some(mut dir) = dirs::data_dir() {
        dir.push("commit-wizard");
        let _ = std::fs::create_dir_all(&dir); // Ensure directory exists
        dir.push("commit-wizard.log");
        dir
    } else {
        // Fallback to home directory if data_dir is not available
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".local");
        home.push("share");
        home.push("commit-wizard");
        let _ = std::fs::create_dir_all(&home);
        home.push("commit-wizard.log");
        home
    }
}

/// Custom logger that writes to a file
struct FileLogger {
    file: Mutex<File>,
    level: LevelFilter,
}

impl FileLogger {
    fn new(path: &Path, level: LevelFilter) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            file: Mutex::new(file),
            level,
        })
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level = record.level();
        let target = record.target();
        let message = record.args();

        let log_line = format!("[{}] {} [{}] {}\n", timestamp, level, target, message);

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

/// Initializes the logging system.
///
/// # Arguments
///
/// * `enabled` - Whether logging is enabled (controlled by --log flag)
/// * `use_local_path` - If true, writes to ./commit-wizard.log, otherwise /var/log/commit-wizard.log
/// * `verbose` - If true, sets log level to DEBUG, otherwise INFO
///
/// # Returns
///
/// Returns Ok(Some(path)) if logging was initialized successfully,
/// Ok(None) if logging is disabled,
/// Err if initialization failed.
pub fn init_logging(
    enabled: bool,
    use_local_path: bool,
    verbose: bool,
) -> anyhow::Result<Option<PathBuf>> {
    if !enabled {
        return Ok(None);
    }

    let log_path = if use_local_path {
        PathBuf::from(LOCAL_LOG_FILE)
    } else {
        default_log_path()
    };

    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // Try to create the logger
    match FileLogger::new(&log_path, level) {
        Ok(logger) => {
            set_logger(Box::leak(Box::new(logger)))
                .map_err(|e| anyhow::anyhow!("Failed to set logger: {}", e))?;
            set_max_level(level);

            info!("=== Commit Wizard Started ===");
            info!("Log level: {}", level);

            Ok(Some(log_path))
        }
        Err(e) => {
            // If default path fails, try local path as fallback
            if !use_local_path {
                eprintln!("⚠️  Failed to write to {}: {}", log_path.display(), e);
                eprintln!("   Trying local directory instead...");

                let local_path = PathBuf::from(LOCAL_LOG_FILE);
                let logger = FileLogger::new(&local_path, level).map_err(|e2| {
                    anyhow::anyhow!(
                        "Failed to initialize logging (tried both {} and {}): {} / {}",
                        log_path.display(),
                        LOCAL_LOG_FILE,
                        e,
                        e2
                    )
                })?;

                set_logger(Box::leak(Box::new(logger)))
                    .map_err(|e| anyhow::anyhow!("Failed to set logger: {}", e))?;
                set_max_level(level);

                info!("=== Commit Wizard Started ===");
                info!("Log level: {}", level);
                warn!("Using fallback log path due to permission error");

                Ok(Some(local_path))
            } else {
                anyhow::bail!(
                    "Failed to initialize logging at {}: {}",
                    log_path.display(),
                    e
                );
            }
        }
    }
}

/// Logs an error with context
pub fn log_error(context: &str, error: &anyhow::Error) {
    error!("{}: {}", context, error);

    // Log the full error chain
    for (i, cause) in error.chain().enumerate().skip(1) {
        error!("  Caused by ({}): {}", i, cause);
    }
}

/// Logs API request details
pub fn log_api_request(provider: &str, model: &str, prompt_length: usize) {
    info!(
        "API Request: provider={}, model={}, prompt_length={}",
        provider, model, prompt_length
    );
}

/// Logs API response details
pub fn log_api_response(provider: &str, success: bool, response_length: Option<usize>) {
    if success {
        info!(
            "API Response: provider={}, success=true, response_length={}",
            provider,
            response_length.unwrap_or(0)
        );
    } else {
        error!("API Response: provider={}, success=false", provider);
    }
}

/// Logs file grouping results
pub fn log_grouping_result(files_count: usize, groups_count: usize, ai_used: bool) {
    info!(
        "Grouping: files={}, groups={}, ai_used={}",
        files_count, groups_count, ai_used
    );
}
