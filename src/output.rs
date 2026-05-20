use serde::Serialize;
use serde_json::json;

use crate::error::AppError;

#[derive(Debug, Serialize)]
struct SuccessEnvelope<'a, T> {
    ok: bool,
    command: &'a str,
    data: T,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
    ok: bool,
    command: &'a str,
    error: ErrorPayload<'a>,
}

#[derive(Debug, Serialize)]
struct ErrorPayload<'a> {
    code: &'a str,
    message: &'a str,
    relogin_required: bool,
    entitlement_denied: bool,
}

pub fn print_json_success<T>(command: &str, data: &T)
where
    T: Serialize + Clone,
{
    let envelope = SuccessEnvelope {
        ok: true,
        command,
        data: data.clone(),
    };
    println!(
        "{}",
        serde_json::to_string(&envelope).expect("serialize success envelope")
    );
}

pub fn print_json_error(command: &str, error: &AppError) {
    let envelope = ErrorEnvelope {
        ok: false,
        command,
        error: ErrorPayload {
            code: error.code.as_str(),
            message: &error.message,
            relogin_required: error.relogin_required,
            entitlement_denied: error.entitlement_denied,
        },
    };
    println!(
        "{}",
        serde_json::to_string(&envelope).expect("serialize error envelope")
    );
}

pub fn print_human_error(command: &str, error: &AppError) {
    eprintln!("{command}: {} ({})", error.message, error.code);
}

pub fn print_pretty_json(value: serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!(value)).expect("serialize pretty json")
    );
}
