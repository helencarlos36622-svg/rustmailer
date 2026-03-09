// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MimePart {
    pub mime_type: MimeType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum MimeType {
    /// message/delivery-status
    MessageDeliveryStatus,
    /// text/delivery-status
    TextDeliveryStatus,
    /// text/rfc822-headers
    TextRfc822Headers,
    /// message/rfc822-headers
    MessageRfc822Headers,
    /// message/rfc822
    MessageRfc822,
    /// text/rfc822
    TextRfc822,
    /// message/feedback-report
    FeedbackReport,
}
