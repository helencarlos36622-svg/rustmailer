// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::cache::vendor::gmail::sync::labels::GmailLabels;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LabelList {
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Label {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub label_type: String, // "system" or "user"
    #[serde(rename = "labelListVisibility")]
    pub label_list_visibility: Option<String>,
    #[serde(rename = "messageListVisibility")]
    pub message_list_visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LabelDetail {
    /// Optional color configuration for user-created labels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<serde_json::Value>,
    /// Unique identifier of the label
    pub id: String,
    /// Visibility of the label in Gmail's label list
    #[serde(rename = "labelListVisibility")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_list_visibility: Option<String>,
    /// Visibility of messages with this label in Gmail's message list
    #[serde(rename = "messageListVisibility")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_list_visibility: Option<String>,
    /// Total number of messages with this label
    #[serde(rename = "messagesTotal")]
    pub messages_total: Option<u32>,
    /// Number of unread messages with this label
    #[serde(rename = "messagesUnread")]
    pub messages_unread: Option<u32>,
    /// Display name of the label
    pub name: String,
    /// Total number of threads with this label
    #[serde(rename = "threadsTotal")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_total: Option<i64>,
    /// Number of unread threads with this label
    #[serde(rename = "threadsUnread")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_unread: Option<i64>,
    /// Type of the label ("user" or "system")
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

impl From<LabelDetail> for GmailLabels {
    fn from(label: LabelDetail) -> Self {
        Self {
            id: 0,
            account_id: 0,
            name: label.name,
            exists: label.messages_total.unwrap_or_default(),
            unseen: label.messages_unread.unwrap_or_default(),
            label_id: label.id,
        }
    }
}
