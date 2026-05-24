use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::app::AppContext;
use crate::auth::resolver::{
    RuntimeCredentialOptions, resolve_runtime_credentials, resolve_runtime_credentials_with_options,
};
use crate::error::{AppError, ErrorCode};
use crate::usage::model::{RateLimitCaptureBucket, RateLimitsCapture};

const DEFAULT_RESPONSES_PATH: &str = "/responses";
const DEFAULT_RESPONSES_RETRIES: usize = 2;
pub const DEFAULT_TEXT_TIMEOUT_SECONDS: u64 = 3_600;
pub const DEFAULT_MEDIA_TIMEOUT_SECONDS: u64 = 120;

#[derive(Debug, Clone, Copy, Default)]
pub struct UpstreamAuthOptions {
    pub refresh_if_expiring: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpstreamResponseEnvelope {
    pub credential_source: String,
    pub response: Value,
    pub usage: ResponseUsageSummary,
    pub rate_limits: Option<RateLimitsCapture>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpstreamJsonEnvelope {
    pub credential_source: String,
    pub response: Value,
    pub usage: ResponseUsageSummary,
    pub rate_limits: Option<RateLimitsCapture>,
}

#[derive(Debug, Clone)]
pub struct UpstreamBytesEnvelope {
    pub credential_source: String,
    pub bytes: Vec<u8>,
    pub rate_limits: Option<RateLimitsCapture>,
}

#[derive(Debug)]
pub struct UpstreamStreamEnvelope {
    pub credential_source: String,
    pub response: reqwest::blocking::Response,
    pub rate_limits: Option<RateLimitsCapture>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ResponseUsageSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct OAuthErrorResponse {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

pub fn post_responses_api(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    body: &Value,
    timeout_seconds: Option<u64>,
) -> Result<UpstreamResponseEnvelope, AppError> {
    let credentials = resolve_runtime_credentials(ctx, auth_file)?;
    let endpoint = format!(
        "{}{}",
        credentials.base_url.trim_end_matches("/"),
        DEFAULT_RESPONSES_PATH
    );

    let response = send_responses_request_with_retry(
        ctx,
        &endpoint,
        &credentials.token_type,
        &credentials.access_token,
        "application/json",
        body,
        timeout_seconds,
    )
    .map_err(|error| map_transport_error("responses request", &error))?;

    if response.status().is_success() {
        let rate_limits = extract_rate_limits(response.headers(), &credentials.provider);
        let response_json = response.json::<Value>().map_err(|error| {
            AppError::new(
                ErrorCode::ResponseDecodeFailed,
                format!("failed to decode responses API payload: {error}"),
            )
        })?;
        let usage = extract_usage_summary(&response_json);
        return Ok(UpstreamResponseEnvelope {
            credential_source: credentials.provider,
            response: response_json,
            usage,
            rate_limits,
        });
    }

    let retry_after_seconds = parse_retry_after_seconds(response.headers());
    let status = response.status();
    let body_text = response.text().unwrap_or_default();
    let detail = parse_error_detail(&body_text);

    let error_context = UpstreamErrorContext::new("responses request", status, &detail)
        .with_retry_after_seconds(retry_after_seconds);
    if let Some(error) = classify_upstream_error(error_context) {
        return Err(error);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        format!(
            "responses request failed with status {status}: {}",
            if detail.is_empty() {
                json!({"body": body_text}).to_string()
            } else {
                detail
            }
        ),
    ))
}

pub fn post_responses_stream_api(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    body: &Value,
    timeout_seconds: Option<u64>,
) -> Result<UpstreamStreamEnvelope, AppError> {
    let credentials = resolve_runtime_credentials(ctx, auth_file)?;
    let endpoint = format!(
        "{}{}",
        credentials.base_url.trim_end_matches("/"),
        DEFAULT_RESPONSES_PATH
    );

    let response = send_responses_request_with_retry(
        ctx,
        &endpoint,
        &credentials.token_type,
        &credentials.access_token,
        "text/event-stream",
        body,
        timeout_seconds,
    )
    .map_err(|error| map_transport_error("responses stream request", &error))?;

    if response.status().is_success() {
        let rate_limits = extract_rate_limits(response.headers(), &credentials.provider);
        return Ok(UpstreamStreamEnvelope {
            credential_source: credentials.provider,
            response,
            rate_limits,
        });
    }

    let retry_after_seconds = parse_retry_after_seconds(response.headers());
    let status = response.status();
    let body_text = response.text().unwrap_or_default();
    let detail = parse_error_detail(&body_text);

    let error_context = UpstreamErrorContext::new("responses stream request", status, &detail)
        .with_retry_after_seconds(retry_after_seconds);
    if let Some(error) = classify_upstream_error(error_context) {
        return Err(error);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        format!(
            "responses stream request failed with status {status}: {}",
            if detail.is_empty() {
                json!({"body": body_text}).to_string()
            } else {
                detail
            }
        ),
    ))
}

fn send_responses_request_with_retry(
    ctx: &AppContext,
    endpoint: &str,
    token_type: &str,
    access_token: &str,
    accept: &str,
    body: &Value,
    timeout_seconds: Option<u64>,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let timeout =
        std::time::Duration::from_secs(timeout_seconds.unwrap_or(DEFAULT_TEXT_TIMEOUT_SECONDS));
    let mut last_error = None;

    for attempt in 0..=DEFAULT_RESPONSES_RETRIES {
        let response = ctx
            .http_client
            .post(endpoint)
            .header(reqwest::header::ACCEPT, accept)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("{token_type} {access_token}"),
            )
            .timeout(timeout)
            .json(body)
            .send();

        match response {
            Ok(response) => {
                let status = response.status();
                if status.is_server_error() && attempt < DEFAULT_RESPONSES_RETRIES {
                    std::thread::sleep(std::time::Duration::from_millis(
                        300 * (attempt as u64 + 1),
                    ));
                    continue;
                }
                return Ok(response);
            }
            Err(error) => {
                if should_retry_transport_error(&error) && attempt < DEFAULT_RESPONSES_RETRIES {
                    last_error = Some(error);
                    std::thread::sleep(std::time::Duration::from_millis(
                        300 * (attempt as u64 + 1),
                    ));
                    continue;
                }
                return Err(error);
            }
        }
    }

    match last_error {
        Some(error) => Err(error),
        None => unreachable!("responses retry loop should return before exhausting without error"),
    }
}

fn should_retry_transport_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request()
}

