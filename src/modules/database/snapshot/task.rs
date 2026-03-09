// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::META_MODELS;
use crate::modules::scheduler::nativedb::TASK_MODELS;
use crate::modules::settings::cli::SETTINGS;
use crate::modules::settings::dir::{DATA_DIR_MANAGER, META_FILE, TASK_FILE};
use crate::{
    modules::{
        context::RustMailTask,
        error::{code::ErrorCode, RustMailerResult},
        scheduler::periodic::PeriodicTask,
    },
    raise_error,
};
use chrono::Local;
use native_db::Models;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::join;
use tokio::task::spawn_blocking;
use tracing::{error, info};

pub static TASK_INTERVAL: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_secs(SETTINGS.rustmailer_metadata_snapshot_interval_secs));

pub struct DatabaseSnapshotTask;

/// Periodic database snapshot task that creates backups for `meta.db` and `tasks.db`.
/// Runs every 15 minutes and retains only the latest N snapshot files (default: 10).
impl RustMailTask for DatabaseSnapshotTask {
    fn start() {
        if !SETTINGS.rustmailer_metadata_memory_mode_enabled {
            info!("[metadata] Snapshot task completed successfully.");
            return;
        }

        let periodic_task = PeriodicTask::new("database-snapshot-task");
        let task = move |_: Option<u64>| {
            Box::pin(async move {
                DatabaseSnapshotTask::snapshot().await.map_err(|e| {
                    raise_error!(
                        format!("Snapshot task failed: {:#?}", e),
                        ErrorCode::InternalError
                    )
                })?;
                info!("Snapshot task completed successfully.");
                Ok(())
            })
        };
        periodic_task.start(task, None, *TASK_INTERVAL, false, false);
    }
}

impl DatabaseSnapshotTask {
    fn generate_snapshot_filename(db_prefix: &str) -> String {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d-%H-%M").to_string();
        format!("{}.{}.snapshot", db_prefix, timestamp)
    }

    pub async fn snapshot() -> RustMailerResult<()> {
        let (meta_result, task_result) = join!(
            Self::run_snapshot(META_FILE, &META_MODELS),
            Self::run_snapshot(TASK_FILE, &TASK_MODELS)
        );
        meta_result?;
        task_result?;
        Self::prune_old_snapshots(10).await?;
        Ok(())
    }

    pub async fn block_snapshot() -> RustMailerResult<()> {
        Self::run_snapshot(META_FILE, &META_MODELS).await?;
        Self::run_snapshot(TASK_FILE, &TASK_MODELS).await
    }

    async fn run_snapshot(db_prefix: &str, models: &'static Models) -> RustMailerResult<()> {
        let file_name = Self::generate_snapshot_filename(db_prefix);
        let file_path = DATA_DIR_MANAGER.root_dir.join(&file_name);

        info!("Starting snapshot for {} to {:?}", db_prefix, file_path);

        spawn_blocking(move || DB_MANAGER.meta_db().snapshot(models, &file_path))
            .await
            .map_err(|join_err| {
                error!("{} snapshot task panicked: {:?}", db_prefix, join_err);
                raise_error!(
                    format!("{} snapshot task panicked: {:?}", db_prefix, join_err),
                    ErrorCode::InternalError
                )
            })?
            .map_err(|e| {
                error!("{} snapshot failed: {:?}", db_prefix, e);
                raise_error!(
                    format!("{} snapshot error: {:?}", db_prefix, e),
                    ErrorCode::InternalError
                )
            })?;

        info!("Completed snapshot for {}", db_prefix);
        Ok(())
    }

    async fn prune_old_snapshots(max_snapshots: usize) -> RustMailerResult<()> {
        if let Some(result) = DATA_DIR_MANAGER.find_oldest_snapshot_for(META_FILE) {
            if result.total >= max_snapshots {
                if let Some(oldest) = result.path {
                    tokio::fs::remove_file(&oldest).await.map_err(|e| {
                        raise_error!(
                            format!("Failed to delete old snapshot {:#?}: {:#?}", oldest, e),
                            ErrorCode::InternalError
                        )
                    })?;
                }
            }
        }

        if let Some(result) = DATA_DIR_MANAGER.find_oldest_snapshot_for(TASK_FILE) {
            if result.total >= max_snapshots {
                if let Some(oldest) = result.path {
                    tokio::fs::remove_file(&oldest).await.map_err(|e| {
                        raise_error!(
                            format!("Failed to delete old snapshot {:#?}: {:#?}", oldest, e),
                            ErrorCode::InternalError
                        )
                    })?;
                }
            }
        }

        Ok(())
    }
}
