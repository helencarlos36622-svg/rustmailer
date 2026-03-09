// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::sync::LazyLock;

use crate::{
    decrypt, encrypt,
    modules::{
        error::{code::ErrorCode, RustMailerResult},
        settings::cli::SETTINGS,
    },
    raise_error,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::warn;
use url::Url;

pub static HREF_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"href\s*=\s*"([^"]+)""#).unwrap());

pub struct EmailTracker {
    original_html: String,
    modified: bool,
    html: String,
    campaign_id: String,
    message_id: String,
    recipient: String,
    base_url: String,
    account_id: u64,
    account_email: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TrackType {
    Click,
    Open,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingPayload {
    pub track_type: TrackType,
    pub account_id: u64,
    pub account_email: String,
    pub campaign_id: String,
    pub recipient: String,
    pub message_id: String,
    pub url: Option<String>, // only present in click events
}

impl EmailTracker {
    pub fn new(
        campaign_id: String,
        message_id: String,
        recipient: String,
        account_id: u64,
        account_email: String,
    ) -> Self {
        let message_id = message_id
            .trim_matches(|c| c == '<' || c == '>')
            .to_string();

        let base_url = format!(
            "{}",
            SETTINGS.rustmailer_email_tracking_url.trim_end_matches('/')
        );

        EmailTracker {
            original_html: Default::default(),
            modified: false,
            html: Default::default(),
            campaign_id,
            message_id,
            recipient,
            base_url,
            account_id,
            account_email,
        }
    }

    pub fn set_html(&mut self, html: String) {
        self.original_html = html.clone();
        self.html = html;
    }

    /// Track links in the email HTML by replacing them with tracking URLs
    pub fn track_links(&mut self) {
        self.html = HREF_PATTERN
            .replace_all(&self.html, |caps: &regex::Captures| {
                if let Some(url_match) = caps.get(1) {
                    let url = url_match.as_str();

                    // Validate URL
                    if let Ok(parsed_url) = Url::parse(url) {
                        if parsed_url.scheme().is_empty() || parsed_url.host().is_none() {
                            return caps[0].to_string();
                        }

                        match self.get_tracking_url(url) {
                            Ok(tracking_url) => return format!(r#"href="{}""#, tracking_url),
                            Err(e) => {
                                warn!("Failed to get tracking URL for {}: {:#?}", url, e);
                                return caps[0].to_string(); // fallback to original
                            }
                        }
                    }
                }

                caps[0].to_string()
            })
            .into_owned();

        self.modified = self.original_html != self.html;
    }

    /// Generate a tracking URL for click tracking
    fn get_tracking_url(&self, url: &str) -> RustMailerResult<String> {
        let payload = TrackingPayload {
            track_type: TrackType::Click,
            campaign_id: self.campaign_id.clone(),
            recipient: self.recipient.clone(),
            account_id: self.account_id,
            account_email: self.account_email.clone(),
            message_id: self.message_id.clone(),
            url: Some(url.to_string()),
        };
        Ok(format!("{}/{}", self.base_url, Self::encrypt(payload)?))
    }

    /// Append a tracking pixel to the email HTML
    pub fn append_tracking_pixel(&mut self) -> RustMailerResult<()> {
        let tracking_pixel = format!(
            r#"<img src="{}" style="opacity:0; position:absolute; left:-9999px;" alt="" />"#,
            self.get_tracking_pixel()?
        );

        if self.html.contains("</body>") {
            self.html = self
                .html
                .replace("</body>", &format!("{}</body>", tracking_pixel));
            self.modified = true;
            return Ok(());
        }

        if self.html.contains("</html>") {
            self.html = self
                .html
                .replace("</html>", &format!("{}</html>", tracking_pixel));
            self.modified = true;
            return Ok(());
        }

        self.html.push_str(&tracking_pixel);
        self.modified = true;
        return Ok(());
    }

    /// Generate a tracking pixel URL for open tracking
    fn get_tracking_pixel(&self) -> RustMailerResult<String> {
        let payload = TrackingPayload {
            track_type: TrackType::Open,
            campaign_id: self.campaign_id.clone(),
            recipient: self.recipient.clone(),
            account_id: self.account_id,
            account_email: self.account_email.clone(),
            message_id: self.message_id.clone(),
            url: None,
        };
        Ok(format!("{}/{}", self.base_url, Self::encrypt(payload)?))
    }

    /// Placeholder for encryption function - replace with actual implementation
    fn encrypt(data: TrackingPayload) -> RustMailerResult<String> {
        let json = serde_json::to_string(&data).map_err(|e| {
            raise_error!(
                format!("Failed to serialize tracking payload: {}", e),
                ErrorCode::InternalError
            )
        })?;
        encrypt!(&json)
    }

    /// Get the modified HTML
    pub fn get_html(&self) -> &str {
        &self.html
    }

    pub fn decrypt_payload(payload: &str) -> RustMailerResult<TrackingPayload> {
        let decrypted = decrypt!(payload)?;
        let map: TrackingPayload = serde_json::from_str(&decrypted).map_err(|_| {
            raise_error!("Invalid tracking payload".into(), ErrorCode::InternalError)
        })?;
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_tracker() -> EmailTracker {
        EmailTracker::new(
            "test-campaign".to_string(),
            "<test-message-id>".to_string(),
            "test@example.com".to_string(),
            1000u64,
            "test@example.com".to_string(),
        )
    }

    #[test]
    fn test_track_links_replaces_href() {
        let mut tracker = build_tracker();
        tracker.set_html(r#"<a href="https://example.com/page">Click</a>"#.into());
        tracker.track_links();
        println!("{}", &tracker.get_html());
        assert!(tracker.get_html().contains("href=\"http"));
    }

    #[test]
    fn test_append_tracking_pixel_adds_img() {
        let mut tracker = build_tracker();
        tracker.set_html("<html><body>Hello</body></html>".into());
        tracker.append_tracking_pixel().unwrap();
        println!("{}", &tracker.get_html());
        assert!(tracker.get_html().contains("<img src="));
    }

    #[test]
    fn test_append_tracking_pixel_appends_if_no_body_or_html() {
        let mut tracker = build_tracker();
        tracker.set_html("<div>Hello</div>".into());
        tracker.append_tracking_pixel().unwrap();

        assert!(tracker.get_html().contains("<img src="));
    }

    #[test]
    fn test_get_tracking_url_returns_url() {
        let mut tracker = build_tracker();
        tracker.set_html("dummy".into());
        let tracking_url = tracker.get_tracking_url("https://example.com").unwrap();
        assert!(tracking_url.starts_with(&tracker.base_url));
    }

    #[test]
    fn test_does_not_modify_invalid_url() {
        let mut tracker = build_tracker();
        tracker.set_html(r#"<a href="javascript:void(0)">Click</a>"#.into());
        tracker.track_links();

        assert_eq!(tracker.get_html(), tracker.original_html);
    }

    #[test]
    fn test_encrypt_and_decrypt_tracking_payload() {
        let payload = TrackingPayload {
            track_type: TrackType::Open,
            campaign_id: "test-campaign".into(),
            recipient: "test@example.com".into(),
            message_id: "test-message-id".into(),
            account_id: 1000u64,
            account_email: "test@example.com".into(),
            url: None,
        };

        let encrypted = EmailTracker::encrypt(payload).unwrap();
        println!("{}", &encrypted);
        let decrypted = EmailTracker::decrypt_payload(&encrypted).unwrap();

        assert_eq!(decrypted.track_type, TrackType::Open);
        assert_eq!(decrypted.campaign_id, "test-campaign".to_string());
        assert_eq!(decrypted.recipient, "test@example.com".to_string());
    }
}
