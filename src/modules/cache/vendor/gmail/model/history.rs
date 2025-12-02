// Copyright © 2025 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryList {
    #[serde(default)]
    pub history: Vec<History>,
    #[serde(rename = "historyId")]
    pub history_id: String,//This is the current maximum history ID for the entire account.
    #[serde(rename = "nextPageToken")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageIndex {
    pub id: String,
    #[serde(default, rename = "labelIds")]
    pub label_ids: Vec<String>,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MessageObject {
    pub message: MessageIndex,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LabelMessageObject {
    #[serde(rename = "labelIds")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,

    pub message: MessageIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub id: String,
    #[serde(rename = "labelsAdded")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_added: Vec<LabelMessageObject>,
    #[serde(rename = "labelsRemoved")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_removed: Vec<LabelMessageObject>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<MessageIndex>,
    #[serde(rename = "messagesAdded")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages_added: Vec<MessageObject>,
    #[serde(rename = "messagesDeleted")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages_deleted: Vec<MessageObject>,
}

impl History {
    pub fn has_changes(&self) -> bool {
        !(self.labels_added.is_empty()
            && self.labels_removed.is_empty()
            && self.messages_added.is_empty()
            && self.messages_deleted.is_empty())
    }
}
