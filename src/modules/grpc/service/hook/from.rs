// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    grpc::service::rustmailer_grpc::{self},
    hook::{
        entity::{EventHooks, HookType, HttpConfig, HttpMethod},
        events::EventType,
        nats::{NatsAuthType, NatsConfig},
        payload::{EventhookCreateRequest, EventhookUpdateRequest},
        task::SendEventHookTask,
        vrl::payload::{ResolveResult, VrlScriptTestRequest},
    },
    rest::response::DataPage,
    utils::json_value_to_prost_value,
};

impl From<EventHooks> for rustmailer_grpc::EventHooks {
    fn from(value: EventHooks) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            email: value.email,
            description: value.description,
            created_at: value.created_at,
            updated_at: value.updated_at,
            enabled: value.enabled,
            hook_type: value.hook_type.into(),
            http: value.http.map(Into::into),
            nats: value.nats.map(Into::into),
            vrl_script: value.vrl_script,
            call_count: value.call_count,
            success_count: value.success_count,
            failure_count: value.failure_count,
            last_error: value.last_error,
            watched_events: value.watched_events.into_iter().map(|e| e.into()).collect(),
            global: value.global as u32,
        }
    }
}

impl From<HookType> for i32 {
    fn from(value: HookType) -> Self {
        match value {
            HookType::Http => 0,
            HookType::Nats => 1,
        }
    }
}

impl From<HttpConfig> for rustmailer_grpc::HttpConfig {
    fn from(value: HttpConfig) -> Self {
        Self {
            target_url: value.target_url,
            http_method: value.http_method.into(),
            custom_headers: value.custom_headers.into_iter().collect(),
        }
    }
}

impl From<HttpMethod> for i32 {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Post => 0,
            HttpMethod::Put => 1,
        }
    }
}

impl From<NatsAuthType> for i32 {
    fn from(value: NatsAuthType) -> Self {
        match value {
            NatsAuthType::None => 0,
            NatsAuthType::Password => 1,
            NatsAuthType::Token => 2,
        }
    }
}

impl From<NatsConfig> for rustmailer_grpc::NatsConfig {
    fn from(value: NatsConfig) -> Self {
        Self {
            host: value.host,
            port: value.port as u32,
            auth_type: value.auth_type.into(),
            token: value.token,
            username: value.username,
            password: value.password,
            stream_name: value.stream_name,
            namespace: value.namespace,
        }
    }
}

impl From<EventType> for i32 {
    fn from(value: EventType) -> Self {
        match value {
            EventType::EmailAddedToFolder => 0,
            EventType::EmailFlagsChanged => 1,
            EventType::EmailSentSuccess => 2,
            EventType::EmailSendingError => 3,
            EventType::UIDValidityChange => 4,
            EventType::MailboxDeletion => 5,
            EventType::MailboxCreation => 6,
            EventType::AccountFirstSyncCompleted => 7,
            EventType::EmailBounce => 8,
            EventType::EmailFeedBackReport => 9,
            EventType::EmailOpened => 10,
            EventType::EmailLinkClicked => 11,
        }
    }
}

impl TryFrom<rustmailer_grpc::CreateEventHookRequest> for EventhookCreateRequest {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::CreateEventHookRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: value.account_id,
            description: value.description,
            enabled: value.enabled,
            hook_type: value.hook_type.try_into()?,
            http: value.http.map(HttpConfig::try_from).transpose()?,
            nats: value.nats.map(NatsConfig::try_from).transpose()?,
            vrl_script: value.vrl_script,
            watched_events: value
                .watched_events
                .into_iter()
                .map(EventType::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            use_proxy: value.use_proxy,
        })
    }
}

impl TryFrom<i32> for HookType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HookType::Http),
            1 => Ok(HookType::Nats),
            _ => Err("Invalid value for HookType"),
        }
    }
}

impl TryFrom<rustmailer_grpc::HttpConfig> for HttpConfig {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::HttpConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            target_url: value.target_url,
            http_method: value.http_method.try_into()?,
            custom_headers: value.custom_headers.into_iter().collect(),
        })
    }
}

