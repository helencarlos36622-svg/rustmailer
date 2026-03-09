// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::{
        bounce::{detect::BounceMessageExtractor, title::analyze_subject_for_bounce},
        error::{code::ErrorCode, RustMailerResult},
    },
    raise_error,
};
use async_imap::types::Fetch;
use mail_parser::MessageParser;

// check newMessage and bounce detect enabled ,and no matter minimal or what, get meta first ,and check this is a bounce,
// if this is a bounce, then get full message and parse it
// if this is not a bounce, then get message content use meta data

//send_notification_and_detect_bounce

pub fn should_extract_bounce_report(fetch: &Fetch) -> RustMailerResult<bool> {
    // Early error handling for missing components
    let header = fetch
        .header()
        .ok_or_else(|| raise_error!("No header available".into(), ErrorCode::InternalError))?;

    let message = MessageParser::default() // Using default() instead of new() if possible
        .parse(header)
        .ok_or_else(|| {
            raise_error!(
                "Failed to parse email header".into(),
                ErrorCode::InternalError
            )
        })?;

    // Extract bounce information more efficiently
    let is_bounce_subject = analyze_subject_for_bounce(message.subject().map(String::from));
    let has_bounce_content =
        BounceMessageExtractor::new(fetch.bodystructure().ok_or_else(|| {
            raise_error!(
                "No email body structure available".into(),
                ErrorCode::InternalError
            )
        })?)
        .get_bounce_message_parts()
        .is_some();

    Ok(is_bounce_subject && has_bounce_content)
}
