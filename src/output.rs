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
    category: &'a str,
    recovery_action: &'a str,
    retryable: bool,
    retry_after_seconds: Option<u64>,
    billing_required: bool,
    quota_exhausted: bool,
    rate_limited: bool,
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
        serde_json::to_string(&envelope).unwrap_or_else(|_| {
            serde_json::to_string(&serde_json::json!({
                "ok": true, "command": command, "data": null
            }))
            .unwrap_or_else(|_| r#"{"ok":true,"command":"unknown","data":null}"#.to_string())
        })
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
            category: error.category.as_str(),
            recovery_action: error.recovery_action.as_str(),
            retryable: error.retryable,
            retry_after_seconds: error.retry_after_seconds,
            billing_required: error.billing_required,
            quota_exhausted: error.quota_exhausted,
            rate_limited: error.rate_limited,
        },
    };
    println!(
        "{}",
        serde_json::to_string(&envelope).unwrap_or_else(|_| {
            serde_json::to_string(&serde_json::json!({
                "ok": false, "command": command, "error": {"code": "serialization_failed"}
            }))
            .unwrap_or_else(|_| r#"{"ok":false,"command":"unknown","error":{"code":"serialization_failed"}}"#.to_string())
        })
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

#[cfg(test)]
mod tests {
    use super::{ErrorEnvelope, ErrorPayload};
    use crate::error::{AppError, ErrorCode, RecoveryAction};

    #[test]
    fn error_payload_serializes_structured_recovery_fields() {
        let mut error = AppError::new(ErrorCode::RateLimited, "too many requests")
            .with_retry_after_seconds(Some(7));
        error.retryable = true;
        error.recovery_action = RecoveryAction::WaitThenRetry;

        let envelope = ErrorEnvelope {
            ok: false,
            command: "search",
            error: ErrorPayload {
                code: error.code.as_str(),
                message: &error.message,
                relogin_required: error.relogin_required,
                entitlement_denied: error.entitlement_denied,
                category: error.category.as_str(),
                recovery_action: error.recovery_action.as_str(),
                retryable: error.retryable,
                retry_after_seconds: error.retry_after_seconds,
                billing_required: error.billing_required,
                quota_exhausted: error.quota_exhausted,
                rate_limited: error.rate_limited,
            },
        };

        let serialized = serde_json::to_value(envelope).unwrap();
        assert_eq!(serialized["ok"], false);
        assert_eq!(serialized["command"], "search");
        assert_eq!(serialized["error"]["code"], "rate_limited");
        assert_eq!(serialized["error"]["category"], "rate_limited");
        assert_eq!(serialized["error"]["recovery_action"], "wait_then_retry");
        assert_eq!(serialized["error"]["retryable"], true);
        assert_eq!(serialized["error"]["retry_after_seconds"], 7);
        assert_eq!(serialized["error"]["rate_limited"], true);
        assert_eq!(serialized["error"]["billing_required"], false);
        assert_eq!(serialized["error"]["quota_exhausted"], false);
    }
}
