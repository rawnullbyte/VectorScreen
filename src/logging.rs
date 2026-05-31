use std::fs::OpenOptions;

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const LOG_FILE: &str = "/tmp/vector-screen.log";

/// Initialize structured logging with tracing.
///
/// - `level`: Log level filter (e.g., "info", "debug", "trace")
/// - `log_file`: Optional custom log file path. Defaults to `/tmp/vector-screen.log`
pub fn init_logging(level: &str, log_file: Option<&str>) {
    let log_path = log_file.unwrap_or(LOG_FILE);

    // Rotate log file if it exceeds MAX_LOG_SIZE
    if let Ok(metadata) = std::fs::metadata(log_path) {
        if metadata.len() > MAX_LOG_SIZE {
            let rotated = format!("{}.1", log_path);
            let _ = std::fs::rename(log_path, &rotated);
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to open log file {}: {}", log_path, e);
            std::process::abort();
        });

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(file)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_timer(fmt::time::SystemTime),
        )
        .init();
}
