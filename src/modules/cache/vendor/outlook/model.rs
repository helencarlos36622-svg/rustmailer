use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MailFoldersResponse {
    /// The OData context URL
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    /// The list of mail folders
    #[serde(rename = "value")]
    pub value: Vec<MailFolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MailFolder {
    /// The unique identifier of the mail folder (opaque string from Graph API)
    #[serde(rename = "id")]
    pub id: String,

    /// The display name of the mail folder
    #[serde(rename = "displayName")]
    pub display_name: String,

    /// The ID of the parent folder, if any
    #[serde(rename = "parentFolderId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,

    /// Indicates whether the folder is hidden in the client UI
    #[serde(rename = "isHidden")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<bool>,

    /// The size of the folder in bytes
    #[serde(rename = "sizeInBytes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_in_bytes: Option<u64>,

    /// Total number of items in the folder
    #[serde(rename = "totalItemCount")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_item_count: Option<u32>,

    /// Number of unread items in the folder
    #[serde(rename = "unreadItemCount")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread_item_count: Option<u32>,

    /// Number of child folders inside this folder
    #[serde(rename = "childFolderCount")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_folder_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageListResponse {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,

    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,

    pub value: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    #[serde(rename = "@odata.etag")]
    pub etag: Option<String>,
    pub id: String,
    #[serde(rename = "internetMessageId")]
    pub internet_message_id: Option<String>,
    #[serde(rename = "conversationId")]
    pub conversation_id: Option<String>,
    pub subject: Option<String>,
    #[serde(rename = "isRead")]
    pub is_read: Option<bool>,
    #[serde(rename = "receivedDateTime")]
    pub received_date_time: Option<String>,
    #[serde(rename = "sentDateTime")]
    pub sent_date_time: Option<String>,
    pub body: Option<ItemBody>,
    #[serde(rename = "bodyPreview")]
    pub body_preview: Option<String>,
    pub categories: Option<Vec<String>>,
    pub from: Option<Recipient>,
    pub sender: Option<Recipient>,
    #[serde(rename = "replyTo")]
    pub reply_to: Option<Vec<Recipient>>,
    #[serde(rename = "toRecipients")]
    pub to_recipients: Option<Vec<Recipient>>,
    #[serde(rename = "ccRecipients")]
    pub cc_recipients: Option<Vec<Recipient>>,
    #[serde(rename = "bccRecipients")]
    pub bcc_recipients: Option<Vec<Recipient>>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemBody {
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Recipient {
    #[serde(rename = "emailAddress")]
    pub email_address: EmailAddress,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attachment {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    pub size: Option<u32>,
    #[serde(rename = "isInline")]
    pub is_inline: Option<bool>,
    #[serde(rename = "contentId")]
    pub content_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeltaResponse {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,

    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,

    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,

    pub value: Option<Vec<PartialMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialMessage {
    #[serde(rename = "@odata.etag")]
    pub etag: Option<String>,

    #[serde(rename = "@odata.type")]
    pub odata_type: Option<String>,

    pub id: String,
    #[serde(rename = "@removed")]
    pub removed: Option<RemovedInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemovedInfo {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoriesListResponse {
    #[serde(rename = "@odata.context")]
    pub context: Option<String>,

    pub value: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub color: String,
}
