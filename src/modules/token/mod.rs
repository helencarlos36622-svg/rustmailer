// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::migration::AccountModel;
use crate::modules::database::delete_impl;
use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{insert_impl, list_all_impl, update_impl};
use crate::modules::token::payload::AccessTokenUpdateRequest;
use crate::raise_error;
use crate::{
    generate_token, modules::error::RustMailerResult,
    modules::token::payload::AccessTokenCreateRequest, utc_now,
};
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::net::IpAddr;
use std::str::FromStr;

use super::error::code::ErrorCode;

pub mod payload;
pub mod root;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Object)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct AccessToken {
    /// The unique token string used for authentication
    #[primary_key]
    pub token: String,
    /// A set of account information associated with the token.
    pub accounts: BTreeSet<AccountInfo>,
    /// The timestamp (in milliseconds since epoch) when the token was created.
    pub created_at: i64,
    /// The timestamp (in milliseconds since epoch) when the token was last updated.
    pub updated_at: i64,
    /// An optional description of the token's purpose or usage.
    pub description: Option<String>,
    /// A set of scopes defining the token's access permissions.
    pub access_scopes: BTreeSet<AccessTokenScope>,
    /// The timestamp (in milliseconds since epoch) when the token was last used.
    pub last_access_at: i64,
    /// Optional access control settings
    pub acl: Option<AccessControl>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize, Object)]
pub struct AccountInfo {
    /// The unique identifier for the account.
    pub id: u64,
    /// The email address associated with the account.
    pub email: String,
}

impl Ord for AccountInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for AccountInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Object)]
pub struct AccessControl {
    /// An optional set of valid IPv4 or IPv6 addresses allowed to use the access token.
    pub ip_whitelist: Option<BTreeSet<String>>,
    /// An optional rate limit configuration for the access token.
    pub rate_limit: Option<RateLimit>,
}

