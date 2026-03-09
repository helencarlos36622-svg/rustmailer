// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::autoconfig::entity::{MailServerConfig, ServerConfig};
use crate::modules::error::code::ErrorCode;
use crate::{
    modules::{
        account::entity::Encryption, autoconfig::CachedMailSettings, error::RustMailerResult,
    },
    raise_error,
};
use autoconfig::config::{Server, ServerType};
use email_address::EmailAddress;
use std::str::FromStr;
use tracing::error;

pub async fn resolve_autoconfig(
    email: impl AsRef<str>,
) -> RustMailerResult<Option<MailServerConfig>> {
    let email = email.as_ref();

    let email_address = EmailAddress::from_str(email).map_err(|error| {
        raise_error!(
            format!("Invalid email address: {email:#?}. {error:#?}"),
            ErrorCode::InvalidParameter
        )
    })?;

    let domain = email_address.domain();
    // try read local cache first
    if let Some(cached_entity) = CachedMailSettings::get(domain).await? {
        return Ok(Some(cached_entity.config));
    }

    let config = autoconfig::from_addr(email_address.email().as_ref())
        .await
        .map_err(|e| {
            error!(email = %email, domain = %domain, error = ?e, "Autoconfig fetch failed");
            raise_error!(
                format!(
                    "Failed to fetch autoconfig for email '{}': {:#?}",
                    email_address.email(),
                    e
                ),
                ErrorCode::AutoconfigFetchFailed
            )
        })?;

    let (imap_server, smtp_server) = match (
        config
            .email_provider()
            .incoming_servers()
            .into_iter()
            .find(|s| matches!(s.server_type(), ServerType::Imap)),
        config
            .email_provider()
            .outgoing_servers()
            .into_iter()
            .find(|s| matches!(s.server_type(), ServerType::Smtp)),
    ) {
        (Some(imap), Some(smtp)) => (imap, smtp),
        _ => return Ok(None),
    };

    let get_encryption = |server: &Server| {
        server
            .security_type()
            .map_or(Encryption::None, |encryption| match encryption {
                autoconfig::config::SecurityType::Plain => Encryption::None,
                autoconfig::config::SecurityType::Starttls => Encryption::StartTls,
                autoconfig::config::SecurityType::Tls => Encryption::Ssl,
            })
    };

    let get_port = |server: &Server, encryption: &Encryption, tls_port: u16, non_tls_port: u16| {
        server.port().map_or_else(
            || match encryption {
                Encryption::StartTls => tls_port,
                _ => non_tls_port,
            },
            ToOwned::to_owned,
        )
    };

    let get_hostname = |server: &Server, default_prefix: &str| {
        server.hostname().map_or_else(
            || format!("{}.{}", default_prefix, domain),
            ToOwned::to_owned,
        )
    };

    let imap_encryption = get_encryption(imap_server);
    let smtp_encryption = get_encryption(smtp_server);

    let imap_config = ServerConfig::new(
        get_hostname(imap_server, "imap"),
        get_port(imap_server, &imap_encryption, 993, 143),
        imap_encryption,
    );

    let smtp_config = ServerConfig::new(
        get_hostname(smtp_server, "smtp"),
        get_port(smtp_server, &smtp_encryption, 465, 587),
        smtp_encryption,
    );
    let result = MailServerConfig {
        imap: imap_config,
        smtp: smtp_config,
        oauth2: config.oauth2().map(|f| f.into()),
    };
    CachedMailSettings::add(domain.into(), result.clone()).await?;
    Ok(Some(result))
}
