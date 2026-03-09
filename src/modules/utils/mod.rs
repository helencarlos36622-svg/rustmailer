// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use base64::{engine::general_purpose, Engine};
use rand::{rng, Rng};

use super::error::code::ErrorCode;

pub mod encrypt;
pub mod net;
pub mod rate_limit;
pub mod shutdown;
pub mod tls;

#[macro_export]
macro_rules! rustmailer_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

#[macro_export]
macro_rules! utc_now {
    () => {{
        use chrono::Utc;
        Utc::now().timestamp_millis()
    }};
}

#[macro_export]
macro_rules! after_n_days_timestamp {
    ($start_ts:expr, $days:expr) => {{
        const MILLIS_PER_DAY: i64 = 86_400_000; // 24 * 60 * 60 * 1000
        $start_ts + ($days as i64) * MILLIS_PER_DAY
    }};
}

#[macro_export]
macro_rules! base64_encode {
    ($bytes:expr) => {{
        use base64::{engine::general_purpose::STANDARD, *};
        STANDARD.encode($bytes)
    }};
}

#[macro_export]
macro_rules! base64_decode {
    ($key:expr) => {{
        use base64::{engine::general_purpose::STANDARD, *};
        STANDARD.decode($key).unwrap()
    }};
}

#[macro_export]
macro_rules! base64_decode_url_safe {
    ($key:expr) => {{
        use base64::{engine::general_purpose::URL_SAFE, *};
        URL_SAFE.decode($key)
    }};
}

#[macro_export]
macro_rules! base64_encode_url_safe {
    ($key:expr) => {{
        use base64::{engine::general_purpose::URL_SAFE, *};
        URL_SAFE.encode($key)
    }};
}

