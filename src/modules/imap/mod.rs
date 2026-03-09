// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

pub mod capabilities;
pub mod client;
pub mod executor;
pub mod flags;
pub mod manager;
pub mod oauth2;
pub mod pool;
pub mod session;
#[cfg(test)]
mod tests;

pub mod decoder;
pub mod section;
pub mod stats;