impl AccessControl {
    pub fn validate(&self) -> RustMailerResult<()> {
        if let Some(ip_whitelist) = &self.ip_whitelist {
            for ip in ip_whitelist {
                if ip.parse::<IpAddr>().is_err() {
                    return Err(raise_error!(
                        format!("Invalid IP address: {}", ip),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
        }

        // Validate rate limit
        if let Some(rate_limit) = &self.rate_limit {
            if rate_limit.interval < 1 {
                return Err(raise_error!(
                    "Rate limit interval must be at least 1 second".into(),
                    ErrorCode::InvalidParameter
                ));
            }
            if rate_limit.quota < 1 {
                return Err(raise_error!(
                    "Rate limit quota must be at least 1".into(),
                    ErrorCode::InvalidParameter
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Object)]
pub struct RateLimit {
    /// The time window in seconds for the rate limit.
    pub interval: u64,
    /// The maximum number of allowed requests within the time window.
    pub quota: u32,
}

impl AccessToken {
    pub fn new(
        token: String,
        accounts: BTreeSet<AccountInfo>,
        description: Option<String>,
        access_scopes: BTreeSet<AccessTokenScope>,
        acl: Option<AccessControl>,
    ) -> Self {
        Self {
            token,
            accounts,
            created_at: utc_now!(),
            updated_at: utc_now!(),
            description,
            access_scopes,
            last_access_at: Default::default(),
            acl,
        }
    }

    pub async fn try_update_access_timestamp(token: &str) -> RustMailerResult<AccessToken> {
        let token = token.to_string();
        update_impl(
            DB_MANAGER.meta_db(),
            |rw| {
                rw.get()
                    .primary::<AccessToken>(token)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!("Token not exist.".into(), ErrorCode::ResourceNotFound)
                    })
            },
            |current| {
                let mut updated = current.clone();
                updated.last_access_at = utc_now!();
                Ok(updated)
            },
        )
        .await
    }

    pub async fn grant_account_access(token: &str, account: AccountInfo) -> RustMailerResult<()> {
        let token = token.to_string();
        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .primary::<AccessToken>(token.clone())
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!(
                            format!(
                            "The access token with token={} that you want to modify was not found.",
                            token
                        ),
                            ErrorCode::ResourceNotFound
                        )
                    })
            },
            |current| {
                let mut updated = current.clone();
                updated.accounts.insert(account);
                updated.updated_at = utc_now!();
                Ok(updated)
            },
        )
        .await?;
        Ok(())
    }

    pub async fn update(token: &str, request: AccessTokenUpdateRequest) -> RustMailerResult<()> {
        if request.should_skip_update() {
            return Err(raise_error!(
                "No changes detected in access scopes, description, or accounts. \
                 Please modify at least one of these fields to perform an update."
                    .into(),
                ErrorCode::InvalidParameter
            ));
        }
        request.validate().await?;

        let account_infos = if let Some(accounts) = &request.accounts {
            let mut account_infos = BTreeSet::new();
            for account_id in accounts {
                let account = AccountModel::get(*account_id).await?;
                account_infos.insert(AccountInfo {
                    id: *account_id,
                    email: account.email,
                });
            }
            account_infos
        } else {
            BTreeSet::new()
        };

        let token = token.to_string();
        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .primary::<AccessToken>(token.clone())
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!(
                            format!(
                            "The access token with token={} that you want to modify was not found.",
                            token
                        ),
                            ErrorCode::ResourceNotFound
                        )
                    })
            },
            move |current| {
                let mut updated = current.clone();
                if let Some(access_scopes) = request.access_scopes {
                    updated.access_scopes = access_scopes;
                }

                if let Some(description) = request.description {
                    updated.description = Some(description);
                }

                if request.accounts.is_some() {
                    updated.accounts = account_infos;
                }

                if let Some(acl) = request.acl {
                    updated.acl = Some(acl);
                }

                updated.updated_at = utc_now!();
                Ok(updated)
            },
        )
        .await?;
        Ok(())
    }

    pub async fn create(request: AccessTokenCreateRequest) -> RustMailerResult<String> {
        // Validate request parameters first
        request.validate().await?;

        let AccessTokenCreateRequest {
            accounts,
            description,
            access_scopes,
            acl,
        } = request;

        let mut account_infos = BTreeSet::new();
        for &account_id in &accounts {
            let account = AccountModel::get(account_id).await?;
            account_infos.insert(AccountInfo {
                id: account_id,
                email: account.email,
            });
        }

        let token = generate_token!(128);
        let access_token = AccessToken::new(
            token.clone(),
            account_infos,
            description,
            access_scopes,
            acl,
        );

        insert_impl(DB_MANAGER.meta_db(), access_token).await?;
        Ok(token)
    }

    pub async fn delete(token: &str) -> RustMailerResult<()> {
        let token = token.to_string();
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get()
                .primary::<AccessToken>(token.clone())
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(|| {
                    raise_error!(
                        format!("Token '{}' not found during deletion process.", token),
                        ErrorCode::ResourceNotFound
                    )
                })
        })
        .await
    }

    pub async fn list_all() -> RustMailerResult<Vec<AccessToken>> {
        list_all_impl(DB_MANAGER.meta_db()).await
    }

    pub async fn list_account_tokens(account_id: u64) -> RustMailerResult<Vec<AccessToken>> {
        let all = AccessToken::list_all().await?;
        let result: Vec<AccessToken> = all
            .into_iter()
            .filter(|e| {
                e.accounts
                    .iter()
                    .any(|account_info| account_info.id == account_id)
            })
            .collect();
        Ok(result)
    }

    pub async fn cleanup_account(account_id: u64) -> RustMailerResult<()> {
        let tokens = Self::list_account_tokens(account_id).await?;
        if tokens.is_empty() {
            return Ok(());
        }

        for token in tokens {
            update_impl(
                DB_MANAGER.meta_db(),
                move |rw| {
                    rw.get()
                        .primary::<AccessToken>(token.token.clone())
                        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                        .ok_or_else(|| {
                            raise_error!(
                                format!("Cannot find access token, {}", token.token),
                                ErrorCode::ResourceNotFound
                            )
                        })
                },
                move |current| {
                    let mut updated = current.clone();
                    updated.updated_at = utc_now!();
                    updated.accounts.retain(|account| account.id != account_id);
                    Ok(updated)
                },
            )
            .await?;
        }
        Ok(())
    }

    pub fn can_access_account(&self, account_id: u64) -> bool {
        self.accounts.iter().any(|account| account.id == account_id)
    }
}

/// Defines the scope of access for an access token.
#[derive(Enum, Hash, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Ord, PartialOrd)]
pub enum AccessTokenScope {
    /// Grants access to API-related operations.
    Api,
    /// Grants access to Prometheus metrics endpoints.
    Metrics,
}

impl FromStr for AccessTokenScope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "api" => Ok(AccessTokenScope::Api),
            _ => Err(()),
        }
    }
}
