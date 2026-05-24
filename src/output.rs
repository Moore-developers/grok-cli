use serde::Serialize;
use serde_json::json;

use crate::error::{AppError, ErrorCode};

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
    match serde_json::to_string(&envelope) {
        Ok(serialized) => println!("{serialized}"),
        Err(error) => print_json_error(
            command,
            &AppError::new(
                ErrorCode::OutputSerializationFailed,
                format!("failed to serialize success envelope: {error}"),
            ),
        ),
    }
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
    println!("{}", serialize_error_envelope(&envelope));
}

pub fn print_human_error(command: &str, error: &AppError) {
    eprintln!("{command}: {} ({})", error.message, error.code);
}

fn serialize_error_envelope(envelope: &ErrorEnvelope<'_>) -> String {
    serde_json::to_string(envelope).unwrap_or_else(|_| {
        let command = escape_json_string(envelope.command);
        r#"{"ok":false,"command":"#
            .to_string()
            + &command
            + r#","error":{"code":"output_serialization_failed","message":"failed to serialize error envelope","relogin_required":false,"entitlement_denied":false,"category":"request_failed","recovery_action":"stop_unknown","retryable":false,"retry_after_seconds":null,"billing_required":false,"quota_exhausted":false,"rate_limited":false}}"#
    })
}

fn escape_json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"unknown\"".to_string())
}

pub fn print_pretty_json(value: serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!(value)).expect("serialize pretty json")
    );
}

#[cfg(test)]
mod tests {
    use super::{ErrorEnvelope, ErrorPayload, serialize_error_envelope};
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

    #[test]
    fn serialization_failure_error_uses_failure_code_not_success_envelope() {
        let error = AppError::new(
            ErrorCode::OutputSerializationFailed,
            "failed to serialize success envelope",
        );
        let envelope = ErrorEnvelope {
            ok: false,
            command: "chat",
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

        let serialized = serialize_error_envelope(&envelope);
        let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["command"], "chat");
        assert_eq!(json["error"]["code"], "output_serialization_failed");
        assert_eq!(json["error"]["category"], "request_failed");
    }
}
