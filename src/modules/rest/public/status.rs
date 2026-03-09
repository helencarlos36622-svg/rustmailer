// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::modules::context::status::RustMailerStatus;
use poem::{handler, web::Json, IntoResponse};

#[handler]
pub async fn get_status() -> impl IntoResponse {
    Json(RustMailerStatus::get())
}
