use serde::Serialize;

use crate::app::AppContext;
use crate::error::{AppError, ErrorCode};
use crate::state::model::AuthState;

use super::login::{
    TokenResponse, build_last_auth_error, build_oauth_error_context, map_refresh_error,
    now_rfc3339, refresh_form_for_debug, send_oauth_form_with_retry, token_endpoint_for_state,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeCredentialOptions {
    pub refresh_if_expiring: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeCredentials {
    pub provider: String,
    pub base_url: String,
    pub access_token: String,
    pub token_type: String,
}

pub fn resolve_runtime_credentials(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
) -> Result<RuntimeCredentials, AppError> {
    resolve_runtime_credentials_with_options(ctx, auth_file, RuntimeCredentialOptions::default())
}

pub fn resolve_runtime_credentials_with_options(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    options: RuntimeCredentialOptions,
) -> Result<RuntimeCredentials, AppError> {
    let auth_store_path = ctx.state_store.resolve_path(auth_file);
    let mut state = ctx.state_store.load_valid_state(&auth_store_path)?;

    if should_refresh_before_request(&state, options, time::OffsetDateTime::now_utc()) {
        refresh_state_tokens(ctx, &auth_store_path, &mut state)?;
    }

    if let Some(access_token) = usable_access_token(&state) {
        return Ok(RuntimeCredentials {
            provider: state.provider.clone(),
            base_url: state.base_url.clone(),
            access_token,
            token_type: state
                .tokens
                .token_type
                .clone()
                .unwrap_or_else(|| "Bearer".to_string()),
        });
    }

    refresh_state_tokens(ctx, &auth_store_path, &mut state)?;

    let access_token = usable_access_token(&state).ok_or_else(|| {
        AppError::new(
            ErrorCode::AuthMissing,
            "access token is missing after refresh; run `grok-cli login` again",
        )
    })?;

    Ok(RuntimeCredentials {
        provider: state.provider.clone(),
        base_url: state.base_url.clone(),
        access_token,
        token_type: state
            .tokens
            .token_type
            .clone()
            .unwrap_or_else(|| "Bearer".to_string()),
    })
}

pub fn refresh_state_tokens(
    ctx: &AppContext,
    auth_store_path: &std::path::Path,
    state: &mut AuthState,
) -> Result<(), AppError> {
    let refresh_token = state.tokens.refresh_token.clone().ok_or_else(|| {
        AppError::new(
            ErrorCode::AuthMissing,
            "refresh token is missing; run `grok-cli login` again",
        )
    })?;

    let token_endpoint = token_endpoint_for_state(state)?;
    let form = refresh_form_for_debug(&refresh_token);
    let response = send_oauth_form_with_retry(
        ctx,
        &token_endpoint,
        "refresh",
        "refresh_token",
        &form,
        ErrorCode::AuthRefreshFailed,
    )?;

    if response.status().is_success() {
        let token_response = response.json::<TokenResponse>().map_err(|error| {
            AppError::new(
                ErrorCode::AuthRefreshFailed,
                format!("failed to decode refresh response: {error}"),
            )
        })?;

        let now = now_rfc3339();
        let existing_refresh = state.tokens.refresh_token.clone();
        state.tokens.access_token = Some(token_response.access_token);
        state.tokens.refresh_token = token_response.refresh_token.or(existing_refresh);
        state.tokens.id_token = token_response.id_token;
        state.tokens.expires_in = token_response.expires_in;
        state.tokens.expires_at = None;
        state.tokens.token_type = token_response.token_type.or(Some("Bearer".to_string()));
        state.last_refresh = Some(now);
        state.last_auth_error = None;

        ctx.state_store.write_state(auth_store_path, state)?;
        return Ok(());
    }

    let status = response.status();
    let body = response.text().unwrap_or_default();
    let error = map_refresh_error(status, &body);
    state.last_auth_error = Some(build_last_auth_error(
        &error,
        "refresh_failure",
        Some(build_oauth_error_context(
            &token_endpoint,
            "refresh",
            "refresh_token",
            state.redirect_uri.as_deref(),
            true,
            2,
        )),
    ));
    let _ = ctx.state_store.write_state(auth_store_path, state);
    Err(error)
}

fn usable_access_token(state: &AuthState) -> Option<String> {
    state
        .tokens
        .access_token
        .clone()
        .filter(|token| !token.trim().is_empty())
}

fn should_refresh_before_request(
    state: &AuthState,
    options: RuntimeCredentialOptions,
    now: time::OffsetDateTime,
) -> bool {
    options.refresh_if_expiring
        && state
            .tokens
            .refresh_token
            .as_deref()
            .map(|token| !token.trim().is_empty())
            .unwrap_or(false)
        && state.access_token_expiring_now(now)
}

#[cfg(test)]
mod tests {
    use super::{RuntimeCredentialOptions, should_refresh_before_request};
    use crate::state::model::AuthState;
    use time::OffsetDateTime;

    #[test]
    fn proactive_refresh_triggers_for_expiring_token() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.tokens.access_token = Some("sample-access-token".to_string());
        state.tokens.refresh_token = Some("sample-refresh-token".to_string());
        state.tokens.expires_in = Some(3600);
        state.last_refresh = Some("2026-05-19T00:00:00Z".to_string());

        assert!(should_refresh_before_request(
            &state,
            RuntimeCredentialOptions {
                refresh_if_expiring: true,
            },
            OffsetDateTime::parse(
                "2026-05-20T00:00:00Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
        ));
    }

    #[test]
    fn proactive_refresh_stays_off_without_opt_in() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.tokens.access_token = Some("sample-access-token".to_string());
        state.tokens.refresh_token = Some("sample-refresh-token".to_string());
        state.tokens.expires_in = Some(3600);
        state.last_refresh = Some("2026-05-19T00:00:00Z".to_string());

        assert!(!should_refresh_before_request(
            &state,
            RuntimeCredentialOptions {
                refresh_if_expiring: false,
            },
            OffsetDateTime::parse(
                "2026-05-20T00:00:00Z",
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap(),
        ));
    }
}
