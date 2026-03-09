// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::error::RustMailerResult;

pub trait EmailBuilder {
    async fn validate(&self) -> RustMailerResult<()>;
    async fn build(&self, account_id: u64) -> RustMailerResult<()>;
}
