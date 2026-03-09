// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    account::{
        entity::{AuthConfig, AuthType, Encryption, ImapConfig, MailerType, SmtpConfig},
        migration::AccountModel,
        payload::{AccountCreateRequest, AccountUpdateRequest, MinimalAccount},
        since::{DateSince, RelativeDate, Unit},
        status::{AccountError, AccountRunningState},
    },
    grpc::service::rustmailer_grpc,
};

impl TryFrom<i32> for Encryption {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Encryption::Ssl),
            1 => Ok(Encryption::StartTls),
            2 => Ok(Encryption::None),
            _ => Err("Invalid value for Encryption"),
        }
    }
}

impl From<Encryption> for i32 {
    fn from(value: Encryption) -> Self {
        match value {
            Encryption::Ssl => 0,
            Encryption::StartTls => 1,
            Encryption::None => 2,
        }
    }
}

impl From<rustmailer_grpc::AuthType> for AuthType {
    fn from(value: rustmailer_grpc::AuthType) -> Self {
        match value {
            rustmailer_grpc::AuthType::Password => AuthType::Password,
            rustmailer_grpc::AuthType::Oauth2 => AuthType::OAuth2,
        }
    }
}

impl From<AuthType> for i32 {
    fn from(value: AuthType) -> Self {
        match value {
            AuthType::Password => 0,
            AuthType::OAuth2 => 1,
        }
    }
}

impl TryFrom<i32> for AuthType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AuthType::Password),
            1 => Ok(AuthType::OAuth2),
            _ => Err("Invalid value for AuthType"),
        }
    }
}

impl TryFrom<rustmailer_grpc::AuthConfig> for AuthConfig {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::AuthConfig) -> Result<Self, Self::Error> {
        Ok(AuthConfig {
            auth_type: AuthType::try_from(value.auth_type)?,
            password: value.password,
        })
    }
}

impl From<AuthConfig> for rustmailer_grpc::AuthConfig {
    fn from(value: AuthConfig) -> Self {
        rustmailer_grpc::AuthConfig {
            auth_type: value.auth_type as i32,
            password: value.password,
        }
    }
}

impl TryFrom<rustmailer_grpc::ImapConfig> for ImapConfig {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::ImapConfig) -> Result<Self, Self::Error> {
        Ok(ImapConfig {
            host: value.host,
            port: value.port as u16,
            encryption: Encryption::try_from(value.encryption)?,
            auth: AuthConfig::try_from(
                value
                    .auth
                    .ok_or("AuthConfig is not set in ImapConfig, which is required")?,
            )?,
            use_proxy: value.use_proxy,
        })
    }
}

impl From<ImapConfig> for rustmailer_grpc::ImapConfig {
    fn from(value: ImapConfig) -> Self {
        rustmailer_grpc::ImapConfig {
            host: value.host,
            port: value.port as u32,
            encryption: value.encryption as i32,
            auth: Some(value.auth.into()),
            use_proxy: value.use_proxy,
        }
    }
}

impl TryFrom<rustmailer_grpc::SmtpConfig> for SmtpConfig {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::SmtpConfig) -> Result<Self, Self::Error> {
        Ok(SmtpConfig {
            host: value.host,
            port: value.port as u16,
            encryption: Encryption::try_from(value.encryption)?,
            auth: AuthConfig::try_from(
                value
                    .auth
                    .ok_or("AuthConfig is not set in SmtpConfig, which is required")?,
            )?,
            use_proxy: value.use_proxy,
        })
    }
}

impl From<SmtpConfig> for rustmailer_grpc::SmtpConfig {
    fn from(value: SmtpConfig) -> Self {
        rustmailer_grpc::SmtpConfig {
            host: value.host,
            port: value.port as u32,
            encryption: value.encryption as i32,
            auth: Some(value.auth.into()),
            use_proxy: value.use_proxy,
        }
    }
}

impl TryFrom<i32> for Unit {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Unit::Days),
            1 => Ok(Unit::Months),
            2 => Ok(Unit::Years),
            _ => Err("Invalid value for Unit"),
        }
    }
}

impl From<Unit> for i32 {
    fn from(value: Unit) -> Self {
        match value {
            Unit::Days => 0,
            Unit::Months => 1,
            Unit::Years => 2,
        }
    }
}

impl TryFrom<rustmailer_grpc::RelativeDate> for RelativeDate {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::RelativeDate) -> Result<Self, Self::Error> {
        Ok(RelativeDate {
            unit: Unit::try_from(value.unit)?,
            value: value.value,
        })
    }
}

impl From<RelativeDate> for rustmailer_grpc::RelativeDate {
    fn from(value: RelativeDate) -> Self {
        rustmailer_grpc::RelativeDate {
            unit: value.unit as i32,
            value: value.value,
        }
    }
}

impl TryFrom<rustmailer_grpc::DateSince> for DateSince {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::DateSince) -> Result<Self, Self::Error> {
        Ok(DateSince {
            fixed: value.fixed,
            relative: value.relative.map(RelativeDate::try_from).transpose()?,
        })
    }
}

impl From<DateSince> for rustmailer_grpc::DateSince {
    fn from(value: DateSince) -> Self {
        rustmailer_grpc::DateSince {
            fixed: value.fixed,
            relative: value.relative.map(Into::into),
        }
    }
}

