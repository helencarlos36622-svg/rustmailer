// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::account::entity::Encryption;
use crate::modules::database::manager::DB_MANAGER;
use crate::modules::database::{
    delete_impl, paginate_query_primary_scan_all_impl, secondary_find_impl, update_impl,
};
use crate::modules::error::code::ErrorCode;
use crate::modules::rest::response::DataPage;
use crate::modules::smtp::mta::payload::MTACreateRequest;
use crate::modules::smtp::mta::payload::MTAUpdateRequest;
use crate::{encrypt, id, raise_error};
use crate::{modules::database::insert_impl, modules::error::RustMailerResult, utc_now};
use native_db::*;
use native_model::{native_model, Model};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
#[native_model(id = 7, version = 1)]
#[native_db(primary_key(pk -> String))]
pub struct Mta {
    #[secondary_key(unique)]
    pub id: u64,
    /// Optional descriptive text about the MTA.
    pub description: Option<String>,

    /// Credentials used for authenticating with the MTA server.
    pub credentials: MTACredentials,

    /// SMTP server configuration details.
    pub server: SmtpServerConfig,

    /// Timestamp (Unix epoch milliseconds) when the MTA was created.
    pub created_at: i64,

    /// Indicates if the MTA supports DSN (Delivery Status Notification).
    pub dsn_capable: bool,

    /// Timestamp (Unix epoch milliseconds) when the MTA was last updated.
    pub updated_at: i64,

    /// Timestamp (Unix epoch milliseconds) when the MTA was last accessed.
    pub last_access_at: i64,

    /// Optional proxy ID for establishing the connection.
    /// - If `None` or not provided, the client will connect directly to the MTA server.
    /// - If `Some(proxy_id)`, the client will use the pre-configured proxy with the given ID.
    pub use_proxy: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct MTACredentials {
    /// Username for MTA authentication.
    #[oai(validator(min_length = 1, max_length = 256))]
    pub username: String,

    /// Password for MTA authentication.
    ///
    /// Users should provide a plaintext password (1 to 256 characters).
    /// The server will encrypt the password using AES-256-GCM and securely store it.
    /// The plaintext password is never stored, so users must remember it for authentication.
    #[oai(validator(min_length = 1, max_length = 256))]
    pub password: Option<String>,
}

impl MTACredentials {
    pub fn encrypt(self) -> RustMailerResult<Self> {
        let password = &self.password.ok_or_else(|| {
            raise_error!(
                "Password is required for creating an MTA.".into(),
                ErrorCode::InternalError
            )
        })?;

        Ok(Self {
            username: self.username,
            password: Some(encrypt!(&password)?),
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct SmtpServerConfig {
    /// Hostname or IP address of the SMTP server.
    #[oai(validator(max_length = 253, pattern = r"^[a-zA-Z0-9\-\.]+$"))]
    pub host: String,

    /// Port number on which the SMTP server listens.
    #[oai(validator(minimum(value = "1"), maximum(value = "65535")))]
    pub port: u16,

    /// Connection encryption method
    pub encryption: Encryption,
}

impl Mta {
    fn pk(&self) -> String {
        format!("{}_{}", self.created_at, self.id)
    }

    pub fn new(value: MTACreateRequest) -> RustMailerResult<Self> {
        Ok(Self {
            id: id!(64),
            description: value.description,
            credentials: value.credentials.encrypt()?,
            server: value.server,
            dsn_capable: value.dsn_capable,
            created_at: utc_now!(),
            updated_at: utc_now!(),
            last_access_at: Default::default(),
            use_proxy: value.use_proxy,
        })
    }

    pub async fn paginate_list(
        page: Option<u64>,
        page_size: Option<u64>,
        desc: Option<bool>,
    ) -> RustMailerResult<DataPage<Mta>> {
        paginate_query_primary_scan_all_impl(DB_MANAGER.meta_db(), page, page_size, desc)
            .await
            .map(DataPage::from)
    }

    pub async fn get(id: u64) -> RustMailerResult<Option<Mta>> {
        secondary_find_impl(DB_MANAGER.meta_db(), MtaKey::id, id).await
    }

    pub async fn delete(id: u64) -> RustMailerResult<()> {
        delete_impl(DB_MANAGER.meta_db(), move |rw| {
            rw.get()
                .secondary::<Mta>(MtaKey::id, id)
                .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                .ok_or_else(|| {
                    raise_error!(
                        format!("The MTA with id={id} that you want to delete was not found."),
                        ErrorCode::ResourceNotFound
                    )
                })
        })
        .await
    }

    pub async fn save(self) -> RustMailerResult<()> {
        insert_impl(DB_MANAGER.meta_db(), self).await
    }

    pub async fn update(id: u64, request: MTAUpdateRequest) -> RustMailerResult<()> {
        update_impl(
            DB_MANAGER.meta_db(),
            move |rw| {
                rw.get()
                    .secondary::<Mta>(MtaKey::id, id)
                    .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::InternalError))?
                    .ok_or_else(|| {
                        raise_error!(
                            format!(
                                "The MTA with id={id} that you want to modify was not found."
                            ),
                            ErrorCode::ResourceNotFound
                        )
                    })
            },
            |current| apply_update(current, request),
        )
        .await?;

        Ok(())
    }
}

fn apply_update(old: &Mta, request: MTAUpdateRequest) -> RustMailerResult<Mta> {
    let mut new = old.clone();
    if let Some(credentials) = request.credentials {
        new.credentials.username = credentials.username;
        if let Some(password) = credentials.password {
            new.credentials.password = Some(encrypt!(&password)?);
        }
    }
    if let Some(server) = request.server {
        new.server = server;
    }
    if let Some(description) = request.description {
        new.description = Some(description);
    }
    if let Some(dsn_capable) = request.dsn_capable {
        new.dsn_capable = dsn_capable;
    }

    if let Some(use_proxy) = request.use_proxy {
        new.use_proxy = Some(use_proxy);
    }

    new.updated_at = utc_now!();
    Ok(new)
}
