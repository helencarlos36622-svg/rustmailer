// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::logger::{validate_log_level, LocalTimer};
use crate::modules::settings::cli::SETTINGS;
use crate::modules::settings::dir::DATA_DIR_MANAGER;
use std::sync::OnceLock;
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

pub static LOG_WORKER_GUARD: OnceLock<Vec<WorkerGuard>> = OnceLock::new();

pub fn setup_file_logger() -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
    validate_log_level(&SETTINGS.rustmailer_log_level);
    let level = SETTINGS.rustmailer_log_level.parse::<Level>().unwrap();
    let with_ansi = SETTINGS.rustmailer_ansi_logs;

    let (server_nonb, server_guard) = server_log_writer();
    LOG_WORKER_GUARD.set(vec![server_guard]).unwrap();

    let server_layer = fmt::layer()
        .with_timer(LocalTimer)
        .with_ansi(with_ansi)
        .with_level(true)
        .with_writer(server_nonb)
        .with_target(true);

    let subscriber = tracing_subscriber::registry()
        .with(LevelFilter::from_level(level))
        .with(server_layer);

    // Set the combined subscriber as the global default
    tracing::subscriber::set_global_default(subscriber)
}

fn server_log_writer() -> (NonBlocking, WorkerGuard) {
    let rolling = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("server")
        .max_log_files(SETTINGS.rustmailer_max_server_log_files)
        .build(DATA_DIR_MANAGER.log_dir.clone())
        .expect("failed to initialize rolling file appender");
    let (nb, wg) = tracing_appender::non_blocking(rolling);
    (nb, wg)
}