impl TryFrom<rustmailer_grpc::Account> for AccountModel {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::Account) -> Result<Self, Self::Error> {
        Ok(AccountModel {
            id: value.id,
            imap: value.imap.map(|imap| imap.try_into()).transpose()?,
            smtp: value.smtp.map(|smtp| smtp.try_into()).transpose()?,
            enabled: value.enabled,
            mailer_type: value.mailer_type.try_into()?,
            email: value.email,
            name: value.name,
            minimal_sync: value.minimal_sync,
            capabilities: if value.capabilities.is_empty() {
                None
            } else {
                Some(value.capabilities)
            },
            dsn_capable: value.dsn_capable,
            date_since: value.date_since.map(|ds| ds.try_into()).transpose()?,
            sync_folders: value.sync_folders,
            known_folders: value.known_folders.into_iter().collect(),
            full_sync_interval_min: value.full_sync_interval_min,
            incremental_sync_interval_sec: value.incremental_sync_interval_sec,
            created_at: value.created_at,
            updated_at: value.updated_at,
            use_proxy: value.use_proxy,
            folder_limit: value.folder_limit,
        })
    }
}

impl From<AccountModel> for rustmailer_grpc::Account {
    fn from(value: AccountModel) -> Self {
        rustmailer_grpc::Account {
            id: value.id,
            imap: value.imap.map(|imap| imap.into()),
            smtp: value.smtp.map(|smtp| smtp.into()),
            enabled: value.enabled,
            mailer_type: value.mailer_type.into(),
            email: value.email,
            name: value.name,
            minimal_sync: value.minimal_sync,
            capabilities: value.capabilities.unwrap_or_default(),
            dsn_capable: value.dsn_capable,
            date_since: value.date_since.map(Into::into),
            sync_folders: value.sync_folders,
            known_folders: value.known_folders.into_iter().collect(),
            full_sync_interval_min: value.full_sync_interval_min,
            incremental_sync_interval_sec: value.incremental_sync_interval_sec,
            created_at: value.created_at,
            updated_at: value.updated_at,
            use_proxy: value.use_proxy,
            folder_limit: value.folder_limit,
        }
    }
}

impl TryFrom<rustmailer_grpc::AccountCreateRequest> for AccountCreateRequest {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::AccountCreateRequest) -> Result<Self, Self::Error> {
        Ok(AccountCreateRequest {
            email: value.email,
            name: value.name,
            imap: value.imap.map(|imap| imap.try_into()).transpose()?,
            smtp: value.smtp.map(|smtp| smtp.try_into()).transpose()?,
            enabled: value.enabled,
            mailer_type: value.mailer_type.try_into()?,
            date_since: value.date_since.map(|ds| ds.try_into()).transpose()?,
            minimal_sync: value.minimal_sync,
            full_sync_interval_min: value.full_sync_interval_min,
            incremental_sync_interval_sec: value.incremental_sync_interval_sec,
            use_proxy: value.use_proxy,
            folder_limit: value.folder_limit,
        })
    }
}

impl TryFrom<rustmailer_grpc::AccountUpdateRequest> for AccountUpdateRequest {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::AccountUpdateRequest) -> Result<Self, Self::Error> {
        Ok(AccountUpdateRequest {
            enabled: value.enabled,
            name: value.name,
            date_since: value.date_since.map(|ds| ds.try_into()).transpose()?,
            sync_folders: value.sync_folders.is_empty().then_some(value.sync_folders),
            full_sync_interval_min: value.full_sync_interval_min,
            incremental_sync_interval_sec: value.incremental_sync_interval_sec,
            imap: value.imap.map(|imap| imap.try_into()).transpose()?,
            smtp: value.smtp.map(|smtp| smtp.try_into()).transpose()?,
            use_proxy: value.use_proxy,
            folder_limit: value.folder_limit,
        })
    }
}

impl From<AccountRunningState> for rustmailer_grpc::AccountRunningState {
    fn from(value: AccountRunningState) -> Self {
        Self {
            account_id: value.account_id,
            last_full_sync_start: value.last_full_sync_start,
            last_full_sync_end: value.last_full_sync_end,
            last_incremental_sync_start: value.last_incremental_sync_start,
            last_incremental_sync_end: value.last_incremental_sync_end,
            errors: value.errors.into_iter().map(Into::into).collect(),
            is_initial_sync_completed: value.is_initial_sync_completed,
            initial_sync_folders: value.initial_sync_folders,
            current_syncing_folder: value.current_syncing_folder,
            current_batch_number: value.current_batch_number,
            current_total_batches: value.current_total_batches,
            initial_sync_start_time: value.initial_sync_start_time,
            initial_sync_end_time: value.initial_sync_end_time,
        }
    }
}

impl From<AccountError> for rustmailer_grpc::AccountError {
    fn from(value: AccountError) -> Self {
        Self {
            error: value.error,
            at: value.at,
        }
    }
}

impl From<MinimalAccount> for rustmailer_grpc::MinimalAccount {
    fn from(value: MinimalAccount) -> Self {
        Self {
            id: value.id,
            email: value.email,
            mailer_type: value.mailer_type.into(),
        }
    }
}

impl TryFrom<i32> for MailerType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MailerType::ImapSmtp),
            1 => Ok(MailerType::GmailApi),
            2 => Ok(MailerType::GraphApi),
            _ => Err("Invalid value for Unit"),
        }
    }
}

impl From<MailerType> for i32 {
    fn from(value: MailerType) -> Self {
        match value {
            MailerType::ImapSmtp => 0,
            MailerType::GmailApi => 1,
            MailerType::GraphApi => 2,
        }
    }
}
