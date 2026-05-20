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
    ModelCapabilityMismatch,
    RequestFailed,
    NotImplemented,
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
            Self::ModelCapabilityMismatch => "model_capability_mismatch",
            Self::RequestFailed => "request_failed",
            Self::NotImplemented => "not_implemented",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub relogin_required: bool,
    pub entitlement_denied: bool,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let code_value = code;
        Self {
            code: code_value,
            message: message.into(),
            relogin_required: matches!(code_value, ErrorCode::AuthReloginRequired),
            entitlement_denied: matches!(code_value, ErrorCode::XaiOauthTierDenied),
        }
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
            ErrorCode::XaiOauthTierDenied => 4,
            ErrorCode::ModelCapabilityMismatch => 5,
            _ => 1,
        }
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
