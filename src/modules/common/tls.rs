// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem::listener::{RustlsCertificate, RustlsConfig};

use crate::{
    modules::{
        error::{code::ErrorCode, RustMailerResult},
        settings::dir::DATA_DIR_MANAGER,
    },
    raise_error,
};

pub fn rustls_config() -> RustMailerResult<RustlsConfig> {
    let cert = std::fs::read_to_string(&DATA_DIR_MANAGER.tls_cert).map_err(|e| {
        raise_error!(
            format!(
                "Failed to read TLS certificate: '{}' (error: {})",
                DATA_DIR_MANAGER.tls_cert.display(),
                e
            ),
            ErrorCode::InternalError
        )
    })?;

    let key = std::fs::read_to_string(&DATA_DIR_MANAGER.tls_key).map_err(|e| {
        raise_error!(
            format!(
                "Failed to read TLS private key: '{}' (error: {})",
                DATA_DIR_MANAGER.tls_key.display(),
                e
            ),
            ErrorCode::InternalError
        )
    })?;
    let rustls_certificate = RustlsCertificate::new().cert(cert).key(key);
    Ok(RustlsConfig::new().fallback(rustls_certificate))
}
