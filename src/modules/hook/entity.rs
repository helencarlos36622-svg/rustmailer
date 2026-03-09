// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::id;
use crate::modules::account::migration::AccountModel;
use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{
    delete_impl, filter_by_secondary_key_impl, paginate_query_primary_scan_all_impl,
    secondary_find_impl, update_impl,
};
use crate::modules::error::code::ErrorCode;
use crate::modules::hook::events::EventType;
use crate::modules::hook::nats::NatsConfig;
use crate::modules::hook::payload::apply_update;
use crate::modules::hook::payload::{EventhookCreateRequest, EventhookUpdateRequest};
use crate::modules::hook::vrl::compile_vrl_script;
use crate::modules::rest::response::DataPage;
use crate::{
    modules::database::insert_impl, modules::error::RustMailerResult, raise_error, utc_now,
};
use http::{HeaderName, HeaderValue};
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::types::Type;
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use url::Url;

use crate::modules::hook::payload::{apply_internal_update, InternalEventHookUpdateRequest};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Enum)]
pub enum HttpMethod {
    #[default]
    Post,
    Put,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Enum)]
pub enum HookType {
    ///using HTTP protocol for sending payloads
    #[default]
    Http,
    ///using NATS messaging system for event delivery
    Nats,
}

impl HookType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookType::Http => "http",
            HookType::Nats => "nats",
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Post => write!(f, "Post"),
            HttpMethod::Put => write!(f, "Put"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct HttpConfig {
    /// The target URL where the webhook payload is sent.
    pub target_url: String,
    /// The HTTP method used to send the webhook payload.
    pub http_method: HttpMethod,
    /// Custom headers included in the webhook request, stored as key-value pairs.
    pub custom_headers: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
#[native_model(id = 11, version = 1)]
#[native_db(primary_key(pk -> String))]
pub struct EventHooks {
    /// The unique identifier of the event hook
    #[secondary_key(unique)]
    pub id: u64,
    /// Unique identifier of the account associated with the hook.
    #[secondary_key(unique, optional)]
    pub account_id: Option<u64>,
    /// Email address of the account associated with the hook.
    pub email: Option<String>,
    /// Optional description providing additional context about the hook.
    pub description: Option<String>,
    /// Timestamp (in milliseconds) when the hook was created.
    pub created_at: i64,
    /// Timestamp (in milliseconds) when the hook was last updated.
    pub updated_at: i64,
    /// Indicates whether the hook is global and applies to all accounts. 1: true, 0: false
    #[secondary_key]
    pub global: u8,
    /// Indicates whether the hook is currently active and processing events.
    pub enabled: bool,
    /// The type of hook (e.g., HTTP or NATS).
    pub hook_type: HookType,
    /// Optional HTTP configuration for HTTP-based hook.
    pub http: Option<HttpConfig>,
    /// Optional NATS configuration for NATS-based hook.
    pub nats: Option<NatsConfig>,
    /// Optional VRL (Vector Remap Language) script for customizing the hook payload.
    pub vrl_script: Option<String>,
    /// Total number of times the hook has been triggered.
    pub call_count: u64,
    /// Number of times the hook has been successfully executed.
    pub success_count: u64,
    /// Number of times the hook execution has failed.
    pub failure_count: u64,
    /// Details of the last error encountered during hook execution, if any.
    pub last_error: Option<String>,
    /// List of event types the hook is configured to monitor.
    pub watched_events: Vec<EventType>,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the webhook server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

impl EventHooks {
    fn pk(&self) -> String {
        format!("{}_{}", self.created_at, self.id)
    }

    pub async fn new(request: EventhookCreateRequest) -> RustMailerResult<Self> {
        let (email, global) = if let Some(account_id) = request.account_id {
            (Some(AccountModel::get(account_id).await?.email), 0)
        } else {
            (None, 1)
        };
        Ok(Self {
            id: id!(64),
            account_id: request.account_id,
            email,
            description: request.description,
            created_at: utc_now!(),
            updated_at: utc_now!(),
            global,
            enabled: request.enabled,
            hook_type: request.hook_type,
            http: request.http,
            nats: request.nats,
            vrl_script: request.vrl_script,
            call_count: 0,
            success_count: 0,
            failure_count: 0,
            last_error: None,
            watched_events: request.watched_events,
            use_proxy: request.use_proxy,
        })
    }

    pub async fn paginate_list(
        page: Option<u64>,
        page_size: Option<u64>,
        desc: Option<bool>,
    ) -> RustMailerResult<DataPage<EventHooks>> {
        paginate_query_primary_scan_all_impl(DB_MANAGER.meta_db(), page, page_size, desc)
            .await
            .map(DataPage::from)
    }

    /// Save the current Webhook entity to the database
    pub async fn save(self) -> RustMailerResult<()> {
        self.validate().await?;
        insert_impl(DB_MANAGER.meta_db(), self).await
    }

    /// Get a specific Webhook entity by its ID
    pub async fn get_by_id(id: u64) -> RustMailerResult<Option<EventHooks>> {
        secondary_find_impl(DB_MANAGER.meta_db(), EventHooksKey::id, id).await
    }
    /// Get a specific Webhook entity by its account id
    pub async fn get_by_account_id(account_id: u64) -> RustMailerResult<Option<EventHooks>> {
        secondary_find_impl(
            DB_MANAGER.meta_db(),
            EventHooksKey::account_id,
            Some(account_id),
        )
        .await
    }

    pub async fn global_hooks() -> RustMailerResult<Vec<EventHooks>> {
        filter_by_secondary_key_impl(DB_MANAGER.meta_db(), EventHooksKey::global, 1u8).await
    }

    /// Delete a specific Webhook entity by its ID
    pub async fn delete(id: u64) -> RustMailerResult<()> {
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get()
                .secondary::<EventHooks>(EventHooksKey::id, id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(move || {
                    raise_error!(
                        format!(
                            "The event hook with id={id} that you want to delete was not found."
                        ),
                        ErrorCode::ResourceNotFound
                    )
                })
        })
        .await
    }

    pub async fn try_delete(account_id: u64) -> RustMailerResult<()> {
        if Self::get_by_account_id(account_id).await?.is_none() {
            return Ok(());
        }
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get()
                .secondary::<EventHooks>(EventHooksKey::account_id, Some(account_id))
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(|| {
                    raise_error!(
                        format!(
                            "The event hook with id={account_id} that you want to delete was not found."
                        ),
                        ErrorCode::ResourceNotFound
                    )
                })
        })
        .await?;
        Ok(())
    }

    pub async fn update(id: u64, request: EventhookUpdateRequest) -> RustMailerResult<()> {
        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .secondary::<EventHooks>(EventHooksKey::id, id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {raise_error!(format!("The event hook entity with id={} that you want to modify was not found.",id), ErrorCode::ResourceNotFound)})
            },
            |current| Ok(apply_update(current, request)),
        )
        .await?;
        Ok(())
    }

    pub async fn internal_update(
        id: u64,
        request: InternalEventHookUpdateRequest,
    ) -> RustMailerResult<()> {
        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .secondary::<EventHooks>(EventHooksKey::id, id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {raise_error!(format!("The event hook entity with id={} that you want to modify was not found.",id), ErrorCode::ResourceNotFound)})
            },
            |current| Ok(apply_internal_update(current, request)),
        )
        .await?;
        Ok(())
    }

    async fn validate(&self) -> RustMailerResult<()> {
        if let Some(account_id) = self.account_id {
            if AccountModel::get(account_id).await?.is_none() {
                return Err(raise_error!(
                    format!("Account with id '{}' not exists", account_id),
                    ErrorCode::InvalidParameter
                ));
            }

            if Self::get_by_account_id(account_id).await?.is_some() {
                return Err(raise_error!(
                    "Account already has an EventHook".into(),
                    ErrorCode::AlreadyExists
                ));
            }
        }

        match &self.hook_type {
            HookType::Http => {
                if self.http.is_none() {
                    return Err(raise_error!(
                        "when event hook type is `Http`, field `http` must be configured".into(),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
            HookType::Nats => {
                if self.nats.is_none() {
                    return Err(raise_error!(
                        "when event hook type is `Nats`, field `nats` must be configured".into(),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
        }

        if self.http.is_some() && self.nats.is_some() {
            return Err(raise_error!(
                "Do not configure both http and nats".into(),
                ErrorCode::InvalidParameter
            ));
        }

        if let Some(http) = &self.http {
            if let Err(e) = Url::parse(&http.target_url) {
                return Err(raise_error!(
                    format!("{:#?}", e),
                    ErrorCode::InvalidParameter
                ));
            }

            for (key, value) in &http.custom_headers {
                if HeaderName::from_bytes(key.as_bytes()).is_err() {
                    return Err(raise_error!(
                        format!("Invalid header name: {}", key),
                        ErrorCode::InvalidParameter
                    ));
                }

                if HeaderValue::from_str(value).is_err() {
                    return Err(raise_error!(
                        format!("Invalid header value: {}", value),
                        ErrorCode::InvalidParameter
                    ));
                }
            }
        }

        if let Some(nats) = &self.nats {
            nats.validate()?;
        }

        if self.watched_events.is_empty() {
            return Err(raise_error!(
                "Please select at least one event to watch".into(),
                ErrorCode::InvalidParameter
            ));
        }

        if let Some(vrl_script) = &self.vrl_script {
            compile_vrl_script(vrl_script)?;
        }
        Ok(())
    }
}
