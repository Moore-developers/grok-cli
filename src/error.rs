use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidArgs,
    IoError,
    StateFileMissing,
    StateFileInvalid,
    AuthMissing,
    AuthExpired,
    AuthRefreshFailed,
    AuthReloginRequired,
    AuthStateMismatch,
    AuthCallbackTimeout,
    AuthTokenExchangeFailed,
    XaiOauthTierDenied,
    BillingRequired,
    QuotaExhausted,
    RateLimited,
    ModelCapabilityMismatch,
    RequestFailed,
    NotImplemented,
    NetworkTransportError,
    IOBufferOverflow,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgs => "invalid_args",
            Self::IoError => "io_error",
            Self::StateFileMissing => "state_file_missing",
            Self::StateFileInvalid => "state_file_invalid",
            Self::AuthMissing => "auth_missing",
            Self::AuthExpired => "auth_expired",
            Self::AuthRefreshFailed => "auth_refresh_failed",
            Self::AuthReloginRequired => "auth_relogin_required",
            Self::AuthStateMismatch => "auth_state_mismatch",
            Self::AuthCallbackTimeout => "auth_callback_timeout",
            Self::AuthTokenExchangeFailed => "auth_token_exchange_failed",
            Self::XaiOauthTierDenied => "xai_oauth_tier_denied",
            Self::BillingRequired => "billing_required",
            Self::QuotaExhausted => "quota_exhausted",
            Self::RateLimited => "rate_limited",
            Self::ModelCapabilityMismatch => "model_capability_mismatch",
            Self::RequestFailed => "request_failed",
            Self::NotImplemented => "not_implemented",
            Self::NetworkTransportError => "network_transport_error",
            Self::IOBufferOverflow => "io_buffer_overflow",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    InvalidRequest,
    LocalState,
    AuthRefreshable,
    AuthReloginRequired,
    BillingRequired,
    QuotaExhausted,
    RateLimited,
    EntitlementDenied,
    CapabilityMismatch,
    RequestFailed,
    NotImplemented,
}

impl ErrorCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::LocalState => "local_state",
            Self::AuthRefreshable => "auth_refreshable",
            Self::AuthReloginRequired => "auth_relogin_required",
            Self::BillingRequired => "billing_required",
            Self::QuotaExhausted => "quota_exhausted",
            Self::RateLimited => "rate_limited",
            Self::EntitlementDenied => "entitlement_denied",
            Self::CapabilityMismatch => "capability_mismatch",
            Self::RequestFailed => "request_failed",
            Self::NotImplemented => "not_implemented",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    FixArgsThenRetry,
    RefreshThenRetry,
    LoginThenRetry,
    WaitThenRetry,
    StopBilling,
    StopQuota,
    StopRateLimit,
    StopEntitlement,
    StopUnknown,
}