#[macro_export]
macro_rules! product_public_key {
    () => {
        $crate::base64_decode!(r#"BNlT+WjdEls9VGfry+zKygx+UoypxSqsMBddMGxYgbhWOz7Xfh7YJXGMeby9jBtbz3rhSGrTuZCYA9uwwMMYkhI="#)
    };
}

#[macro_export]
macro_rules! license_header {
    () => {
        "{\"alg\":\"ES256\",\"typ\":\"JWT\"}"
    };
}

#[macro_export]
macro_rules! raise_error {
    ($msg:expr, $code:expr) => {
        $crate::modules::error::RustMailerError::Generic {
            message: $msg,
            location: snafu::Location::default(),
            code: $code,
        }
    };
}
#[macro_export]
macro_rules! run_with_timeout {
    ($duration:expr, $task:expr, $err_msg:expr) => {{
        match tokio::time::timeout($duration, $task).await {
            Ok(result) => Ok(result),
            Err(_) => Err($err_msg),
        }
    }};
}

#[macro_export]
macro_rules! free_memory {
    () => {{
        let mut sys = sysinfo::System::new_all();
        sys.refresh_memory();
        sys.free_memory()
    }};
}

#[macro_export]
macro_rules! validate_identifier {
    ($input:expr, $param_name:expr) => {{
        match $crate::modules::utils::validate_id($input, $param_name) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }};
}

pub fn validate_id(input: &str, param_name: &str) -> crate::modules::error::RustMailerResult<()> {
    // Check if the string is empty
    if input.is_empty() {
        return Err(raise_error!(
            format!("'{}' cannot be empty.", param_name),
            ErrorCode::InvalidParameter
        ));
    }

    // Check if the length is greater than 64 characters
    if input.len() > 64 {
        return Err(raise_error!(
            format!("'{}' cannot be longer than 64 characters.", param_name),
            ErrorCode::InvalidParameter
        ));
    }

    // Regular expression: must start with a letter and can contain letters, numbers, underscores, or dashes
    let re = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*").unwrap();
    if re.is_match(input) {
        Ok(())
    } else {
        Err(raise_error!(
            format!("'{}' must start with a letter and can only contain letters, numbers, underscores, or dashes.", param_name), 
            ErrorCode::InvalidParameter
        ))
    }
}

#[macro_export]
macro_rules! generate_token {
    ($bit_strength:expr) => {{
        $crate::modules::utils::generate_token_impl($bit_strength)
    }};
}

pub(crate) fn generate_token_impl(bit_strength: usize) -> String {
    let byte_length = (bit_strength + 23) / 24 * 3;
    let random_bytes: Vec<u8> = (0..byte_length).map(|_| rand::random::<u8>()).collect();
    let mut encoded = general_purpose::URL_SAFE.encode(&random_bytes);

    encoded = encoded
        .chars()
        .map(|c| {
            if c == '/' || c == '+' || c == '-' || c == '_' {
                make_single_random_char()
            } else {
                c
            }
        })
        .collect();

    encoded
}

fn make_single_random_char() -> char {
    let random_bytes: [u8; 3] = rng().random();
    let encoded = general_purpose::URL_SAFE.encode(random_bytes);
    encoded
        .chars()
        .find(|&c| c != '-' && c != '_' && c != '+' && c != '/')
        .unwrap_or('a')
}

#[macro_export]
macro_rules! ensure_access {
    ($dir:expr) => {{
        $crate::modules::utils::ensure_dir_and_test_access($dir)
    }};
}

#[macro_export]
macro_rules! decode_mailbox_name {
    ($name:expr) => {{
        utf7_imap::decode_utf7_imap($name.to_string())
    }};
}
#[macro_export]
macro_rules! encode_mailbox_name {
    ($name:expr) => {{
        utf7_imap::encode_utf7_imap($name.to_string())
    }};
}

#[macro_export]
macro_rules! get_encoding {
    ($label:expr) => {
        match encoding_rs::Encoding::for_label($label.as_bytes()) {
            None => None,
            Some(encoding) => Some(encoding),
        }
    };
}

#[macro_export]
macro_rules! current_datetime {
    () => {{
        use chrono::Local;
        let now = Local::now();
        now.format("%Y%m%d%H%M").to_string()
    }};
}

#[macro_export]
macro_rules! validate_email {
    ($email:expr) => {{
        $crate::modules::utils::validate_email($email)
    }};
}

#[macro_export]
macro_rules! encrypt {
    ($plaintext:expr) => {{
        $crate::modules::utils::encrypt::encrypt_string($plaintext)
    }};
}

#[macro_export]
macro_rules! decrypt {
    ($plaintext:expr) => {{
        $crate::modules::utils::encrypt::decrypt_string($plaintext)
    }};
}

pub fn validate_email(email: &str) -> crate::modules::error::RustMailerResult<()> {
    use std::str::FromStr;
    let email_address = email_address::EmailAddress::from_str(email).map_err(|_| {
        raise_error!(
            format!("Invalid email format : {}", email),
            ErrorCode::InvalidParameter
        )
    })?;
    if email != email_address.email() {
        return Err(raise_error!(
            format!("Invalid email format: {}", email),
            ErrorCode::InvalidParameter
        ));
    }
    Ok(())
}

#[macro_export]
macro_rules! calculate_hash {
    ($name:expr) => {
        $crate::modules::utils::hash($name)
    };
}

#[macro_export]
macro_rules! id {
    ($bit_strength:expr) => {{
        // Generate a token with the given bit strength
        let token = $crate::modules::utils::generate_token_impl($bit_strength);
        // Hash the generated token
        $crate::modules::utils::hash(&token)
    }};
}

pub fn prost_value_to_json_value(prost_value: prost_types::Value) -> serde_json::Value {
    match prost_value.kind {
        Some(kind) => match kind {
            prost_types::value::Kind::NullValue(_) => serde_json::Value::Null,
            prost_types::value::Kind::NumberValue(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
            ),
            prost_types::value::Kind::StringValue(s) => serde_json::Value::String(s),
            prost_types::value::Kind::BoolValue(b) => serde_json::Value::Bool(b),
            prost_types::value::Kind::StructValue(s) => {
                let fields = s
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, prost_value_to_json_value(v)))
                    .collect();
                serde_json::Value::Object(fields)
            }
            prost_types::value::Kind::ListValue(l) => {
                let values = l
                    .values
                    .into_iter()
                    .map(prost_value_to_json_value)
                    .collect();
                serde_json::Value::Array(values)
            }
        },
        None => serde_json::Value::Null,
    }
}

