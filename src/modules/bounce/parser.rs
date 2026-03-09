// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use ahash::AHashMap;
use mail_parser::{Address, Message, MessageParser, MimeHeaders};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::modules::common::AddrVec;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct BounceReport {
    /// Optional raw headers of the original email associated with the bounce or feedback event.
    pub original_headers: Option<RawEmailHeaders>,
    /// Optional delivery status information for the bounced email.
    pub delivery_status: Option<DeliveryStatus>,
    /// Optional feedback report details (e.g., spam or abuse report) for the email.
    pub feedback_report: Option<FeedbackReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct RawEmailHeaders {
    /// Optional unique message ID of the email.
    pub message_id: Option<String>,
    /// Optional subject line of the email.
    pub subject: Option<String>,
    /// Optional sender email address of the email.
    pub from: Option<String>,
    /// Optional list of recipient email addresses (To field) for the email.
    pub to: Option<Vec<String>>,
    /// Optional date (in milliseconds) of the email, typically from the email's header.
    pub date: Option<i64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct DeliveryStatus {
    /// Optional email address of the recipient for which the delivery failed.
    pub recipient: Option<String>,
    /// Optional action taken by the mail server (e.g., "failed", "delayed").
    pub action: Option<String>,
    /// Optional status code or description of the delivery outcome.
    pub status: Option<String>,
    /// Optional source of the error (e.g., "smtp", "dns").
    pub error_source: Option<String>,
    /// Optional diagnostic code providing details about the delivery failure.
    pub diagnostic_code: Option<String>,
    /// Optional remote MTA (Mail Transfer Agent) involved in the delivery attempt.
    pub remote_mta: Option<String>,
    /// Optional reporting MTA that generated the bounce report.
    pub reporting_mta: Option<String>,
    /// Optional MTA that received the email before the bounce.
    pub received_from_mta: Option<String>,
    /// Optional Postfix queue ID associated with the email.
    pub postfix_queue_id: Option<String>,
    /// Optional date and time when the email arrived at the MTA, as a string.
    pub arrival_date: Option<String>,
    /// Optional message ID of the original email.
    pub original_message_id: Option<String>,
    /// Optional sender email address used by the Postfix system.
    pub postfix_sender: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct FeedbackReport {
    /// Optional type of feedback (e.g., "abuse", "spam", "unsubscribe").
    pub feedback_type: Option<String>,
    /// Optional version of the feedback report format.
    pub version: Option<String>,
    /// Optional user agent of the system or client reporting the feedback.
    pub user_agent: Option<String>,
    /// Optional original sender email address from the email's envelope.
    pub original_mail_from: Option<String>,
    /// Optional original recipient email address from the email's envelope.
    pub original_rcpt_to: Option<String>,
    /// Optional original envelope ID of the email.
    pub original_envelope_id: Option<String>,
    /// Optional date and time when the feedback was received, as a string.
    pub received_date: Option<String>,
    /// Optional domain reported in the feedback.
    pub reported_domain: Option<String>,
    /// Optional URI reported in the feedback (e.g., for unsubscribe links).
    pub reported_uri: Option<String>,
    /// Optional reporting MTA that generated the feedback report.
    pub reporting_mta: Option<String>,
    /// Optional IP address of the source sending the feedback.
    pub source_ip: Option<String>,
    /// Optional port used by the source sending the feedback.
    pub source_port: Option<String>,
    /// Optional DNS record used for SPF (Sender Policy Framework) validation.
    pub spf_dns: Option<String>,
    /// Optional result of the delivery attempt associated with the feedback.
    pub delivery_result: Option<String>,
    /// Optional authentication results (e.g., DKIM, SPF) from the feedback.
    pub authentication_results: Option<String>,
    /// Optional details of any authentication failure.
    pub auth_failure: Option<String>,
    /// Optional date and time when the email arrived, as a string.
    pub arrival_date: Option<String>,
}

impl FeedbackReport {
    pub fn is_empty(&self) -> bool {
        self.feedback_type.is_none()
            && self.version.is_none()
            && self.user_agent.is_none()
            && self.original_mail_from.is_none()
            && self.original_rcpt_to.is_none()
            && self.original_envelope_id.is_none()
            && self.received_date.is_none()
            && self.reported_domain.is_none()
            && self.reported_uri.is_none()
            && self.reporting_mta.is_none()
            && self.source_ip.is_none()
            && self.source_port.is_none()
            && self.spf_dns.is_none()
            && self.delivery_result.is_none()
            && self.authentication_results.is_none()
            && self.auth_failure.is_none()
            && self.arrival_date.is_none()
    }
}

impl DeliveryStatus {
    pub fn is_empty(&self) -> bool {
        self.recipient.is_none()
            && self.action.is_none()
            && self.status.is_none()
            && self.error_source.is_none()
            && self.diagnostic_code.is_none()
            && self.remote_mta.is_none()
            && self.reporting_mta.is_none()
            && self.received_from_mta.is_none()
            && self.postfix_queue_id.is_none()
            && self.arrival_date.is_none()
            && self.original_message_id.is_none()
            && self.postfix_sender.is_none()
    }
}

pub fn extract_bounce_report(message: &Message<'_>) -> BounceReport {
    let mut delivery_status = extract_workmail_delivery_status(message);
    if delivery_status.is_none() {
        delivery_status = parse_delivery_status_from_part(message);
    }

    let mut original_headers = parse_original_message_headers(message);
    if original_headers.is_none() {
        original_headers = parse_original_message(message);
    }

    let feedback_report = parse_feedback_report_from_part(message);

    BounceReport {
        original_headers,
        delivery_status,
        feedback_report,
    }
}

fn extract_from_address<'x>(address: Option<&Address<'x>>) -> Option<String> {
    address
        .map(Into::<AddrVec>::into)
        .and_then(|addr_vec| addr_vec.0.first().and_then(|addr| addr.address.clone()))
}

fn extract_to_addresses<'x>(address: Option<&Address<'x>>) -> Option<Vec<String>> {
    address.map(|addr| {
        let addr_vec: AddrVec = addr.into();
        addr_vec
            .0
            .into_iter()
            .filter_map(|addr| addr.address)
            .collect::<Vec<String>>()
    })
}

fn try_parse_rfc822_text(text: &str) -> Option<RawEmailHeaders> {
    let text = clean_leading_space(text);
    MessageParser::new()
        .parse(&text)
        .filter(|m| !m.is_empty())
        .map(|message| RawEmailHeaders {
            message_id: message.message_id().map(String::from),
            subject: message.subject().map(String::from),
            from: extract_from_address(message.from()),
            to: extract_to_addresses(message.to()),
            date: message.date().map(|d| d.to_timestamp() * 1000),
        })
}

fn parse_original_message(message: &Message<'_>) -> Option<RawEmailHeaders> {
    let part = message.parts.iter().find(|p| {
        p.content_type()
            .and_then(|ct| ct.subtype())
            .is_some_and(|st| st.eq_ignore_ascii_case("rfc822"))
    })?;

    if part.is_message() {
        part.message().and_then(|sub_message| {
            if sub_message.is_empty() {
                None
            } else {
                Some(RawEmailHeaders {
                    message_id: sub_message.message_id().map(String::from),
                    subject: sub_message.subject().map(String::from),
                    from: extract_from_address(sub_message.from()),
                    to: extract_to_addresses(sub_message.to()),
                    date: sub_message.date().map(|d| d.to_timestamp() * 1000),
                })
            }
        })
    } else {
        part.text_contents().and_then(try_parse_rfc822_text)
    }
}

fn clean_leading_space(text: &str) -> String {
    text.trim()
        .lines()
        .map(|line| {
            if line.starts_with(' ')
                && line
                    .chars()
                    .nth(1)
                    .map(|c| !c.is_whitespace())
                    .unwrap_or(false)
            {
                &line[1..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_original_message_headers(message: &Message<'_>) -> Option<RawEmailHeaders> {
    let rfc822_header_part = message.parts.iter().find(|p| {
        p.content_type()
            .and_then(|ct| ct.subtype())
            .map(|st| st.to_lowercase().contains("rfc822-headers"))
            .unwrap_or(false)
    });

    let part = rfc822_header_part?;

    let sub_message = part.is_message().then(|| part.message()).flatten()?;
    if sub_message.is_empty() {
        return None;
    }

    let headers = RawEmailHeaders {
        message_id: sub_message.message_id().map(String::from),
        subject: sub_message.subject().map(String::from),
        from: extract_from_address(sub_message.from()),
        to: extract_to_addresses(sub_message.to()),
        date: sub_message.date().map(|d| d.to_timestamp() * 1000),
    };

    Some(headers)
}

fn parse_delivery_status_from_part(message: &Message<'_>) -> Option<DeliveryStatus> {
    let part = message.parts.iter().find(|p| {
        p.content_type()
            .and_then(|ct| ct.subtype())
            .map(|st| st.to_lowercase().contains("delivery-status"))
            .unwrap_or(false)
    });

    let part = part?;

    if part.is_message() {
        let message = part.message()?;
        if message.is_empty() {
            return None;
        }
    }

    let text = part.text_contents()?;

    let status = parse_delivery_status(text);
    if status.is_empty() {
        None
    } else {
        Some(status)
    }
}

fn parse_feedback_report_from_part(message: &Message<'_>) -> Option<FeedbackReport> {
    let part = message.parts.iter().find(|p| {
        p.content_type()
            .and_then(|ct| ct.subtype())
            .map(|st| st.to_lowercase().contains("feedback-report"))
            .unwrap_or(false)
    });

    let part = part?;

    if part.is_message() {
        let message = part.message()?;
        if message.is_empty() {
            return None;
        }
    }

    let text = part.text_contents()?;
    let report = parse_feedback_report(text);
    if report.is_empty() {
        None
    } else {
        Some(report)
    }
}

fn extract_workmail_delivery_status(message: &Message<'_>) -> Option<DeliveryStatus> {
    let is_amazon_workmail = get_header_value(message, "x-mailer")
        .map(|s| s.to_lowercase())
        .map(|s| s.contains("amazon workmail"))
        .unwrap_or(false);

    if !is_amazon_workmail {
        return None;
    }

    let text = message.body_text(0)?;

    let split_marker = "technical report:";
    let text_lower = text.to_lowercase();
    let split_pos = text_lower.find(split_marker)?;

    let bounce_details = text[split_pos + split_marker.len()..].trim();
    let status = parse_delivery_status(bounce_details);
    if !status.is_empty() {
        Some(status)
    } else {
        None
    }
}

fn extract_after_semicolon(value: &str) -> String {
    value
        .find(';')
        .map(|pos| value[pos + 1..].trim().to_string())
        .unwrap_or_else(|| value.trim().to_string())
}

fn parse_feedback_report(feedback_report: &str) -> FeedbackReport {
    let mut details = AHashMap::new();

    for line in feedback_report.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let Some((key, value)) = trimmed.split_once(':') {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim();
                if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                    details.insert(key_trimmed.to_lowercase(), value_trimmed.to_string());
                }
            }
        }
    }

    FeedbackReport {
        feedback_type: details.get("feedback-type").cloned(),
        version: details.get("version").cloned(),
        user_agent: details.get("user-agent").cloned(),
        original_mail_from: details.get("original-mail-from").cloned(),
        original_rcpt_to: details.get("original-rcpt-to").cloned(),
        original_envelope_id: details.get("original-envelope-id").cloned(),
        received_date: details.get("received-date").cloned(),
        reported_domain: details.get("reported-domain").cloned(),
        reported_uri: details.get("reported-uri").cloned(),
        reporting_mta: details.get("reporting-mta").cloned(),
        source_ip: details.get("source-ip").cloned(),
        source_port: details.get("source-port").cloned(),
        spf_dns: details.get("spf-dns").cloned(),
        delivery_result: details.get("delivery-result").cloned(),
        authentication_results: details.get("authentication-results").cloned(),
        auth_failure: details.get("auth-failure").cloned(),
        arrival_date: details.get("arrival-date").cloned(),
    }
}

fn parse_delivery_status(bounce_details: &str) -> DeliveryStatus {
    let mut details = AHashMap::new();
    let mut result = DeliveryStatus::default();

    for line in bounce_details.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let Some((key, value)) = trimmed.split_once(':') {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim();
                if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                    details.insert(key_trimmed.to_lowercase(), value_trimmed.to_string());
                }
            }
        }
    }

    if let Some(value) = details.get("final-recipient").filter(|v| !v.is_empty()) {
        result.recipient = Some(extract_after_semicolon(value));
    } else if let Some(value) = details.get("original-recipient").filter(|v| !v.is_empty()) {
        result.recipient = Some(extract_after_semicolon(value));
    }

    if let Some(code) = details.get("diagnostic-code").filter(|v| !v.is_empty()) {
        if let Some(split_pos) = code.find(';') {
            result.error_source = Some(code[..split_pos].trim().to_string());
            result.diagnostic_code = Some(code[split_pos + 1..].trim().to_string());
        } else {
            result.diagnostic_code = Some(code.to_string());
        }
    }

    for (key, field) in [
        ("remote-mta", &mut result.remote_mta),
        ("reporting-mta", &mut result.reporting_mta),
        ("received-from-mta", &mut result.received_from_mta),
        ("x-postfix-sender", &mut result.postfix_sender),
    ] {
        if let Some(value) = details.get(key).filter(|v| !v.is_empty()) {
            *field = Some(extract_after_semicolon(value));
        }
    }

    for (key, field) in [
        ("action", &mut result.action),
        ("status", &mut result.status),
        ("x-postfix-queue-id", &mut result.postfix_queue_id),
        ("arrival-date", &mut result.arrival_date),
        ("x-original-message-id", &mut result.original_message_id),
    ] {
        if let Some(value) = details.get(key) {
            *field = Some(value.to_string());
        }
    }

    result
}

fn get_header_value(message: &Message<'_>, key: &str) -> Option<String> {
    if message.is_empty() {
        return None;
    }
    message
        .headers()
        .iter()
        .find(|header| header.name().to_lowercase() == key.to_lowercase())
        .and_then(|header| header.value().as_text().map(|s| s.trim().to_string()))
}