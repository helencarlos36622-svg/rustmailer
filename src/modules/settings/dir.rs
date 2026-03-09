// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use chrono::NaiveDateTime;
use tracing::warn;

use crate::modules::context::Initialize;
use crate::modules::settings::cli::SETTINGS;
use crate::{
    modules::error::{code::ErrorCode, RustMailerResult},
    raise_error,
};
use std::path::PathBuf;
use std::sync::LazyLock;

pub const META_FILE: &str = "meta.db";
pub const TASK_FILE: &str = "tasks.db";
pub const ENVELOPE_FILE: &str = "envelope.db";
const DISK_CACHE_DIR: &str = "cache";
// const INDEX_DIR: &str = "index";
const LOG_DIR: &str = "logs";

const TLS_CERT: &str = "cert.pem";
const TLS_KEY: &str = "key.pem";

pub static DATA_DIR_MANAGER: LazyLock<DataDirManager> =
    LazyLock::new(|| DataDirManager::new(PathBuf::from(&SETTINGS.rustmailer_root_dir)));

#[derive(Debug)]
pub struct DataDirManager {
    pub root_dir: PathBuf,
    pub meta_db: PathBuf,
    pub task_db: PathBuf,
    pub envelope_db: PathBuf,
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
    pub disk_cache: PathBuf,
    // pub index: PathBuf,
    pub log_dir: PathBuf,
}

pub struct SnapshotScanResult {
    pub path: Option<PathBuf>,
    pub total: usize,
}

impl Initialize for DataDirManager {
    async fn initialize() -> RustMailerResult<()> {
        std::fs::create_dir_all(&DATA_DIR_MANAGER.root_dir)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        std::fs::create_dir_all(&DATA_DIR_MANAGER.disk_cache)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        std::fs::create_dir_all(&DATA_DIR_MANAGER.log_dir)
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?;
        Ok(())
    }
}

impl DataDirManager {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir: root_dir.clone(),
            meta_db: root_dir.join(META_FILE),
            task_db: root_dir.join(TASK_FILE),
            envelope_db: root_dir.join(ENVELOPE_FILE),
            tls_key: root_dir.join(TLS_KEY),
            tls_cert: root_dir.join(TLS_CERT),
            disk_cache: root_dir.join(DISK_CACHE_DIR),
            log_dir: root_dir.join(LOG_DIR),
        }
    }

    pub fn find_latest_snapshot_for(&self, db_prefix: &str) -> Option<PathBuf> {
        let pattern = format!("{}.*.snapshot", db_prefix);
        let pattern_path = self.root_dir.join(&pattern);
        let pattern_str = pattern_path.to_str()?;

        let mut snapshot_files = Vec::new();
        for entry in glob::glob(pattern_str).ok()? {
            if let Ok(path) = entry {
                snapshot_files.push(path);
            }
        }

        let mut dated_files: Vec<(NaiveDateTime, PathBuf)> = snapshot_files
            .into_iter()
            .filter_map(|path| {
                let filename = path.file_name()?.to_str()?;
                let timestamp_str = filename
                    .strip_prefix(&format!("{}.", db_prefix))?
                    .strip_suffix(".snapshot")?;
                NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d-%H-%M")
                    .ok()
                    .map(|dt| (dt, path))
            })
            .collect();

        dated_files.sort_by(|a, b| b.0.cmp(&a.0));
        dated_files.into_iter().next().map(|(_, path)| path)
    }

    pub fn find_oldest_snapshot_for(&self, db_prefix: &str) -> Option<SnapshotScanResult> {
        let pattern = format!("{}.*.snapshot", db_prefix);
        let pattern_path = self.root_dir.join(&pattern);
        let pattern_str = pattern_path.to_str()?;

        let mut snapshot_files = Vec::new();
        for entry in glob::glob(pattern_str).ok()? {
            if let Ok(path) = entry {
                snapshot_files.push(path);
            }
        }

        let mut dated_files: Vec<(NaiveDateTime, PathBuf)> = snapshot_files
            .into_iter()
            .filter_map(|path| {
                let filename = path.file_name()?.to_str()?;
                let timestamp_str = filename
                    .strip_prefix(&format!("{}.", db_prefix))?
                    .strip_suffix(".snapshot")?;
                NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d-%H-%M")
                    .ok()
                    .map(|dt| (dt, path))
            })
            .collect();

        let total = dated_files.len();
        if total == 0 {
            warn!("No snapshot files found for '{}'", db_prefix);
        }
        // Ascending order: oldest snapshot first
        dated_files.sort_by(|a, b| a.0.cmp(&b.0));
        let oldest = dated_files.into_iter().next().map(|(_, path)| path);

        Some(SnapshotScanResult {
            path: oldest,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, path::Path};
    use tempfile::tempdir;

    fn create_test_snapshot(dir: &Path, db_prefix: &str, timestamp: &str) {
        let filename = format!("{}.{}.snapshot", db_prefix, timestamp);
        File::create(dir.join(filename)).unwrap();
    }

    #[test]
    fn test_find_latest_snapshot_with_valid_files() {
        let temp_dir = tempdir().unwrap();
        let manager = DataDirManager::new(temp_dir.path().to_path_buf());

        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-16-44");
        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-16-54");
        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-17-04");

        let latest = manager.find_latest_snapshot_for("meta.db").unwrap();
        assert!(latest.ends_with("meta.db.2025-07-03-17-04.snapshot"));
    }

    #[test]
    fn test_find_latest_snapshot_with_empty_dir() {
        let temp_dir = tempdir().unwrap();
        let manager = DataDirManager::new(temp_dir.path().to_path_buf());

        assert!(manager.find_latest_snapshot_for("meta.db").is_none());
    }

    #[test]
    fn test_find_latest_snapshot_with_invalid_files() {
        let temp_dir = tempdir().unwrap();
        let manager = DataDirManager::new(temp_dir.path().to_path_buf());

        File::create(temp_dir.path().join("meta.db.invalid-format.snapshot")).unwrap();
        File::create(temp_dir.path().join("random_file.txt")).unwrap();

        assert!(manager.find_latest_snapshot_for("meta.db").is_none());

        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-16-44");
        let latest = manager.find_latest_snapshot_for("meta.db").unwrap();
        assert!(latest.ends_with("meta.db.2025-07-03-16-44.snapshot"));
    }

    #[test]
    fn test_find_latest_snapshot_timestamp_order() {
        let temp_dir = tempdir().unwrap();
        let manager = DataDirManager::new(temp_dir.path().to_path_buf());

        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-17-04");
        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-16-44");
        create_test_snapshot(temp_dir.path(), "meta.db", "2025-07-03-16-54");

        let latest = manager.find_latest_snapshot_for("meta.db").unwrap();
        assert!(latest.ends_with("meta.db.2025-07-03-17-04.snapshot"));
    }

    #[test]
    fn test_find_latest_snapshot_for_tasks_db() {
        let temp_dir = tempdir().unwrap();
        let manager = DataDirManager::new(temp_dir.path().to_path_buf());

        create_test_snapshot(temp_dir.path(), "tasks.db", "2025-07-03-10-00");
        create_test_snapshot(temp_dir.path(), "tasks.db", "2025-07-03-12-00");

        let latest = manager.find_latest_snapshot_for("tasks.db").unwrap();
        assert!(latest.ends_with("tasks.db.2025-07-03-12-00.snapshot"));
    }
}
