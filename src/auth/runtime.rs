use std::io::IsTerminal;

use crate::app::AppContext;
use crate::args::LoginOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::state::model::AuthState;

use super::callback::{parse_manual_callback_input, wait_for_callback};
use super::login::{
    LoginData, build_authorize_params, build_last_auth_error, exchange_pending_session,
    open_browser, persist_pending_session,
};

pub fn login(ctx: &AppContext, opts: LoginOptions) -> CommandResult {
    let command = "login";
    let params = build_authorize_params(ctx, &opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_store_path = persist_pending_session(ctx, &opts, &params)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if opts.manual_paste {
        let data = LoginData {
            provider: "xai-oauth".to_string(),
            auth_mode: "oauth_pkce".to_string(),
            saved: true,
            auth_store_path: auth_store_path.display().to_string(),
            redirect_uri: params.redirect_uri,
            base_url: "https://api.x.ai/v1".to_string(),
            authorize_url: params.authorize_url,
            state: params.state,
            nonce: params.nonce,
            pkce_method: params.pkce_method,
        };

        if opts.common.json {
            crate::output::print_json_success(command, &data);
        } else {
            println!("saved: {}", data.saved);
            println!("auth_store_path: {}", data.auth_store_path);
            println!("redirect_uri: {}", data.redirect_uri);
            println!("authorize_url: {}", data.authorize_url);
        }
        return Ok(());
    }

    if !opts.no_browser {
        open_browser(&params.authorize_url)
            .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    }

    match wait_for_callback(&params.redirect_uri, opts.timeout) {
        Ok(callback) => finalize_login(ctx, &opts, params.redirect_uri, callback),
        Err(error) => {
            if should_fallback_to_manual_paste(&error, &opts) {
                persist_auth_error(
                    ctx,
                    opts.common.auth_file.as_deref(),
                    &error,
                    "callback_runtime_fallback",
                );
                print_manual_fallback_guidance(&params.redirect_uri, &params.authorize_url);
                let manual = parse_manual_callback_input(&prompt_manual_callback_input())
                    .map_err(|error| CommandError::new(command, opts.common.json, error))?;
                finalize_login_with_fallback_state(
                    ctx,
                    &opts,
                    params.redirect_uri,
                    params.state,
                    manual,
                )
            } else {
                persist_auth_error(
                    ctx,
                    opts.common.auth_file.as_deref(),
                    &error,
                    "callback_runtime_failure",
                );
                Err(CommandError::new(command, opts.common.json, error))
            }
        }
    }
}

fn finalize_login(
    ctx: &AppContext,
    opts: &LoginOptions,
    redirect_uri: String,
    callback: super::callback::CallbackResult,
) -> CommandResult {
    let command = "login";

    if let Some(error) = callback.error {
        let message = callback.error_description.unwrap_or_else(|| error.clone());
        let app_error = AppError::new(
            ErrorCode::AuthTokenExchangeFailed,
            format!("OAuth authorization failed: {message}"),
        );
        persist_auth_error(
            ctx,
            opts.common.auth_file.as_deref(),
            &app_error,
            "authorization_failure",
        );
        return Err(CommandError::new(command, opts.common.json, app_error));
    }

    let code = callback.code.ok_or_else(|| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::new(
                crate::error::ErrorCode::AuthCallbackTimeout,
                "callback did not include an authorization code",
            ),
        )
    })?;

    let returned_state = callback.state.ok_or_else(|| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::new(
                crate::error::ErrorCode::AuthStateMismatch,
                "callback did not include an OAuth state value",
            ),
        )
    })?;

    exchange_pending_session(
        command,
        ctx,
        opts.common.auth_file.clone(),
        code,
        returned_state,
        redirect_uri,
        opts.common.json,
    )
}

fn finalize_login_with_fallback_state(
    ctx: &AppContext,
    opts: &LoginOptions,
    redirect_uri: String,
    expected_state: String,
    mut callback: super::callback::CallbackResult,
) -> CommandResult {
    if callback.state.is_none() && callback.code.is_some() {
        callback.state = Some(expected_state);
    }
    finalize_login(ctx, opts, redirect_uri, callback)
}

fn prompt_manual_callback_input() -> String {
    println!();
    println!("Manual callback fallback");
    println!("Paste the failed callback URL, query string, or the copied authorization code:");
    print!("Callback input: ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut raw = String::new();
    match std::io::stdin().read_line(&mut raw) {
        Ok(_) => raw,
        Err(_) => String::new(),
    }
}

fn should_fallback_to_manual_paste(error: &AppError, opts: &LoginOptions) -> bool {
    matches!(error.code, crate::error::ErrorCode::AuthCallbackTimeout)
        && !opts.no_browser
        && std::io::stdin().is_terminal()
}

fn print_manual_fallback_guidance(redirect_uri: &str, authorize_url: &str) {
    println!();
    println!("Loopback callback did not arrive in time.");
    println!(
        "If your browser shows a failure page or a copied authorization code prompt, that is expected."
    );
    println!("Authorize URL: {authorize_url}");
    println!("Expected redirect URI: {redirect_uri}");
    println!("Paste one of the following:");
    println!("1. The full failed callback URL from the browser address bar");
    println!("2. A bare query string like ?code=...&state=...");
    println!("3. The copied authorization code if xAI only shows a code");
}

fn persist_auth_error(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
    error: &AppError,
    reason: &str,
) {
    let auth_store_path = ctx.state_store.resolve_path(auth_file);
    if let Ok(mut state) = load_state_or_empty(ctx, &auth_store_path) {
        state.last_auth_error = Some(build_last_auth_error(error, reason, None));
        let _ = ctx.state_store.write_state(&auth_store_path, &state);
    }
}

fn load_state_or_empty(
    ctx: &AppContext,
    auth_store_path: &std::path::Path,
) -> Result<AuthState, AppError> {
    match ctx.state_store.load_valid_state(auth_store_path) {
        Ok(state) => Ok(state),
        Err(error) if matches!(error.code, ErrorCode::StateFileMissing) => {
            Ok(AuthState::empty("https://api.x.ai/v1".to_string()))
        }
        Err(error) => Err(error),
    }
}
