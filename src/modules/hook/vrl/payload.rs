// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use vrl::core::Value;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct VrlScriptTestRequest {
    /// The VRL script program to be tested.
    pub program: String,
    /// Optional event data to be used as input for testing the VRL script.
    pub event: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub enum Outcome {
    Success { output: Value, result: Value },
    Error(String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Object)]
pub struct ResolveResult {
    /// The result of the operation, if successful.
    /// This is represented as a `json value`, allowing for flexible structured data.
    pub result: Option<serde_json::Value>,
    /// An error message describing the failure, if the operation was not successful.
    pub error: Option<String>,
}