pub(crate) fn map_transport_error(operation: &str, error: &reqwest::Error) -> AppError {
    let code = if error.is_timeout() {
        ErrorCode::NetworkTimeout
    } else if error.is_connect() {
        ErrorCode::NetworkConnectFailed
    } else {
        ErrorCode::NetworkTransportFailed
    };
    AppError::new(code, format!("{operation} failed: {error}"))
}

pub fn post_json_api_with_options(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    endpoint_path: &str,
    body: &Value,
    timeout_seconds: Option<u64>,
    auth_options: UpstreamAuthOptions,
) -> Result<UpstreamJsonEnvelope, AppError> {
    let credentials = resolve_runtime_credentials_with_options(
        ctx,
        auth_file,
        RuntimeCredentialOptions {
            refresh_if_expiring: auth_options.refresh_if_expiring,
        },
    )?;
    let endpoint = format!(
        "{}{}",
        credentials.base_url.trim_end_matches("/"),
        endpoint_path
    );

    let response = ctx
        .http_client
        .post(&endpoint)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{} {}", credentials.token_type, credentials.access_token),
        )
        .timeout(std::time::Duration::from_secs(
            timeout_seconds.unwrap_or(DEFAULT_MEDIA_TIMEOUT_SECONDS),
        ))
        .json(body)
        .send()
        .map_err(|error| map_transport_error(endpoint_path, &error))?;

    map_json_response(response, credentials.provider, endpoint_path)
}

pub fn get_json_api_with_options(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    endpoint_path: &str,
    timeout_seconds: Option<u64>,
    auth_options: UpstreamAuthOptions,
) -> Result<UpstreamJsonEnvelope, AppError> {
    let credentials = resolve_runtime_credentials_with_options(
        ctx,
        auth_file,
        RuntimeCredentialOptions {
            refresh_if_expiring: auth_options.refresh_if_expiring,
        },
    )?;
    let endpoint = format!(
        "{}{}",
        credentials.base_url.trim_end_matches("/"),
        endpoint_path
    );

    let response = ctx
        .http_client
        .get(&endpoint)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{} {}", credentials.token_type, credentials.access_token),
        )
        .timeout(std::time::Duration::from_secs(
            timeout_seconds.unwrap_or(DEFAULT_MEDIA_TIMEOUT_SECONDS),
        ))
        .send()
        .map_err(|error| map_transport_error(endpoint_path, &error))?;

    map_json_response(response, credentials.provider, endpoint_path)
}

