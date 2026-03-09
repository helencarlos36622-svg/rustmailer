// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::{Arc, LazyLock};

use tokio::sync::Semaphore;

use crate::modules::settings::cli::SETTINGS;

pub mod disk;
pub mod imap;
pub mod model;
pub mod sync_type;
pub mod vendor;

pub static SEMAPHORE: LazyLock<Arc<Semaphore>> = LazyLock::new(|| {
    Arc::new(Semaphore::new(
        SETTINGS
            .rustmailer_sync_concurrency
            .map(|c| c as usize)
            .unwrap_or(num_cpus::get() * 2),
    ))
});
