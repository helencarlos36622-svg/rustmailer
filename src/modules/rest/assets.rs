// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web/dist/"]
pub struct FrontEndAssets;