pub fn post_bytes_api_with_options(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    endpoint_path: &str,
    body: &Value,
    timeout_seconds: Option<u64>,
    auth_options: UpstreamAuthOptions,
) -> Result<UpstreamBytesEnvelope, AppError> {
    let credentials = resolve_runtime_credentials_with_options(
        ctx,
        auth_file,
        RuntimeCredentialOptions {
            refresh_if_expiring: auth_options.refresh_if_expiring,
        },
    )?;
    let endpoint = format!(
        "{}{}",
        credentials.base_url.trim_end_matches("/"),
        endpoint_path
    );

    let response = ctx
        .http_client
        .post(&endpoint)
        .header(reqwest::header::ACCEPT, "*/*")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{} {}", credentials.token_type, credentials.access_token),
        )
        .timeout(std::time::Duration::from_secs(
            timeout_seconds.unwrap_or(DEFAULT_MEDIA_TIMEOUT_SECONDS),
        ))
        .json(body)
        .send()
        .map_err(|error| map_transport_error(endpoint_path, &error))?;

    map_bytes_response(response, credentials.provider, endpoint_path)
}

pub fn post_multipart_api_with_options(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    endpoint_path: &str,
    form: reqwest::blocking::multipart::Form,
    timeout_seconds: Option<u64>,
    auth_options: UpstreamAuthOptions,
) -> Result<UpstreamJsonEnvelope, AppError> {
    let credentials = resolve_runtime_credentials_with_options(
        ctx,
        auth_file,
        RuntimeCredentialOptions {
            refresh_if_expiring: auth_options.refresh_if_expiring,
        },
    )?;
    let endpoint = format!(
        "{}{}",
        credentials.base_url.trim_end_matches("/"),
        endpoint_path
    );

    let response = ctx
        .http_client
        .post(&endpoint)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{} {}", credentials.token_type, credentials.access_token),
        )
        .timeout(std::time::Duration::from_secs(
            timeout_seconds.unwrap_or(DEFAULT_MEDIA_TIMEOUT_SECONDS),
        ))
        .multipart(form)
        .send()
        .map_err(|error| map_transport_error(endpoint_path, &error))?;

    map_json_response(response, credentials.provider, endpoint_path)
}

fn map_json_response(
    response: reqwest::blocking::Response,
    credential_source: String,
    endpoint_path: &str,
) -> Result<UpstreamJsonEnvelope, AppError> {
    if response.status().is_success() {
        let rate_limits = extract_rate_limits(response.headers(), &credential_source);
        let response_json = response.json::<Value>().map_err(|error| {
            AppError::new(
                ErrorCode::ResponseDecodeFailed,
                format!("failed to decode {endpoint_path} payload: {error}"),
            )
        })?;
        let usage = extract_usage_summary(&response_json);
        return Ok(UpstreamJsonEnvelope {
            credential_source,
            response: response_json,
            usage,
            rate_limits,
        });
    }

    let retry_after_seconds = parse_retry_after_seconds(response.headers());
    let status = response.status();
    let body_text = response.text().unwrap_or_default();
    let detail = parse_error_detail(&body_text);

    let error_context = UpstreamErrorContext::new(endpoint_path, status, &detail)
        .with_retry_after_seconds(retry_after_seconds);
    if let Some(error) = classify_upstream_error(error_context) {
        return Err(error);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        format!(
            "{endpoint_path} failed with status {status}: {}",
            if detail.is_empty() {
                json!({"body": body_text}).to_string()
            } else {
                detail
            }
        ),
    ))
}

