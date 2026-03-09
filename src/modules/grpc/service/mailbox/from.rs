// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::cache::imap::mailbox::{
    Attribute, AttributeEnum, EmailFlag, EnvelopeFlag, MailBox,
};
use crate::modules::grpc::service::rustmailer_grpc;
use crate::modules::mailbox::create::{CreateMailboxRequest, LabelColor};
use crate::modules::mailbox::rename::MailboxUpdateRequest;

impl From<MailBox> for rustmailer_grpc::MailBox {
    fn from(value: MailBox) -> Self {
        Self {
            mailbox_id: value.id,
            account_hash: value.account_id,
            name: value.name,
            delimiter: value.delimiter,
            attributes: value.attributes.into_iter().map(Into::into).collect(),
            flags: value.flags.into_iter().map(Into::into).collect(),
            exists: value.exists,
            unseen: value.unseen,
            permanent_flags: value.permanent_flags.into_iter().map(Into::into).collect(),
            uid_next: value.uid_next,
            uid_validity: value.uid_validity,
            highest_modseq: value.highest_modseq,
        }
    }
}

impl From<AttributeEnum> for i32 {
    fn from(value: AttributeEnum) -> Self {
        match value {
            AttributeEnum::NoInferiors => 0,
            AttributeEnum::NoSelect => 1,
            AttributeEnum::Marked => 2,
            AttributeEnum::Unmarked => 3,
            AttributeEnum::All => 4,
            AttributeEnum::Archive => 5,
            AttributeEnum::Drafts => 6,
            AttributeEnum::Flagged => 7,
            AttributeEnum::Junk => 8,
            AttributeEnum::Sent => 9,
            AttributeEnum::Trash => 10,
            AttributeEnum::Extension => 11,
            AttributeEnum::Unknown => 12,
        }
    }
}

impl From<EmailFlag> for i32 {
    fn from(value: EmailFlag) -> Self {
        match value {
            EmailFlag::Seen => 0,
            EmailFlag::Answered => 1,
            EmailFlag::Flagged => 2,
            EmailFlag::Deleted => 3,
            EmailFlag::Draft => 4,
            EmailFlag::Recent => 5,
            EmailFlag::MayCreate => 6,
            EmailFlag::Custom => 7,
        }
    }
}

impl From<Attribute> for rustmailer_grpc::Attribute {
    fn from(value: Attribute) -> Self {
        Self {
            attr: value.attr.into(),
            extension: value.extension,
        }
    }
}

impl From<EnvelopeFlag> for rustmailer_grpc::EnvelopeFlag {
    fn from(value: EnvelopeFlag) -> Self {
        Self {
            flag: value.flag.into(),
            custom: value.custom,
        }
    }
}

impl From<rustmailer_grpc::MailboxUpdateRequest> for MailboxUpdateRequest {
    fn from(value: rustmailer_grpc::MailboxUpdateRequest) -> Self {
        Self {
            current_name: value.current_name,
            new_name: value.new_name,
            label_color: value.label_color.map(|c| c.into()),
        }
    }
}

impl From<rustmailer_grpc::CreateMailboxRequest> for CreateMailboxRequest {
    fn from(value: rustmailer_grpc::CreateMailboxRequest) -> Self {
        Self {
            mailbox_name: value.mailbox_name,
            parent_name: value.parent_name,
            label_color: value.label_color.map(|c| c.into()),
        }
    }
}

impl From<rustmailer_grpc::LabelColor> for LabelColor {
    fn from(value: rustmailer_grpc::LabelColor) -> Self {
        Self {
            text_color: value.text_color,
            background_color: value.background_color,
        }
    }
}
