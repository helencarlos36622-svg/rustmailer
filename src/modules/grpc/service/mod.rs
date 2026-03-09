// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

pub mod account;
pub mod autoconfig;
pub mod hook;
pub mod mailbox;
pub mod message;
pub mod mta;
pub mod oauth2;
pub mod send;
pub mod status;
pub mod template;

pub mod rustmailer_grpc {
    poem_grpc::include_proto!("rustmailer.grpc");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        poem_grpc::include_file_descriptor_set!("rustmailer.bin");
}
