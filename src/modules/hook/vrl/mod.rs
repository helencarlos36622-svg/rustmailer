// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::error::{code::ErrorCode, RustMailerResult},
    raise_error,
};
use payload::{Outcome, ResolveResult, VrlScriptTestRequest};
use std::collections::BTreeMap;
use vrl::{
    compiler::{compile_with_state, runtime::Runtime, CompileConfig, TargetValueRef},
    core::Value,
    diagnostic::Formatter,
    prelude::{TimeZone, TypeState},
    value::Secrets,
};

pub mod payload;

// The VRL resolution logic as an HTTP handler
pub async fn resolve_vrl_input(input: VrlScriptTestRequest) -> RustMailerResult<ResolveResult> {
    let outcome = resolve(input)?;
    let result = match outcome {
        Outcome::Success { output: _, result } => ResolveResult {
            result: Some(result.try_into().map_err(|_| {
                raise_error!(
                    "Failed to convert output to JSON".into(),
                    ErrorCode::InternalError
                )
            })?),
            error: None,
        },
        Outcome::Error(error) => ResolveResult {
            result: None,
            error: Some(error),
        },
    };
    Ok(result)
}

fn resolve(input: VrlScriptTestRequest) -> RustMailerResult<Outcome> {
    let event_str = input
        .event
        .as_deref()
        .ok_or_else(|| raise_error!("Missing event data".into(), ErrorCode::InvalidParameter))?;
    let json_value: Value = serde_json::from_str(event_str).map_err(|e| {
        raise_error!(
            format!("Input event is not a valid JSON string: {}", e),
            ErrorCode::InternalError
        )
    })?;
    let mut value = Value::try_from(json_value).map_err(|_| {
        raise_error!(
            "Input event is not a valid JSON string".into(),
            ErrorCode::InternalError
        )
    })?;

    let functions = vrl::stdlib::all();
    let state = TypeState::default();
    let mut runtime = Runtime::default();
    let config = CompileConfig::default();
    let timezone = TimeZone::default();

    let mut metadata = Value::Object(BTreeMap::new());
    let mut secrets = Secrets::new();

    let mut target_value = TargetValueRef {
        value: &mut value,
        metadata: &mut metadata,
        secrets: &mut secrets,
    };

    let program = match compile_with_state(&input.program, &functions, &state, config) {
        Ok(program) => program,
        Err(diagnostics) => {
            let msg = Formatter::new(&input.program, diagnostics).to_string();
            return Ok(Outcome::Error(msg));
        }
    };

    match runtime.resolve(&mut target_value, &program.program, &timezone) {
        Ok(result) => Ok(Outcome::Success {
            output: result,
            result: value.clone(),
        }),
        Err(err) => Ok(Outcome::Error(err.to_string())),
    }
}

pub fn compile_vrl_script(vrl_script: &str) -> RustMailerResult<()> {
    let functions = vrl::stdlib::all();
    let state = TypeState::default();
    let config = CompileConfig::default();
    match compile_with_state(vrl_script, &functions, &state, config) {
        Ok(_) => Ok(()),
        Err(diagnostics) => {
            let msg = Formatter::new(&vrl_script, diagnostics).to_string();
            Err(raise_error!(
                format!(
                    "VRL script contains syntax errors. Please fix before submission: {}",
                    msg
                ),
                ErrorCode::VRLScriptSyntaxError
            ))
        }
    }
}
