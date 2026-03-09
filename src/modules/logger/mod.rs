// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::logger::file::setup_file_logger;
use crate::modules::settings::cli::SETTINGS;
use chrono::Local;
use std::process;
use tracing::Level;
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

mod file;

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z"))
    }
}

pub fn initialize_logging() {
    if SETTINGS.rustmailer_log_to_file {
        setup_file_logger().unwrap();
    } else {
        setup_stdout_logger().unwrap();
    }
}

fn setup_stdout_logger() -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
    validate_log_level(&SETTINGS.rustmailer_log_level);
    let level = SETTINGS.rustmailer_log_level.parse::<Level>().unwrap();
    let with_ansi = SETTINGS.rustmailer_ansi_logs;

    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(true)
        .with_timer(LocalTimer);

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(with_ansi)
        .with_writer(std::io::stdout)
        .event_format(format)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
}

fn validate_log_level(value: &String) {
    if value.parse::<Level>().is_err() {
        eprintln!(
            "Invalid log level specified. Use one of: error, warn, info, debug, trace. 
        The log level you currently specified is 'rustmailer_log_level'='{}'",
            value
        );
        process::exit(1);
    }
}