pub fn json_value_to_prost_value(json_value: serde_json::Value) -> prost_types::Value {
    let kind = match json_value {
        serde_json::Value::Null => Some(prost_types::value::Kind::NullValue(0)),
        serde_json::Value::Bool(b) => Some(prost_types::value::Kind::BoolValue(b)),
        serde_json::Value::Number(n) => Some(prost_types::value::Kind::NumberValue(
            n.as_f64().unwrap_or(0.0),
        )),
        serde_json::Value::String(s) => Some(prost_types::value::Kind::StringValue(s)),
        serde_json::Value::Array(arr) => Some(prost_types::value::Kind::ListValue(
            prost_types::ListValue {
                values: arr.into_iter().map(json_value_to_prost_value).collect(),
            },
        )),
        serde_json::Value::Object(obj) => {
            Some(prost_types::value::Kind::StructValue(prost_types::Struct {
                fields: obj
                    .into_iter()
                    .map(|(k, v)| (k, json_value_to_prost_value(v)))
                    .collect(),
            }))
        }
    };
    prost_types::Value { kind }
}

/// Generates a 64-bit hash from a string, ensuring the output is within JavaScript's safe integer range (0 to 2^53 - 1).
pub fn hash(s: &str) -> u64 {
    let mut cursor = Vec::new();
    cursor.extend_from_slice(s.as_bytes());
    let mut cursor = std::io::Cursor::new(cursor);
    let hash = murmur3::murmur3_x64_128(&mut cursor, 0).unwrap();
    (hash & 0x1F_FFFF_FFFF_FFFF) as u64
}

pub fn mailbox_id(account_id: u64, mailbox_name: &str) -> u64 {
    // Construct a buffer of bytes from account_id and mailbox_name
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&account_id.to_le_bytes()); // Convert u64 to bytes
    buffer.push(b':'); // Separator
    buffer.extend_from_slice(mailbox_name.as_bytes()); // Add mailbox name
                                                       // Create a Cursor for the buffer
    let mut cursor = std::io::Cursor::new(buffer);
    // Compute the 128-bit Murmur3 hash and cast to u64
    let hash = murmur3::murmur3_x64_128(&mut cursor, 0).unwrap();
    hash as u64 // Take lower 64 bits
}

pub fn envelope_hash(account_id: u64, mailbox_id: u64, uid: u32) -> u64 {
    let mut buffer = Vec::with_capacity(8 + 8 + 4); // Preallocate for efficiency
    buffer.extend_from_slice(&account_id.to_be_bytes());
    buffer.extend_from_slice(&mailbox_id.to_be_bytes());
    buffer.extend_from_slice(&uid.to_be_bytes());
    let mut cursor = std::io::Cursor::new(buffer);
    let hash = murmur3::murmur3_x64_128(&mut cursor, 0).unwrap();
    hash as u64
}

/// Generate a 64-bit hash for a GmailEnvelope using account_id, mailbox_id, and gmail api message id.
/// The `id` string is hashed to produce a consistent u64 value.
pub fn envelope_hash_from_id(account_id: u64, mailbox_id: u64, id: &str) -> u64 {
    let mut buffer = Vec::with_capacity(8 + 8 + id.len());
    buffer.extend_from_slice(&account_id.to_be_bytes());
    buffer.extend_from_slice(&mailbox_id.to_be_bytes());
    buffer.extend_from_slice(id.as_bytes());
    let mut cursor = std::io::Cursor::new(buffer);
    let hash128 = murmur3::murmur3_x64_128(&mut cursor, 0).unwrap();
    hash128 as u64
}
