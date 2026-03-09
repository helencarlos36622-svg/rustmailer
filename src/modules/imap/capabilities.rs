// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::code::ErrorCode;
use crate::modules::imap::session::SessionStream;
use crate::{modules::error::RustMailerResult, raise_error};
use async_imap::types::Capability;
use async_imap::{types::Capabilities, Session};

pub async fn fetch_capabilities(
    session: &mut Session<Box<dyn SessionStream>>,
) -> RustMailerResult<Capabilities> {
    session
        .capabilities()
        .await
        .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::ImapCommandFailed))
}

pub fn check_capabilities(capabilities: &Capabilities) -> RustMailerResult<()> {
    if !capabilities.has_str("IMAP4rev1") {
        return Err(raise_error!(
            "Server does not support IMAP4rev1".into(),
            ErrorCode::Incompatible
        ));
    }
    Ok(())
}

pub fn capability_to_string(capability: &Capability) -> String {
    match capability {
        Capability::Imap4rev1 => "IMAP4rev1".into(),
        Capability::Auth(v) => format!("AUTH={}", v),
        Capability::Atom(v) => v.into(),
    }
}
