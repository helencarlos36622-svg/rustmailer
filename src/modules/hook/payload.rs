// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::hook::entity::HookType;
use crate::modules::hook::events::EventType;
use crate::modules::hook::{entity::HttpConfig, nats::NatsConfig};
use crate::{modules::hook::entity::EventHooks, utc_now};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct EventhookCreateRequest {
    /// Unique identifier of the account associated with the hook.
    pub account_id: Option<u64>,
    /// Optional description providing additional context about the hook.
    pub description: Option<String>,
    /// Indicates whether the hook is active and processing events upon creation.
    pub enabled: bool,
    /// The type of hook (e.g., HTTP or NATS).
    pub hook_type: HookType,
    /// Optional HTTP configuration for HTTP-based hook.
    pub http: Option<HttpConfig>,
    /// Optional NATS configuration for NATS-based hook.
    pub nats: Option<NatsConfig>,
    /// Optional VRL (Vector Remap Language) script for customizing the hook payload.
    pub vrl_script: Option<String>,
    /// List of event types the hook is configured to monitor.
    pub watched_events: Vec<EventType>,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the webhook server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct EventhookUpdateRequest {
    /// Optional description providing additional context about the hook.
    pub description: Option<String>,
    /// Indicates whether the hook is active and processing events upon creation.
    pub enabled: Option<bool>,
    /// Optional HTTP configuration for HTTP-based hook.
    pub http: Option<HttpConfig>,
    /// Optional NATS configuration for NATS-based hook.
    pub nats: Option<NatsConfig>,
    /// Optional VRL (Vector Remap Language) script for customizing the hook payload.
    pub vrl_script: Option<String>,
    /// List of event types the hook is configured to monitor.
    pub watched_events: Option<Vec<EventType>>,
    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the webhook server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InternalEventHookUpdateRequest {
    pub increase_call_count: Option<bool>,
    pub increase_success_count: Option<bool>,
    pub increase_failure_count: Option<bool>,
    pub last_error: Option<String>,
}

pub fn apply_update(old: &EventHooks, request: EventhookUpdateRequest) -> EventHooks {
    let mut new = old.clone();

    if request.description.is_some() {
        new.description = request.description;
    }

    if let Some(enabled) = request.enabled {
        new.enabled = enabled;
    }

    if let Some(http) = request.http {
        new.http = Some(http);
    }

    if let Some(nats) = request.nats {
        new.nats = Some(nats);
    }

    if let Some(vrl_script) = request.vrl_script {
        new.vrl_script = Some(vrl_script);
    }

    if let Some(use_proxy) = request.use_proxy {
        new.use_proxy = Some(use_proxy)
    }

    if let Some(watched_events) = request.watched_events {
        new.watched_events = watched_events;
    }

    new.updated_at = utc_now!();

    new
}

pub fn apply_internal_update(
    old: &EventHooks,
    request: InternalEventHookUpdateRequest,
) -> EventHooks {
    let mut new = old.clone();
    if let Some(true) = request.increase_call_count {
        new.call_count += 1;
    }
    if let Some(true) = request.increase_success_count {
        new.success_count += 1;
    }
    if let Some(true) = request.increase_failure_count {
        new.failure_count += 1;
    }
    if request.last_error.is_some() {
        new.last_error = request.last_error;
    }
    new.updated_at = utc_now!();
    new
}