fn map_bytes_response(
    response: reqwest::blocking::Response,
    credential_source: String,
    endpoint_path: &str,
) -> Result<UpstreamBytesEnvelope, AppError> {
    if response.status().is_success() {
        let rate_limits = extract_rate_limits(response.headers(), &credential_source);
        let bytes = response.bytes().map_err(|error| {
            AppError::new(
                ErrorCode::NetworkTransportFailed,
                format!("failed to read {endpoint_path} bytes payload: {error}"),
            )
        })?;
        return Ok(UpstreamBytesEnvelope {
            credential_source,
            bytes: bytes.to_vec(),
            rate_limits,
        });
    }

    let retry_after_seconds = parse_retry_after_seconds(response.headers());
    let status = response.status();
    let body_text = response.text().unwrap_or_default();
    let detail = parse_error_detail(&body_text);

    let error_context = UpstreamErrorContext::new(endpoint_path, status, &detail)
        .with_retry_after_seconds(retry_after_seconds);
    if let Some(error) = classify_upstream_error(error_context) {
        return Err(error);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        format!(
            "{endpoint_path} failed with status {status}: {}",
            if detail.is_empty() {
                json!({"body": body_text}).to_string()
            } else {
                detail
            }
        ),
    ))
}

fn parse_error_detail(body_text: &str) -> String {
    let payload =
        serde_json::from_str::<OAuthErrorResponse>(body_text).unwrap_or(OAuthErrorResponse {
            error: None,
            error_description: None,
            message: None,
        });
    payload
        .error_description
        .or(payload.message)
        .or(payload.error)
        .unwrap_or_else(|| body_text.to_string())
}

struct UpstreamErrorContext<'a> {
    operation: &'a str,
    status: reqwest::StatusCode,
    detail: &'a str,
    retry_after_seconds: Option<u64>,
}

impl<'a> UpstreamErrorContext<'a> {
    fn new(operation: &'a str, status: reqwest::StatusCode, detail: &'a str) -> Self {
        Self {
            operation,
            status,
            detail,
            retry_after_seconds: None,
        }
    }

    fn with_retry_after_seconds(mut self, retry_after_seconds: Option<u64>) -> Self {
        self.retry_after_seconds = retry_after_seconds;
        self
    }

    fn message(&self, kind: &str) -> String {
        format!("{} {kind}: {}", self.operation, self.detail)
    }
}

fn classify_upstream_error(ctx: UpstreamErrorContext<'_>) -> Option<AppError> {
    let lower = ctx.detail.to_lowercase();
    if lower.contains("bad-credentials")
        || lower.contains("access token could not be validated")
        || lower.contains("token could not be validated")
        || lower.contains("invalid token")
        || lower.contains("expired token")
        || lower.contains("unauthenticated")
        || lower.contains("incorrect api key provided")
    {
        return Some(AppError::new(
            ErrorCode::AuthExpired,
            ctx.message("auth failed"),
        ));
    }

    if ctx.status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        if has_quota_signal(&lower) {
            return Some(AppError::new(
                ErrorCode::QuotaExhausted,
                ctx.message("quota exhausted"),
            ));
        }
        let mut error = AppError::new(ErrorCode::RateLimited, ctx.message("rate limited"))
            .with_retry_after_seconds(ctx.retry_after_seconds);
        if ctx.retry_after_seconds.is_some() {
            error.retryable = true;
            error.recovery_action = crate::error::RecoveryAction::WaitThenRetry;
        }
        return Some(error);
    }

    if ctx.status == reqwest::StatusCode::PAYMENT_REQUIRED || has_billing_signal(&lower) {
        return Some(AppError::new(
            ErrorCode::BillingRequired,
            ctx.message("billing failed"),
        ));
    }

    if has_quota_signal(&lower) {
        return Some(AppError::new(
            ErrorCode::QuotaExhausted,
            ctx.message("quota exhausted"),
        ));
    }

    if ctx.status == reqwest::StatusCode::FORBIDDEN {
        return Some(AppError::new(
            ErrorCode::XaiOauthTierDenied,
            ctx.message("was denied"),
        ));
    }

    None
}

fn has_billing_signal(lower: &str) -> bool {
    lower.contains("billing")
        || lower.contains("payment required")
        || lower.contains("payment_required")
        || lower.contains("insufficient funds")
        || lower.contains("insufficient_funds")
        || lower.contains("insufficient balance")
        || lower.contains("balance")
        || lower.contains("credits")
        || lower.contains("credit")
        || lower.contains("spend cap")
        || lower.contains("spend_cap")
}

fn has_quota_signal(lower: &str) -> bool {
    lower.contains("quota")
        || lower.contains("insufficient_quota")
        || lower.contains("quota_exceeded")
        || lower.contains("quota exceeded")
        || lower.contains("usage limit")
        || lower.contains("usage_limit")
}

fn parse_retry_after_seconds(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
}

