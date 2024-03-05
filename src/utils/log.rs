//! Log Module

use colored::Colorize;
use std::sync::{Once, RwLock};

static INIT: Once = Once::new();
static LOG_LEVEL: RwLock<LogLevel> = RwLock::new(LogLevel::Info);

/// This enum is used to represent the different log levels
#[derive(PartialEq, PartialOrd, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Log,
    Warn,
    Error,
}

/// Initializes the log level, which is called only once when the program starts
fn init_log_level() {
    let level = std::env::var("RUXGO_LOG_LEVEL").unwrap_or_else(|_| "Info".to_string());
    let log_level = match level.as_str() {
        "Debug" => LogLevel::Debug,
        "Info" => LogLevel::Info,
        "Log" => LogLevel::Log,
        "Warn" => LogLevel::Warn,
        "Error" => LogLevel::Error,
        _ => LogLevel::Log,
    };

    // Use write lock to update the log level
    let mut write_lock = LOG_LEVEL.write().unwrap();
    *write_lock = log_level;
}

/// This function is used to log messages to the console
/// # Arguments
/// * `level` - The log level of the message
/// * `message` - The message to log
/// # Example
/// ```
/// log(LogLevel::Info, "Hello World!");
/// log(LogLevel::Error, &format!("Something went wrong! {}", error));
/// ```
///
/// # Level setting
/// The log level can be set by setting the environment variable `RUXGO_LOG_LEVEL`
/// to one of the following values:
/// * `Debug`
/// * `Info`
/// * `Log`
/// * `Warn`
/// * `Error`
/// If the environment variable is not set, the default log level is `Log`
pub fn log(level: LogLevel, message: &str) {
    INIT.call_once(|| {
        init_log_level();
    });
    let level_str = match level {
        LogLevel::Debug => "[DEBUG]".purple(),
        LogLevel::Info => "[INFO]".blue(),
        LogLevel::Log => "[LOG]".green(),
        LogLevel::Warn => "[WARN]".yellow(),
        LogLevel::Error => "[ERROR]".red(),
    };
    // Use read lock to check log level
    if level >= *LOG_LEVEL.read().unwrap() {
        println!("{} {}", level_str, message);
    }
}
