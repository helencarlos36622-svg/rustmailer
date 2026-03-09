// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::entity::{AuthType, MailerType};
use crate::modules::cache::imap::sync::execute_imap_sync;
use crate::modules::cache::vendor::gmail::sync::execute_gmail_sync;
use crate::modules::cache::vendor::outlook::sync::execute_outlook_sync;
use crate::modules::oauth2::token::OAuth2AccessToken;
use crate::modules::scheduler::periodic::TaskHandle;
use crate::modules::{
    account::{dispatcher::STATUS_DISPATCHER, migration::AccountModel},
    error::RustMailerResult,
    scheduler::periodic::PeriodicTask,
};
use crate::utc_now;
use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::{sync::LazyLock, time::Duration};
use tracing::{error, warn};

static _DESCRIPTION: &str = "This task periodically synchronizes mailbox data for a specified account, ensuring that all local data is up-to-date.";
const TASK_INTERVAL: Duration = Duration::from_secs(10);
pub static SYNC_TASKS: LazyLock<AccountSyncTask> = LazyLock::new(AccountSyncTask::new);
static LAST_WARN_TIME: AtomicI64 = AtomicI64::new(0);
const WARN_INTERVAL_MS: i64 = 600_000;

pub struct AccountSyncTask {
    tasks: DashMap<u64, TaskHandle>,
}

impl AccountSyncTask {
    pub fn new() -> Self {
        Self {
            tasks: DashMap::new(),
        }
    }

    pub async fn start_account_sync_task(&self, account_id: u64, email: String) {
        let task_name = format!("account-sync-task-{}-{}", account_id, &email);
        let periodic_task = PeriodicTask::new(&task_name);
        let task = move |param: Option<u64>| {
            let account_id = param.unwrap();
            Box::pin(async move {
                let account = AccountModel::get(account_id).await.ok();
                match account {
                    Some(account) => {
                        if !account.enabled {
                            let last = LAST_WARN_TIME.load(Ordering::Relaxed);
                            let now = utc_now!();
                            if now - last >= WARN_INTERVAL_MS {
                                LAST_WARN_TIME.store(now, Ordering::Relaxed);
                                warn!(
                                    "Account {}: Sync aborted. Account is currently disabled.",
                                    account_id
                                );
                            }
                        } else {
                            match account.mailer_type {
                                MailerType::ImapSmtp => {
                                    if let AuthType::OAuth2 = account.imap.as_ref().expect("BUG: account.imap is None, but this should never happen here").auth.auth_type {
                                        if OAuth2AccessToken::get(account.id).await?.is_none() {
                                            if utc_now!() % 300_000 == 0 {
                                                warn!("Account {}: Sync aborted. OAuth2 authorization not completed. Please visit the rustmailer admin page to authorize this account.", account_id);
                                            }
                                            return Ok(());
                                        }
                                    }
                                    if let Err(e) = execute_imap_sync(&account).await {
                                        STATUS_DISPATCHER
                                            .append_error(
                                                account_id,
                                                format!("error in account sync task: {:#?}", e),
                                            )
                                            .await;
                                        error!(
                                            "Failed to synchronize mailbox data for '{}': {:?}",
                                            account_id, e
                                        )
                                    }
                                }
                                MailerType::GmailApi => {
                                    if OAuth2AccessToken::get(account.id).await?.is_none() {
                                        if utc_now!() % 300_000 == 0 {
                                            warn!("Account {}: Sync aborted. OAuth2 authorization not completed. Please visit the rustmailer admin page to authorize this account.", account_id);
                                        }
                                        return Ok(());
                                    }
                                    if let Err(e) = execute_gmail_sync(&account).await {
                                        STATUS_DISPATCHER
                                            .append_error(
                                                account_id,
                                                format!("error in account sync task: {:#?}", e),
                                            )
                                            .await;
                                        error!(
                                            "Failed to synchronize mailbox data for '{}': {:?}",
                                            account_id, e
                                        )
                                    }
                                }
                                MailerType::GraphApi => {
                                    if OAuth2AccessToken::get(account.id).await?.is_none() {
                                        if utc_now!() % 300_000 == 0 {
                                            warn!("Account {}: Sync aborted. OAuth2 authorization not completed. Please visit the rustmailer admin page to authorize this account.", account_id);
                                        }
                                        return Ok(());
                                    }
                                    if let Err(e) = execute_outlook_sync(&account).await {
                                        STATUS_DISPATCHER
                                            .append_error(
                                                account_id,
                                                format!("error in account sync task: {:#?}", e),
                                            )
                                            .await;
                                        error!(
                                            "Failed to synchronize mailbox data for '{}': {:?}",
                                            account_id, e
                                        )
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        error!(
                            "Account {}: Sync aborted. Account entity not found.",
                            account_id
                        );
                    }
                }
                Ok(())
            })
        };
        let handler = periodic_task.start(task, Some(account_id), TASK_INTERVAL, true, true);
        self.tasks.insert(account_id, handler);
    }

    pub async fn stop(&self, account_id: u64) -> RustMailerResult<()> {
        if let Some((_, handler)) = self.tasks.remove(&account_id) {
            handler.cancel().await;
        } else {
            warn!("No sync task found for account: {}", account_id);
        }
        Ok(())
    }
}
