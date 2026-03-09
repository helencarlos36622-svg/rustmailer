// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::context::Initialize;
use crate::modules::error::code::ErrorCode;
use crate::raise_error;
use crate::{
    modules::{
        account::migration::AccountModel,
        context::controller::SYNC_CONTROLLER,
        error::RustMailerResult,
        imap::{executor::ImapExecutor, pool::build_imap_pool},
        smtp::{executor::SmtpExecutor, manager::SmtpServerType, pool::build_smtp_pool},
    },
    utc_now,
};
use dashmap::DashMap;
use std::sync::{Arc, LazyLock};
use tracing::info;

pub static RUST_MAIL_CONTEXT: LazyLock<EmailClientExecutors> =
    LazyLock::new(EmailClientExecutors::new);

pub struct EmailClientExecutors {
    start_at: i64,
    imap: DashMap<u64, Arc<ImapExecutor>>,
    smtp: DashMap<u64, Arc<SmtpExecutor>>,
}

impl Initialize for EmailClientExecutors {
    async fn initialize() -> RustMailerResult<()> {
        RUST_MAIL_CONTEXT.start_account_syncers().await
    }
}

impl EmailClientExecutors {
    pub fn new() -> Self {
        Self {
            start_at: utc_now!(),
            imap: DashMap::new(),
            smtp: DashMap::new(),
        }
    }
    pub fn uptime_ms(&self) -> i64 {
        utc_now!() - self.start_at
    }

    pub async fn imap(&self, account_id: u64) -> RustMailerResult<Arc<ImapExecutor>> {
        if let Some(executor) = self.imap.get(&account_id) {
            return Ok(executor.value().clone());
        }

        let pool = build_imap_pool(account_id).await?;
        let new_executor = Arc::new(ImapExecutor::new(pool));

        match self.imap.try_entry(account_id) {
            Some(dashmap::mapref::entry::Entry::Occupied(entry)) => Ok(entry.get().clone()),
            Some(dashmap::mapref::entry::Entry::Vacant(entry)) => {
                entry.insert(new_executor.clone());
                Ok(new_executor)
            }
            None => Err(raise_error!(
                "DashMap locked".into(),
                ErrorCode::InternalError
            )),
        }
    }

    pub async fn smtp(&self, account_id: u64) -> RustMailerResult<Arc<SmtpExecutor>> {
        self.get_or_create_smtp_executor(account_id, SmtpServerType::Account(account_id))
            .await
    }

    pub async fn mta(&self, mta_id: u64) -> RustMailerResult<Arc<SmtpExecutor>> {
        self.get_or_create_smtp_executor(mta_id, SmtpServerType::Mta(mta_id))
            .await
    }

    pub async fn clean_account(&self, account_id: u64) -> RustMailerResult<()> {
        if self.imap.remove(&account_id).is_some() {
            info!(account_id, "Closed IMAP pool for account");
        }

        if self.smtp.remove(&account_id).is_some() {
            info!(account_id, "Closed SMTP pool for account");
        }

        Ok(())
    }

    pub async fn get_or_create_smtp_executor(
        &self,
        key: u64,
        server_type: SmtpServerType,
    ) -> RustMailerResult<Arc<SmtpExecutor>> {
        if let Some(executor) = self.smtp.get(&key) {
            return Ok(executor.value().clone());
        }

        let pool = build_smtp_pool(server_type).await?;
        let executor = Arc::new(SmtpExecutor::new(pool));

        match self.smtp.try_entry(key) {
            Some(dashmap::mapref::entry::Entry::Occupied(entry)) => Ok(entry.get().clone()),
            Some(dashmap::mapref::entry::Entry::Vacant(entry)) => {
                entry.insert(executor.clone());
                Ok(executor)
            }
            None => Err(raise_error!(
                "DashMap locked".into(),
                ErrorCode::InternalError
            )),
        }
    }

    pub async fn start_account_syncers(&self) -> RustMailerResult<()> {
        let accounts = AccountModel::list_all().await?;
        let active_accounts: Vec<AccountModel> = accounts.into_iter().filter(|a| a.enabled).collect();

        if active_accounts.is_empty() {
            info!("No active accounts found for account initialization.");
            return Ok(());
        }
        info!(
            "System has {} active accounts to initialize.",
            active_accounts.len()
        );
        for account in active_accounts {
            SYNC_CONTROLLER
                .trigger_start(account.id, account.email)
                .await
        }

        Ok(())
    }
}
