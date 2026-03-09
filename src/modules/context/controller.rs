// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{cache::imap::task::SYNC_TASKS, error::RustMailerResult};
use std::{sync::LazyLock, time::Duration};
use tokio::sync::mpsc;
use tracing::{error, info};

pub static SYNC_CONTROLLER: LazyLock<SyncController> = LazyLock::new(SyncController::new);

pub struct SyncController {
    channel: mpsc::Sender<(u64, String)>, // Channel to trigger account sync by account ID
}

impl SyncController {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<(u64, String)>(100);

        tokio::spawn(async move {
            while let Some((account_id, email)) = rx.recv().await {
                match Self::start_syncer(account_id, email.clone()).await {
                    Ok(Some(_)) => {}
                    Ok(None) => {}
                    Err(err) => {
                        error!(
                            "Failed to prepare and start syncer of account {}-{}, error: {:#?}",
                            &account_id, &email, err
                        );
                    }
                }
            }
        });

        SyncController { channel: tx }
    }

    /// Trigger synchronization for a specific account
    pub async fn trigger_start(&self, account_id: u64, email: String) {
        if let Err(e) = self.channel.send((account_id, email)).await {
            error!("Failed to send trigger event: {:?}", e);
        }
    }

    async fn start_syncer(account_id: u64, email: String) -> RustMailerResult<Option<()>> {
        info!(
            "Account syncer starting for account: {}-{}.",
            account_id, email
        );
        SYNC_TASKS.start_account_sync_task(account_id, email).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(Some(()))
    }
}