impl TryFrom<i32> for HttpMethod {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HttpMethod::Post),
            1 => Ok(HttpMethod::Put),
            _ => Err("Invalid value for HttpMethod"),
        }
    }
}

impl TryFrom<rustmailer_grpc::NatsConfig> for NatsConfig {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::NatsConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            host: value.host,
            port: value.port as u16,
            auth_type: value.auth_type.try_into()?,
            token: value.token,
            username: value.username,
            password: value.password,
            stream_name: value.stream_name,
            namespace: value.namespace,
        })
    }
}

impl TryFrom<i32> for NatsAuthType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NatsAuthType::None),
            1 => Ok(NatsAuthType::Password),
            2 => Ok(NatsAuthType::Token),
            _ => Err("Invalid value for NatsAuthType"),
        }
    }
}

impl TryFrom<i32> for EventType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventType::EmailAddedToFolder),
            1 => Ok(EventType::EmailFlagsChanged),
            2 => Ok(EventType::EmailSentSuccess),
            3 => Ok(EventType::EmailSendingError),
            4 => Ok(EventType::UIDValidityChange),
            5 => Ok(EventType::MailboxDeletion),
            6 => Ok(EventType::MailboxCreation),
            7 => Ok(EventType::AccountFirstSyncCompleted),
            8 => Ok(EventType::EmailBounce),
            9 => Ok(EventType::EmailFeedBackReport),
            10 => Ok(EventType::EmailOpened),
            11 => Ok(EventType::EmailLinkClicked),
            _ => Err("Invalid value for EventType"),
        }
    }
}

impl TryFrom<rustmailer_grpc::UpdateEventhookRequest> for EventhookUpdateRequest {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::UpdateEventhookRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            description: value.description,
            enabled: value.enabled,
            http: value.http.map(HttpConfig::try_from).transpose()?,
            nats: value.nats.map(NatsConfig::try_from).transpose()?,
            vrl_script: value.vrl_script,
            watched_events: {
                if value.watched_events.is_empty() {
                    None
                } else {
                    Some(
                        value
                            .watched_events
                            .into_iter()
                            .map(EventType::try_from)
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                }
            },
            use_proxy: value.use_proxy,
        })
    }
}

impl From<DataPage<EventHooks>> for rustmailer_grpc::PagedEventHooks {
    fn from(value: DataPage<EventHooks>) -> Self {
        Self {
            current_page: value.current_page,
            page_size: value.page_size,
            total_items: value.total_items,
            items: value.items.into_iter().map(Into::into).collect(),
            total_pages: value.total_pages,
        }
    }
}

impl From<DataPage<SendEventHookTask>> for rustmailer_grpc::PagedEventHookTask {
    fn from(value: DataPage<SendEventHookTask>) -> Self {
        Self {
            current_page: value.current_page,
            page_size: value.page_size,
            total_items: value.total_items,
            items: value.items.into_iter().map(Into::into).collect(),
            total_pages: value.total_pages,
        }
    }
}

impl From<SendEventHookTask> for rustmailer_grpc::EventHookTask {
    fn from(value: SendEventHookTask) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            status: value.status.into(),
            stopped_reason: value.stopped_reason,
            error: value.error,
            last_duration_ms: value.last_duration_ms.map(|s| s as u32),
            retry_count: value.retry_count.map(|c| c as u32),
            scheduled_at: value.scheduled_at,
            account_id: value.account_id,
            event: Some(json_value_to_prost_value(value.event)),
            event_type: value.event_type.into(),
        }
    }
}

impl From<rustmailer_grpc::VrlScriptTestRequest> for VrlScriptTestRequest {
    fn from(value: rustmailer_grpc::VrlScriptTestRequest) -> Self {
        Self {
            program: value.program,
            event: value.event,
        }
    }
}

impl From<ResolveResult> for rustmailer_grpc::ResolveResult {
    fn from(value: ResolveResult) -> Self {
        Self {
            result: value.result.map(json_value_to_prost_value),
            error: value.error,
        }
    }
}