impl RecoveryAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FixArgsThenRetry => "fix_args_then_retry",
            Self::RefreshThenRetry => "refresh_then_retry",
            Self::LoginThenRetry => "login_then_retry",
            Self::WaitThenRetry => "wait_then_retry",
            Self::StopBilling => "stop_billing",
            Self::StopQuota => "stop_quota",
            Self::StopRateLimit => "stop_rate_limit",
            Self::StopEntitlement => "stop_entitlement",
            Self::StopUnknown => "stop_unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub relogin_required: bool,
    pub entitlement_denied: bool,
    pub category: ErrorCategory,
    pub recovery_action: RecoveryAction,
    pub retryable: bool,
    pub retry_after_seconds: Option<u64>,
    pub billing_required: bool,
    pub quota_exhausted: bool,
    pub rate_limited: bool,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let code_value = code;
        let (category, recovery_action, retryable) = default_recovery(code_value);
        Self {
            code: code_value,
            message: message.into(),
            relogin_required: matches!(code_value, ErrorCode::AuthReloginRequired),
            entitlement_denied: matches!(code_value, ErrorCode::XaiOauthTierDenied),
            category,
            recovery_action,
            retryable,
            retry_after_seconds: None,
            billing_required: matches!(code_value, ErrorCode::BillingRequired),
            quota_exhausted: matches!(code_value, ErrorCode::QuotaExhausted),
            rate_limited: matches!(code_value, ErrorCode::RateLimited),
        }
    }

    pub fn with_retry_after_seconds(mut self, retry_after_seconds: Option<u64>) -> Self {
        self.retry_after_seconds = retry_after_seconds;
        self
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::IoError, message)
    }

    pub fn state_file_missing(path: &std::path::Path) -> Self {
        Self::new(
            ErrorCode::StateFileMissing,
            format!("state file not found: {}", path.display()),
        )
    }

    pub fn state_file_invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::StateFileInvalid, message)
    }

    pub fn exit_code(&self) -> i32 {
        match self.code {
            ErrorCode::InvalidArgs => 2,
            ErrorCode::AuthMissing | ErrorCode::AuthExpired | ErrorCode::AuthReloginRequired => 3,
            ErrorCode::XaiOauthTierDenied
            | ErrorCode::BillingRequired
            | ErrorCode::QuotaExhausted
            | ErrorCode::RateLimited => 4,
            ErrorCode::ModelCapabilityMismatch => 5,
            _ => 1,
        }
    }
}

fn default_recovery(code: ErrorCode) -> (ErrorCategory, RecoveryAction, bool) {
    match code {
        ErrorCode::InvalidArgs => (
            ErrorCategory::InvalidRequest,
            RecoveryAction::FixArgsThenRetry,
            true,
        ),
        ErrorCode::StateFileMissing => (
            ErrorCategory::AuthReloginRequired,
            RecoveryAction::LoginThenRetry,
            true,
        ),
        ErrorCode::StateFileInvalid
        | ErrorCode::IoError
        | ErrorCode::AuthRefreshFailed
        | ErrorCode::AuthStateMismatch
        | ErrorCode::AuthCallbackTimeout
        | ErrorCode::AuthTokenExchangeFailed => (
            ErrorCategory::LocalState,
            RecoveryAction::StopUnknown,
            false,
        ),
        ErrorCode::AuthMissing | ErrorCode::AuthExpired => (
            ErrorCategory::AuthRefreshable,
            RecoveryAction::RefreshThenRetry,
            true,
        ),
        ErrorCode::AuthReloginRequired => (
            ErrorCategory::AuthReloginRequired,
            RecoveryAction::LoginThenRetry,
            true,
        ),
        ErrorCode::XaiOauthTierDenied => (
            ErrorCategory::EntitlementDenied,
            RecoveryAction::StopEntitlement,
            false,
        ),
        ErrorCode::BillingRequired => (
            ErrorCategory::BillingRequired,
            RecoveryAction::StopBilling,
            false,
        ),
        ErrorCode::QuotaExhausted => (
            ErrorCategory::QuotaExhausted,
            RecoveryAction::StopQuota,
            false,
        ),
        ErrorCode::RateLimited
        | ErrorCode::NetworkTransportError
        | ErrorCode::IOBufferOverflow => (
            ErrorCategory::RateLimited,
            RecoveryAction::StopRateLimit,
            false,
        ),
        ErrorCode::ModelCapabilityMismatch => (
            ErrorCategory::CapabilityMismatch,
            RecoveryAction::StopUnknown,
            false,
        ),
        ErrorCode::RequestFailed => (
            ErrorCategory::RequestFailed,
            RecoveryAction::StopUnknown,
            false,
        ),
        ErrorCode::NotImplemented => (
            ErrorCategory::NotImplemented,
            RecoveryAction::StopUnknown,
            false,
        ),
    }
}

#[derive(Debug, Clone)]
pub struct CommandError {
    pub command: &'static str,
    pub json: bool,
    pub error: AppError,
}

