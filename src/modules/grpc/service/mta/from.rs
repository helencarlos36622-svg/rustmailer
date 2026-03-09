// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::{
    grpc::service::rustmailer_grpc::{self, PagedMta},
    rest::response::DataPage,
    smtp::mta::{
        entity::{MTACredentials, Mta, SmtpServerConfig},
        payload::{MTACreateRequest, MTAUpdateRequest, SendTestEmailRequest},
    },
};

// MTACredentials
impl From<rustmailer_grpc::MtaCredentials> for MTACredentials {
    fn from(value: rustmailer_grpc::MtaCredentials) -> Self {
        MTACredentials {
            username: value.username,
            password: value.password,
        }
    }
}

impl From<MTACredentials> for rustmailer_grpc::MtaCredentials {
    fn from(value: MTACredentials) -> Self {
        rustmailer_grpc::MtaCredentials {
            username: value.username,
            password: value.password,
        }
    }
}

// SmtpServerConfig
impl TryFrom<rustmailer_grpc::SmtpServerConfig> for SmtpServerConfig {
    type Error = &'static str;

    fn try_from(value: rustmailer_grpc::SmtpServerConfig) -> Result<Self, Self::Error> {
        Ok(SmtpServerConfig {
            host: value.host,
            port: value.port as u16,
            encryption: value.encryption.try_into()?,
        })
    }
}

impl From<SmtpServerConfig> for rustmailer_grpc::SmtpServerConfig {
    fn from(value: SmtpServerConfig) -> Self {
        rustmailer_grpc::SmtpServerConfig {
            host: value.host,
            port: value.port as u32,
            encryption: value.encryption.into(),
        }
    }
}

// MtaCreateRequest
impl TryFrom<rustmailer_grpc::MtaCreateRequest> for MTACreateRequest {
    type Error = &'static str;
    fn try_from(value: rustmailer_grpc::MtaCreateRequest) -> Result<Self, Self::Error> {
        Ok(MTACreateRequest {
            description: value.description,
            credentials: value
                .credentials
                .ok_or("field 'credentials' missing")?
                .into(),
            server: value.server.ok_or("field 'server' missing")?.try_into()?,
            dsn_capable: value.dsn_capable,
            use_proxy: value.use_proxy,
        })
    }
}

// MtaUpdateRequest
impl TryFrom<rustmailer_grpc::MtaUpdateRequest> for MTAUpdateRequest {
    type Error = &'static str;
    fn try_from(value: rustmailer_grpc::MtaUpdateRequest) -> Result<Self, Self::Error> {
        Ok(MTAUpdateRequest {
            description: value.description,
            credentials: value.credentials.map(Into::into),
            server: value.server.map(SmtpServerConfig::try_from).transpose()?,
            dsn_capable: value.dsn_capable,
            use_proxy: value.use_proxy,
        })
    }
}

impl From<Mta> for rustmailer_grpc::Mta {
    fn from(value: Mta) -> Self {
        Self {
            id: value.id,
            description: value.description,
            credentials: Some(value.credentials.into()),
            server: Some(value.server.into()),
            created_at: value.created_at,
            dsn_capable: value.dsn_capable,
            updated_at: value.updated_at,
            last_access_at: value.last_access_at,
            use_proxy: value.use_proxy,
        }
    }
}

impl From<DataPage<Mta>> for PagedMta {
    fn from(value: DataPage<Mta>) -> Self {
        Self {
            current_page: value.current_page,
            page_size: value.page_size,
            total_items: value.total_items,
            items: value.items.into_iter().map(Into::into).collect(),
            total_pages: value.total_pages,
        }
    }
}

impl From<rustmailer_grpc::SendTestEmailRequest> for SendTestEmailRequest {
    fn from(value: rustmailer_grpc::SendTestEmailRequest) -> Self {
        Self {
            from: value.from,
            to: value.to,
            subject: value.subject,
            message: value.message,
        }
    }
}
