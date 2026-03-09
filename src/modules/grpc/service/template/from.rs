// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    grpc::service::rustmailer_grpc::{self},
    rest::response::DataPage,
    smtp::template::{
        entity::{EmailTemplate, MessageFormat},
        payload::{TemplateCreateRequest, TemplateSentTestRequest, TemplateUpdateRequest},
    },
    token::AccountInfo,
    utils::prost_value_to_json_value,
};

impl From<AccountInfo> for rustmailer_grpc::AccountInfo {
    fn from(account: AccountInfo) -> Self {
        Self {
            id: account.id,
            email: account.email,
        }
    }
}

impl From<rustmailer_grpc::AccountInfo> for AccountInfo {
    fn from(account: rustmailer_grpc::AccountInfo) -> Self {
        Self {
            id: account.id,
            email: account.email,
        }
    }
}

impl From<EmailTemplate> for rustmailer_grpc::EmailTemplate {
    fn from(value: EmailTemplate) -> Self {
        Self {
            id: value.id,
            description: value.description,
            account: value.account.map(Into::into),
            subject: value.subject,
            preview: value.preview,
            format: value.format.map(Into::into),
            text: value.text,
            html: value.html,
            created_at: value.created_at,
            updated_at: value.updated_at,
            last_access_at: value.last_access_at,
        }
    }
}

impl From<MessageFormat> for i32 {
    fn from(value: MessageFormat) -> Self {
        match value {
            MessageFormat::Markdown => 1,
            MessageFormat::Html => 0,
        }
    }
}

impl TryFrom<rustmailer_grpc::EmailTemplateCreateRequest> for TemplateCreateRequest {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::EmailTemplateCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            description: value.description,
            account_id: value.account_id,
            subject: value.subject,
            preview: value.preview,
            text: value.text,
            html: value.html,
            format: value.format.map(MessageFormat::try_from).transpose()?,
        })
    }
}

impl TryFrom<i32> for MessageFormat {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageFormat::Html),
            1 => Ok(MessageFormat::Markdown),
            _ => Err("Invalid value for MessageFormat"),
        }
    }
}

impl TryFrom<rustmailer_grpc::UpdateTemplateRequest> for TemplateUpdateRequest {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::UpdateTemplateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            description: value.description,
            subject: value.subject,
            preview: value.preview,
            text: value.text,
            html: value.html,
            format: value.format.map(MessageFormat::try_from).transpose()?,
        })
    }
}

impl From<DataPage<EmailTemplate>> for rustmailer_grpc::PagedEmailTemplate {
    fn from(value: DataPage<EmailTemplate>) -> Self {
        Self {
            current_page: value.current_page,
            page_size: value.page_size,
            total_items: value.total_items,
            items: value.items.into_iter().map(Into::into).collect(),
            total_pages: value.total_pages,
        }
    }
}

impl From<rustmailer_grpc::TemplateSentTestRequest> for TemplateSentTestRequest {
    fn from(value: rustmailer_grpc::TemplateSentTestRequest) -> Self {
        Self {
            account_id: value.account_id,
            recipient: value.recipient,
            template_params: value.template_params.map(prost_value_to_json_value),
        }
    }
}