impl CommandError {
    pub fn new(command: &'static str, json: bool, error: AppError) -> Self {
        Self {
            command,
            json,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, ErrorCategory, ErrorCode, RecoveryAction};

    #[test]
    fn default_recovery_maps_invalid_args_to_fix_args_retry() {
        let error = AppError::new(ErrorCode::InvalidArgs, "bad flag");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
        assert_eq!(error.recovery_action, RecoveryAction::FixArgsThenRetry);
        assert!(error.retryable);
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn default_recovery_maps_state_file_missing_to_login_retry() {
        let error = AppError::new(ErrorCode::StateFileMissing, "missing state");
        assert_eq!(error.category, ErrorCategory::AuthReloginRequired);
        assert_eq!(error.recovery_action, RecoveryAction::LoginThenRetry);
        assert!(error.retryable);
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn default_recovery_maps_auth_missing_and_expired_to_refresh_retry() {
        for code in [ErrorCode::AuthMissing, ErrorCode::AuthExpired] {
            let error = AppError::new(code, "auth problem");
            assert_eq!(error.category, ErrorCategory::AuthRefreshable);
            assert_eq!(error.recovery_action, RecoveryAction::RefreshThenRetry);
            assert!(error.retryable);
            assert_eq!(error.exit_code(), 3);
        }
    }

    #[test]
    fn default_recovery_maps_relogin_required_to_login_retry() {
        let error = AppError::new(ErrorCode::AuthReloginRequired, "login again");
        assert_eq!(error.category, ErrorCategory::AuthReloginRequired);
        assert_eq!(error.recovery_action, RecoveryAction::LoginThenRetry);
        assert!(error.retryable);
        assert!(error.relogin_required);
        assert_eq!(error.exit_code(), 3);
    }

    #[test]
    fn default_recovery_maps_entitlement_billing_quota_and_rate_limit_to_stops() {
        let cases = [
            (
                ErrorCode::XaiOauthTierDenied,
                ErrorCategory::EntitlementDenied,
                RecoveryAction::StopEntitlement,
            ),
            (
                ErrorCode::BillingRequired,
                ErrorCategory::BillingRequired,
                RecoveryAction::StopBilling,
            ),
            (
                ErrorCode::QuotaExhausted,
                ErrorCategory::QuotaExhausted,
                RecoveryAction::StopQuota,
            ),
            (
                ErrorCode::RateLimited,
                ErrorCategory::RateLimited,
                RecoveryAction::StopRateLimit,
            ),
        ];

        for (code, category, action) in cases {
            let error = AppError::new(code, "stop");
            assert_eq!(error.category, category);
            assert_eq!(error.recovery_action, action);
            assert!(!error.retryable);
            assert_eq!(error.exit_code(), 4);
        }
    }

    #[test]
    fn default_recovery_maps_model_capability_and_request_failures_to_stop_unknown() {
        let cases = [
            (
                ErrorCode::ModelCapabilityMismatch,
                ErrorCategory::CapabilityMismatch,
                5,
            ),
            (ErrorCode::RequestFailed, ErrorCategory::RequestFailed, 1),
            (ErrorCode::NotImplemented, ErrorCategory::NotImplemented, 1),
        ];

        for (code, category, exit_code) in cases {
            let error = AppError::new(code, "stop");
            assert_eq!(error.category, category);
            assert_eq!(error.recovery_action, RecoveryAction::StopUnknown);
            assert!(!error.retryable);
            assert_eq!(error.exit_code(), exit_code);
        }
    }

    #[test]
    fn retry_after_seconds_is_preserved_without_changing_default_action() {
        let error =
            AppError::new(ErrorCode::RateLimited, "rate").with_retry_after_seconds(Some(42));
        assert_eq!(error.retry_after_seconds, Some(42));
        assert_eq!(error.recovery_action, RecoveryAction::StopRateLimit);
        assert!(!error.retryable);
    }
}