fn extract_usage_summary(response: &Value) -> ResponseUsageSummary {
    let usage = response.get("usage").cloned().unwrap_or(Value::Null);
    let input_total = usage
        .get("input_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("prompt_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let output_tokens = usage
        .get("output_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("completion_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let input_details = usage
        .get("input_tokens_details")
        .or_else(|| usage.get("prompt_tokens_details"));
    let output_details = usage.get("output_tokens_details");
    let cache_read_tokens = input_details
        .and_then(|details| details.get("cached_tokens"))
        .and_then(Value::as_u64)
        .or_else(|| usage.get("cache_read_input_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let cache_write_tokens = input_details
        .and_then(|details| details.get("cache_creation_tokens"))
        .and_then(Value::as_u64)
        .or_else(|| {
            input_details
                .and_then(|details| details.get("cache_write_tokens"))
                .and_then(Value::as_u64)
        })
        .or_else(|| {
            usage
                .get("cache_creation_input_tokens")
                .and_then(Value::as_u64)
        })
        .unwrap_or(0);
    let reasoning_tokens = output_details
        .and_then(|details| details.get("reasoning_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let input_tokens = input_total.saturating_sub(cache_read_tokens + cache_write_tokens);

    ResponseUsageSummary {
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
        reasoning_tokens,
    }
}

fn extract_rate_limits(
    headers: &reqwest::header::HeaderMap,
    provider: &str,
) -> Option<RateLimitsCapture> {
    let captured_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    let rpm = parse_rate_limit_bucket(headers, "requests", "");
    let rph = parse_rate_limit_bucket(headers, "requests", "-1h");
    let tpm = parse_rate_limit_bucket(headers, "tokens", "");
    let tph = parse_rate_limit_bucket(headers, "tokens", "-1h");

    if rpm.is_none() && rph.is_none() && tpm.is_none() && tph.is_none() {
        return None;
    }

    Some(RateLimitsCapture {
        captured_at,
        provider: provider.to_string(),
        requests_per_minute: rpm,
        requests_per_hour: rph,
        tokens_per_minute: tpm,
        tokens_per_hour: tph,
    })
}

fn parse_rate_limit_bucket(
    headers: &reqwest::header::HeaderMap,
    resource: &str,
    suffix: &str,
) -> Option<RateLimitCaptureBucket> {
    let limit_key = format!("x-ratelimit-limit-{resource}{suffix}");
    let remaining_key = format!("x-ratelimit-remaining-{resource}{suffix}");
    let reset_key = format!("x-ratelimit-reset-{resource}{suffix}");

    let limit = header_u64(headers, &limit_key)?;
    let remaining = header_u64(headers, &remaining_key).unwrap_or(0);
    let reset_seconds = header_u64(headers, &reset_key).unwrap_or(0);

    Some(RateLimitCaptureBucket {
        limit,
        remaining,
        reset_seconds,
    })
}

fn header_u64(headers: &reqwest::header::HeaderMap, key: &str) -> Option<u64> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_MEDIA_TIMEOUT_SECONDS, DEFAULT_TEXT_TIMEOUT_SECONDS, UpstreamErrorContext,
        classify_upstream_error, parse_error_detail,
    };
    use crate::error::{ErrorCategory, ErrorCode, RecoveryAction};

    #[test]
    fn timeout_defaults_match_cli_policy() {
        assert_eq!(DEFAULT_TEXT_TIMEOUT_SECONDS, 3_600);
        assert_eq!(DEFAULT_MEDIA_TIMEOUT_SECONDS, 120);
    }

    #[test]
    fn classifier_prefers_credentials_over_forbidden_shape() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "responses request",
            reqwest::StatusCode::FORBIDDEN,
            "The OAuth2 access token could not be validated. [WKE=unauthenticated:bad-credentials]",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::AuthExpired);
        assert_eq!(error.category, ErrorCategory::AuthRefreshable);
        assert_eq!(error.recovery_action, RecoveryAction::RefreshThenRetry);
        assert!(error.retryable);
        assert!(!error.entitlement_denied);
    }

    #[test]
    fn classifier_maps_incorrect_api_key_to_refreshable_auth() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "responses request",
            reqwest::StatusCode::BAD_REQUEST,
            "Incorrect API key provided: de***st. You can obtain an API key from https://console.x.ai.",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::AuthExpired);
        assert_eq!(error.category, ErrorCategory::AuthRefreshable);
        assert_eq!(error.recovery_action, RecoveryAction::RefreshThenRetry);
        assert!(error.retryable);
        assert!(!error.entitlement_denied);
    }

    #[test]
    fn classifier_maps_payment_required_to_billing_stop() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "/videos/generations",
            reqwest::StatusCode::PAYMENT_REQUIRED,
            "Billing required: insufficient credits",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::BillingRequired);
        assert_eq!(error.category, ErrorCategory::BillingRequired);
        assert_eq!(error.recovery_action, RecoveryAction::StopBilling);
        assert!(error.billing_required);
        assert!(!error.retryable);
    }

    #[test]
    fn classifier_maps_quota_signal_to_quota_stop() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "responses request",
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            "insufficient_quota",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::QuotaExhausted);
        assert_eq!(error.category, ErrorCategory::QuotaExhausted);
        assert_eq!(error.recovery_action, RecoveryAction::StopQuota);
        assert!(error.quota_exhausted);
        assert!(!error.retryable);
    }

    #[test]
    fn classifier_maps_retry_after_rate_limit_to_wait_retry() {
        let error = classify_upstream_error(
            UpstreamErrorContext::new(
                "responses request",
                reqwest::StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
            )
            .with_retry_after_seconds(Some(12)),
        )
        .unwrap();

        assert_eq!(error.code, ErrorCode::RateLimited);
        assert_eq!(error.category, ErrorCategory::RateLimited);
        assert_eq!(error.recovery_action, RecoveryAction::WaitThenRetry);
        assert_eq!(error.retry_after_seconds, Some(12));
        assert!(error.rate_limited);
        assert!(error.retryable);
    }

    #[test]
    fn classifier_maps_rate_limit_without_retry_after_to_stop_rate_limit() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "responses request",
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_exceeded",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::RateLimited);
        assert_eq!(error.category, ErrorCategory::RateLimited);
        assert_eq!(error.recovery_action, RecoveryAction::StopRateLimit);
        assert_eq!(error.retry_after_seconds, None);
        assert!(error.rate_limited);
        assert!(!error.retryable);
    }

    #[test]
    fn classifier_maps_non_429_quota_signal_to_quota_stop() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "/images/generations",
            reqwest::StatusCode::FORBIDDEN,
            "usage limit exceeded",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::QuotaExhausted);
        assert_eq!(error.category, ErrorCategory::QuotaExhausted);
        assert_eq!(error.recovery_action, RecoveryAction::StopQuota);
        assert!(error.quota_exhausted);
        assert!(!error.entitlement_denied);
    }

    #[test]
    fn classifier_maps_billing_keywords_without_402_to_billing_stop() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "/videos/generations",
            reqwest::StatusCode::FORBIDDEN,
            "Spend cap reached for account balance",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::BillingRequired);
        assert_eq!(error.category, ErrorCategory::BillingRequired);
        assert_eq!(error.recovery_action, RecoveryAction::StopBilling);
        assert!(error.billing_required);
        assert!(!error.entitlement_denied);
    }

    #[test]
    fn classifier_maps_forbidden_to_entitlement_stop() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "responses request",
            reqwest::StatusCode::FORBIDDEN,
            "tier access denied",
        ))
        .unwrap();

        assert_eq!(error.code, ErrorCode::XaiOauthTierDenied);
        assert_eq!(error.category, ErrorCategory::EntitlementDenied);
        assert_eq!(error.recovery_action, RecoveryAction::StopEntitlement);
        assert!(error.entitlement_denied);
        assert!(!error.retryable);
    }

    #[test]
    fn classifier_returns_none_for_unclassified_server_errors() {
        let error = classify_upstream_error(UpstreamErrorContext::new(
            "responses request",
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "temporary upstream failure",
        ));

        assert!(error.is_none());
    }

    #[test]
    fn parse_error_detail_prefers_description_then_message_then_error_then_raw_body() {
        assert_eq!(
            parse_error_detail(
                r#"{"error":"outer","message":"message text","error_description":"description text"}"#
            ),
            "description text"
        );
        assert_eq!(
            parse_error_detail(r#"{"error":"outer","message":"message text"}"#),
            "message text"
        );
        assert_eq!(parse_error_detail(r#"{"error":"outer"}"#), "outer");
        assert_eq!(parse_error_detail("plain text body"), "plain text body");
    }
}
