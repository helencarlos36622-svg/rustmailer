// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use tracing::info;

use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::META_MODELS;
use crate::modules::settings::dir::META_FILE;
use crate::{
    modules::{
        context::RustMailTask,
        error::{code::ErrorCode, RustMailerResult},
        scheduler::periodic::PeriodicTask,
        settings::cli::SETTINGS,
    },
    raise_error,
};
use std::{path::PathBuf, time::Duration};

const TASK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // Daily backups

///This task periodically backs up meta.db file to a specified backup directory, retaining a configurable number of recent backups.
pub struct MetaBackupTask;

impl RustMailTask for MetaBackupTask {
    fn start() {
        if SETTINGS.rustmailer_metadata_memory_mode_enabled {
            info!("Backup task skipped: backup is only supported in non-memory mode.");
            return;
        }

        let Some(backup_dir) = SETTINGS.rustmailer_backup_dir.clone() else {
            info!("Backup task skipped: no backup directory specified");
            return;
        };
        let periodic_task = PeriodicTask::new("meta-backup-task");
        let task = move |_: Option<u64>| {
            let backup_dir = backup_dir.clone();
            Box::pin(async move {
                MetaBackupTask::backup_files(&backup_dir, SETTINGS.rustmailer_max_backups)
                    .await
                    .map_err(|e| {
                        raise_error!(
                            format!("Backup task failed: {:#?}", e),
                            ErrorCode::InternalError
                        )
                    })?;
                info!(
                    "Backup task completed successfully. Directory: {}, Max backups: {}",
                    backup_dir.display(),
                    SETTINGS.rustmailer_max_backups
                );
                Ok(())
            })
        };

        periodic_task.start(task, None, TASK_INTERVAL, false, false);
    }
}

impl MetaBackupTask {
    pub async fn backup_files(backup_dir: &PathBuf, max_backups: usize) -> RustMailerResult<()> {
        // Ensure the backup directory exists
        tokio::fs::create_dir_all(backup_dir).await.map_err(|e| {
            raise_error!(
                format!("Failed to create backup directory: {:#?}", e),
                ErrorCode::InternalError
            )
        })?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

        let backup_filename = format!("{}_{}", timestamp, META_FILE);
        let backup_path = backup_dir.join(backup_filename);

        tokio::task::spawn_blocking(move || {
            DB_MANAGER.meta_db().snapshot(&META_MODELS, &backup_path)
        })
        .await
        .map_err(|join_err| {
            raise_error!(
                format!("Snapshot task panicked: {:#?}", join_err),
                ErrorCode::InternalError
            )
        })?
        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        // Manage backup retention
        Self::prune_old_backups(backup_dir, max_backups).await?;

        Ok(())
    }

    async fn prune_old_backups(backup_dir: &PathBuf, max_backups: usize) -> RustMailerResult<()> {
        // Collect backup files for this file type
        let mut backups = Vec::new();
        let mut entries = tokio::fs::read_dir(backup_dir).await.map_err(|e| {
            raise_error!(
                format!("Failed to read backup directory: {:#?}", e),
                ErrorCode::InternalError
            )
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            raise_error!(
                format!("Failed to read backup directory entry: {:#?}", e),
                ErrorCode::InternalError
            )
        })? {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                backups.push((path.clone(), file_name.to_string()));
            }
        }

        // Sort by filename (timestamp) in descending order (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Delete older backups beyond max_backups
        let backups_to_keep = backups.iter().take(max_backups).collect::<Vec<_>>();
        for old_backup in backups.iter() {
            if !backups_to_keep.contains(&old_backup) {
                tokio::fs::remove_file(&old_backup.0).await.map_err(|e| {
                    raise_error!(
                        format!("Failed to delete old backup {}: {:#?}", old_backup.1, e),
                        ErrorCode::InternalError
                    )
                })?;
            }
        }
        Ok(())
    }
}
