use std::collections::BTreeMap;
use std::time::Duration as StdDuration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use url::Url;

use crate::app::AppContext;
use crate::args::{ExchangeCodeOptions, LoginOptions};
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::output;
use crate::state::model::{
    AuthState, DiscoveryState, LastAuthError, PendingOAuthState, TokenState,
};

use super::callback::{
    loopback_redirect_uri, manual_paste_redirect_uri, parse_manual_callback_input,
};
use super::pkce;

const DEFAULT_ISSUER: &str = "https://auth.x.ai";
const DISCOVERY_URL: &str = "https://auth.x.ai/.well-known/openid-configuration";
const DEFAULT_AUTHORIZATION_ENDPOINT: &str = "https://auth.x.ai/oauth2/authorize";
const DEFAULT_TOKEN_ENDPOINT: &str = "https://auth.x.ai/oauth2/token";
const CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
const SCOPE: &str = "openid profile email offline_access grok-cli:access api:access";
const DEFAULT_BASE_URL: &str = "https://api.x.ai/v1";
const HERMES_PLAN: &str = "generic";
const HERMES_REFERRER: &str = "hermes-agent";
const OAUTH_REQUEST_TIMEOUT_SECONDS: u64 = 20;
const OAUTH_NETWORK_RETRY_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, Serialize)]
pub struct AuthorizeParamsData {
    pub authorize_url: String,
    pub redirect_uri: String,
    pub state: String,
    pub nonce: String,
    pub pkce_method: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub client_id: String,
    pub scope: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LoginData {
    pub(crate) provider: String,
    pub(crate) auth_mode: String,
    pub(crate) saved: bool,
    pub(crate) auth_store_path: String,
    pub(crate) redirect_uri: String,
    pub(crate) base_url: String,
    pub(crate) authorize_url: String,
    pub(crate) state: String,
    pub(crate) nonce: String,
    pub(crate) pkce_method: String,
}

#[derive(Debug, Clone, Serialize)]
struct ExchangeCodeData {
    provider: String,
    auth_mode: String,
    saved: bool,
    auth_store_path: String,
    redirect_uri: String,
    base_url: String,
    last_refresh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryDocument {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TokenResponse {
    pub(crate) access_token: String,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) id_token: Option<String>,
    #[serde(default)]
    pub(crate) expires_in: Option<i64>,
    #[serde(default)]
    pub(crate) token_type: Option<String>,
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

pub fn exchange_code(ctx: &AppContext, opts: ExchangeCodeOptions) -> CommandResult {
    let command = "exchange-code";
    let code_input = opts.code.clone().ok_or_else(|| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::new(ErrorCode::InvalidArgs, "--code is required"),
        )
    })?;

    let auth_store_path = ctx
        .state_store
        .resolve_path(opts.common.auth_file.as_deref());
    let state = ctx
        .state_store
        .load_valid_state(&auth_store_path)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let pending = state.pending_oauth().ok_or_else(|| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::new(
                ErrorCode::AuthStateMismatch,
                "missing pending OAuth session; run `grok-cli login` first",
            ),
        )
    })?;

    let (code, returned_state) = resolve_exchange_inputs(&code_input, opts.state.clone(), &pending)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if returned_state != pending.state {
        return Err(CommandError::new(
            command,
            opts.common.json,
            AppError::new(
                ErrorCode::AuthStateMismatch,
                "returned OAuth state does not match the pending login session",
            ),
        ));
    }

    let redirect_uri = opts
        .redirect_uri
        .clone()
        .or_else(|| state.redirect_uri.clone())
        .ok_or_else(|| {
            CommandError::new(
                command,
                opts.common.json,
                AppError::state_file_invalid("missing redirect_uri in auth state"),
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
    )?;

    Ok(())
}

fn resolve_exchange_inputs(
    code_input: &str,
    provided_state: Option<String>,
    pending: &PendingOAuthState,
) -> Result<(String, String), AppError> {
    let parsed = parse_manual_callback_input(code_input)?;
    let returned_state = provided_state
        .or(parsed.state)
        .unwrap_or_else(|| pending.state.clone());

    let code = parsed.code.ok_or_else(|| {
        AppError::new(
            ErrorCode::InvalidArgs,
            "callback response is missing the `code` query parameter",
        )
    })?;

    Ok((code, returned_state))
}

pub(crate) fn exchange_pending_session(
    command: &'static str,
    ctx: &AppContext,
    auth_file: Option<std::path::PathBuf>,
    code: String,
    returned_state: String,
    redirect_uri: String,
    json: bool,
) -> Result<(), CommandError> {
    let auth_store_path = ctx.state_store.resolve_path(auth_file.as_deref());
    let mut state = ctx
        .state_store
        .load_valid_state(&auth_store_path)
        .map_err(|error| CommandError::new(command, json, error))?;

    let pending = state.pending_oauth().ok_or_else(|| {
        CommandError::new(
            command,
            json,
            AppError::new(
                ErrorCode::AuthStateMismatch,
                "missing pending OAuth session; run `grok-cli login` first",
            ),
        )
    })?;

    if returned_state != pending.state {
        return Err(CommandError::new(
            command,
            json,
            AppError::new(
                ErrorCode::AuthStateMismatch,
                "returned OAuth state does not match the pending login session",
            ),
        ));
    }

    let token_endpoint = token_endpoint_for_state(&state)
        .map_err(|error| CommandError::new(command, json, error))?;

    match exchange_code_request(ctx, &token_endpoint, &code, &redirect_uri, &pending) {
        Ok(token_response) => {
            let now = now_rfc3339();
            let existing_refresh = state.tokens.refresh_token.clone();
            state.tokens = TokenState {
                access_token: Some(token_response.access_token),
                refresh_token: token_response.refresh_token.or(existing_refresh),
                id_token: token_response.id_token,
                expires_in: token_response.expires_in,
                expires_at: None,
                token_type: token_response.token_type.or(Some("Bearer".to_string())),
            };
            state.last_refresh = Some(now.clone());
            state.last_auth_error = None;
            state.redirect_uri = Some(redirect_uri.clone());
            state.clear_pending_oauth();

            ctx.state_store
                .write_state(&auth_store_path, &state)
                .map_err(|error| CommandError::new(command, json, error))?;

            let data = ExchangeCodeData {
                provider: state.provider.clone(),
                auth_mode: state.auth_mode.clone(),
                saved: true,
                auth_store_path: auth_store_path.display().to_string(),
                redirect_uri,
                base_url: state.base_url.clone(),
                last_refresh: now,
            };

            if json {
                output::print_json_success(command, &data);
            } else {
                println!("saved: {}", data.saved);
                println!("auth_store_path: {}", data.auth_store_path);
                println!("redirect_uri: {}", data.redirect_uri);
                println!("last_refresh: {}", data.last_refresh);
            }

            Ok(())
        }
        Err(error) => {
            state.last_auth_error = Some(build_last_auth_error(
                &error,
                "token_exchange_failure",
                Some(build_oauth_error_context(
                    &token_endpoint,
                    "token_exchange",
                    "authorization_code",
                    Some(&redirect_uri),
                    !pending.manual_paste,
                    OAUTH_NETWORK_RETRY_ATTEMPTS,
                )),
            ));
            let _ = ctx.state_store.write_state(&auth_store_path, &state);
            Err(CommandError::new(command, json, error))
        }
    }
}

pub fn build_authorize_params(
    ctx: &AppContext,
    opts: &LoginOptions,
) -> Result<AuthorizeParamsData, AppError> {
    let discovery = fetch_discovery(ctx)?;
    validate_discovery(&discovery)?;

    let redirect_uri = resolve_redirect_uri(opts.port, opts.manual_paste);
    let pkce = pkce::generate_pkce();
    let state = pkce::generate_state();
    let nonce = pkce::generate_nonce();

    let mut url = Url::parse(&discovery.authorization_endpoint).map_err(|error| {
        AppError::state_file_invalid(format!("invalid authorization endpoint: {error}"))
    })?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", SCOPE)
        .append_pair("code_challenge", &pkce.challenge)
        .append_pair("code_challenge_method", pkce.method)
        .append_pair("state", &state)
        .append_pair("nonce", &nonce)
        .append_pair("plan", HERMES_PLAN)
        .append_pair("referrer", HERMES_REFERRER);

    Ok(AuthorizeParamsData {
        authorize_url: url.to_string(),
        redirect_uri,
        state,
        nonce,
        pkce_method: pkce.method.to_string(),
        code_verifier: pkce.verifier,
        code_challenge: pkce.challenge,
        client_id: CLIENT_ID.to_string(),
        scope: SCOPE.to_string(),
        authorization_endpoint: discovery.authorization_endpoint,
        token_endpoint: discovery.token_endpoint,
    })
}

fn resolve_redirect_uri(port: Option<u16>, manual_paste: bool) -> String {
    if manual_paste {
        manual_paste_redirect_uri(port)
    } else {
        loopback_redirect_uri(port)
    }
}

pub(crate) fn open_browser(authorize_url: &str) -> Result<(), AppError> {
    webbrowser::open(authorize_url).map_err(|error| {
        AppError::new(
            ErrorCode::RequestFailed,
            format!("failed to open browser: {error}"),
        )
    })?;
    Ok(())
}

fn fetch_discovery(ctx: &AppContext) -> Result<DiscoveryDocument, AppError> {
    let response = match ctx.http_client.get(DISCOVERY_URL).send() {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%error, "failed to fetch discovery document, using fallback endpoints");
            return fallback_discovery();
        }
    };

    if !response.status().is_success() {
        tracing::warn!(
            status = %response.status(),
            "discovery request was not successful, using fallback endpoints"
        );
        return fallback_discovery();
    }

    let discovery = response.json::<DiscoveryDocument>().map_err(|error| {
        AppError::new(
            ErrorCode::RequestFailed,
            format!("failed to decode discovery document: {error}"),
        )
    })?;

    tracing::debug!(issuer = %discovery.issuer, "loaded xAI OAuth discovery");

    Ok(discovery)
}

fn fallback_discovery() -> Result<DiscoveryDocument, AppError> {
    let discovery = DiscoveryDocument {
        issuer: DEFAULT_ISSUER.to_string(),
        authorization_endpoint: DEFAULT_AUTHORIZATION_ENDPOINT.to_string(),
        token_endpoint: DEFAULT_TOKEN_ENDPOINT.to_string(),
    };
    validate_discovery(&discovery)?;
    Ok(discovery)
}

fn validate_discovery(discovery: &DiscoveryDocument) -> Result<(), AppError> {
    if discovery.issuer != DEFAULT_ISSUER {
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("unexpected discovery issuer: {}", discovery.issuer),
        ));
    }

    validate_endpoint("authorization_endpoint", &discovery.authorization_endpoint)?;
    validate_endpoint("token_endpoint", &discovery.token_endpoint)?;
    Ok(())
}

fn validate_endpoint(name: &str, endpoint: &str) -> Result<(), AppError> {
    let url = Url::parse(endpoint).map_err(|error| {
        AppError::new(
            ErrorCode::RequestFailed,
            format!("invalid discovery {name}: {error}"),
        )
    })?;

    if url.scheme() != "https" {
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("discovery {name} must use https"),
        ));
    }

    let host = url.host_str().unwrap_or_default();
    if host != "x.ai" && !host.ends_with(".x.ai") {
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("discovery {name} must target x.ai, got {host}"),
        ));
    }

    Ok(())
}

pub fn token_exchange_form_for_debug(
    code: &str,
    redirect_uri: &str,
    verifier: &str,
    challenge: &str,
) -> BTreeMap<&'static str, String> {
    let mut form = BTreeMap::new();
    form.insert("grant_type", "authorization_code".to_string());
    form.insert("code", code.to_string());
    form.insert("redirect_uri", redirect_uri.to_string());
    form.insert("client_id", CLIENT_ID.to_string());
    form.insert("code_verifier", verifier.to_string());
    form.insert("code_challenge", challenge.to_string());
    form.insert("code_challenge_method", "S256".to_string());
    form
}

pub fn refresh_form_for_debug(refresh_token: &str) -> BTreeMap<&'static str, String> {
    let mut form = BTreeMap::new();
    form.insert("grant_type", "refresh_token".to_string());
    form.insert("client_id", CLIENT_ID.to_string());
    form.insert("refresh_token", refresh_token.to_string());
    form
}

pub(crate) fn token_endpoint_for_state(state: &AuthState) -> Result<String, AppError> {
    let endpoint = state
        .discovery
        .token_endpoint
        .clone()
        .unwrap_or_else(|| DEFAULT_TOKEN_ENDPOINT.to_string());
    validate_endpoint("token_endpoint", &endpoint)?;
    Ok(endpoint)
}

pub(crate) fn map_refresh_error(status: reqwest::StatusCode, body: &str) -> AppError {
    let payload = serde_json::from_str::<OAuthErrorResponse>(body).unwrap_or(OAuthErrorResponse {
        error: None,
        error_description: None,
        message: None,
    });

    let reason = payload
        .error_description
        .clone()
        .or(payload.message.clone())
        .unwrap_or_else(|| format!("refresh request failed with status {status}"));
    let error_code = payload.error.unwrap_or_default();
    let lower = format!("{error_code} {reason}").to_lowercase();

    if status == reqwest::StatusCode::FORBIDDEN
        || lower.contains("entitlement")
        || lower.contains("tier")
        || lower.contains("not authorized")
    {
        return AppError::new(ErrorCode::XaiOauthTierDenied, reason);
    }

    if error_code == "invalid_grant" || lower.contains("invalid or unknown refresh token") {
        return AppError::new(ErrorCode::AuthReloginRequired, reason);
    }

    AppError::new(ErrorCode::AuthRefreshFailed, reason)
}

pub(crate) fn build_last_auth_error(
    error: &AppError,
    reason: &str,
    context: Option<Map<String, Value>>,
) -> LastAuthError {
    LastAuthError {
        provider: "xai-oauth".to_string(),
        code: error.code.as_str().to_string(),
        message: error.message.clone(),
        reason: Some(reason.to_string()),
        relogin_required: error.relogin_required,
        entitlement_denied: error.entitlement_denied,
        context: context.unwrap_or_default(),
        at: now_rfc3339(),
    }
}

pub(crate) fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub(crate) fn build_oauth_error_context(
    endpoint: &str,
    phase: &str,
    grant_type: &str,
    redirect_uri: Option<&str>,
    loopback: bool,
    retry_attempts: usize,
) -> Map<String, Value> {
    let mut context = Map::new();
    context.insert("endpoint".to_string(), json!(endpoint));
    context.insert("phase".to_string(), json!(phase));
    context.insert("grant_type".to_string(), json!(grant_type));
    context.insert("loopback".to_string(), json!(loopback));
    context.insert("retry_attempts".to_string(), json!(retry_attempts));
    if let Some(redirect_uri) = redirect_uri {
        context.insert("redirect_uri".to_string(), json!(redirect_uri));
    }
    context
}

pub(crate) fn send_oauth_form_with_retry(
    ctx: &AppContext,
    endpoint: &str,
    phase: &str,
    grant_type: &str,
    form: &BTreeMap<&'static str, String>,
    failure_code: ErrorCode,
) -> Result<reqwest::blocking::Response, AppError> {
    let mut last_error = None;

    for attempt in 1..=OAUTH_NETWORK_RETRY_ATTEMPTS {
        let response = ctx
            .http_client
            .post(endpoint)
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .timeout(StdDuration::from_secs(OAUTH_REQUEST_TIMEOUT_SECONDS))
            .form(form)
            .send();

        match response {
            Ok(response) => return Ok(response),
            Err(error) if should_retry_oauth_transport_error(&error, attempt) => {
                tracing::warn!(
                    %phase,
                    %grant_type,
                    attempt,
                    endpoint,
                    error = %error,
                    "OAuth request failed with a retryable transport error",
                );
                std::thread::sleep(StdDuration::from_millis(250 * attempt as u64));
                last_error = Some(error);
            }
            Err(error) => {
                return Err(build_oauth_transport_error(
                    phase,
                    endpoint,
                    grant_type,
                    &error,
                    failure_code,
                    attempt,
                ));
            }
        }
    }

    let error = last_error.expect("retry loop should capture the last transport error");
    Err(build_oauth_transport_error(
        phase,
        endpoint,
        grant_type,
        &error,
        failure_code,
        OAUTH_NETWORK_RETRY_ATTEMPTS,
    ))
}

fn should_retry_oauth_transport_error(error: &reqwest::Error, attempt: usize) -> bool {
    if attempt >= OAUTH_NETWORK_RETRY_ATTEMPTS {
        return false;
    }

    error.is_timeout() || error.is_connect() || error.is_request()
}

fn build_oauth_transport_error(
    phase: &str,
    endpoint: &str,
    grant_type: &str,
    error: &reqwest::Error,
    failure_code: ErrorCode,
    attempts: usize,
) -> AppError {
    let kind = oauth_transport_error_kind(error);
    let message = format!(
        "{phase} request failed after {attempts} attempt(s) [{kind}] for {endpoint}: {error}"
    );
    tracing::warn!(
        %phase,
        %grant_type,
        %endpoint,
        attempts,
        kind,
        error = %error,
        "OAuth transport request failed",
    );
    AppError::new(failure_code, message)
}

fn oauth_transport_error_kind(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else if error.is_request() {
        "request"
    } else if error.is_status() {
        "status"
    } else if error.is_decode() {
        "decode"
    } else if error.is_body() {
        "body"
    } else {
        "transport"
    }
}

fn exchange_code_request(
    ctx: &AppContext,
    token_endpoint: &str,
    code: &str,
    redirect_uri: &str,
    pending: &PendingOAuthState,
) -> Result<TokenResponse, AppError> {
    let form = token_exchange_form_for_debug(
        code,
        redirect_uri,
        &pending.code_verifier,
        &pending.code_challenge,
    );
    let response = send_oauth_form_with_retry(
        ctx,
        token_endpoint,
        "token_exchange",
        "authorization_code",
        &form,
        ErrorCode::AuthTokenExchangeFailed,
    )?;

    if response.status().is_success() {
        return response.json::<TokenResponse>().map_err(|error| {
            AppError::new(
                ErrorCode::AuthTokenExchangeFailed,
                format!("failed to decode token exchange response: {error}"),
            )
        });
    }

    let status = response.status();
    let body = response.text().unwrap_or_default();
    Err(map_exchange_error(status, &body))
}

fn map_exchange_error(status: reqwest::StatusCode, body: &str) -> AppError {
    let payload = serde_json::from_str::<OAuthErrorResponse>(body).unwrap_or(OAuthErrorResponse {
        error: None,
        error_description: None,
        message: None,
    });

    let reason = payload
        .error_description
        .clone()
        .or(payload.message.clone())
        .unwrap_or_else(|| format!("token exchange failed with status {status}"));
    let error_code = payload.error.unwrap_or_default();
    let lower = format!("{error_code} {reason}").to_lowercase();

    if status == reqwest::StatusCode::FORBIDDEN
        || lower.contains("entitlement")
        || lower.contains("tier")
        || lower.contains("not authorized")
    {
        return AppError::new(ErrorCode::XaiOauthTierDenied, reason);
    }

    AppError::new(ErrorCode::AuthTokenExchangeFailed, reason)
}

pub(crate) fn persist_pending_session(
    ctx: &AppContext,
    opts: &LoginOptions,
    params: &AuthorizeParamsData,
) -> Result<std::path::PathBuf, AppError> {
    let auth_store_path = ctx
        .state_store
        .resolve_path(opts.common.auth_file.as_deref());
    let mut state = AuthState::empty(DEFAULT_BASE_URL.to_string());
    state.discovery = DiscoveryState {
        authorization_endpoint: Some(params.authorization_endpoint.clone()),
        token_endpoint: Some(params.token_endpoint.clone()),
    };
    state.redirect_uri = Some(params.redirect_uri.clone());
    state.set_pending_oauth(PendingOAuthState {
        state: params.state.clone(),
        nonce: params.nonce.clone(),
        code_verifier: params.code_verifier.clone(),
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: params.pkce_method.clone(),
        manual_paste: opts.manual_paste,
        no_browser: opts.no_browser,
        created_at: OffsetDateTime::now_utc().format(&Rfc3339).ok(),
    });

    ctx.state_store.write_state(&auth_store_path, &state)?;
    Ok(auth_store_path)
}

#[cfg(test)]
mod tests {
    use super::{build_oauth_error_context, resolve_exchange_inputs};
    use crate::state::model::PendingOAuthState;

    fn sample_pending() -> PendingOAuthState {
        PendingOAuthState {
            state: "pending-state".to_string(),
            nonce: "sample-nonce".to_string(),
            code_verifier: "sample-verifier".to_string(),
            code_challenge: "sample-challenge".to_string(),
            code_challenge_method: "S256".to_string(),
            manual_paste: true,
            no_browser: false,
            created_at: None,
        }
    }

    #[test]
    fn resolve_exchange_inputs_reuses_pending_state_when_missing() {
        let pending = sample_pending();
        let (code, state) = resolve_exchange_inputs("manual-code", None, &pending).unwrap();
        assert_eq!(code, "manual-code");
        assert_eq!(state, pending.state);
    }

    #[test]
    fn resolve_exchange_inputs_prefers_explicit_state() {
        let pending = sample_pending();
        let (code, state) =
            resolve_exchange_inputs("manual-code", Some("explicit-state".to_string()), &pending)
                .unwrap();
        assert_eq!(code, "manual-code");
        assert_eq!(state, "explicit-state");
    }

    #[test]
    fn build_oauth_error_context_captures_request_shape() {
        let context = build_oauth_error_context(
            "https://auth.x.ai/oauth2/token",
            "token_exchange",
            "authorization_code",
            Some("http://127.0.0.1:56121/callback"),
            true,
            2,
        );

        assert_eq!(
            context.get("endpoint").and_then(|value| value.as_str()),
            Some("https://auth.x.ai/oauth2/token")
        );
        assert_eq!(
            context.get("phase").and_then(|value| value.as_str()),
            Some("token_exchange")
        );
        assert_eq!(
            context.get("grant_type").and_then(|value| value.as_str()),
            Some("authorization_code")
        );
        assert_eq!(
            context.get("redirect_uri").and_then(|value| value.as_str()),
            Some("http://127.0.0.1:56121/callback")
        );
        assert_eq!(
            context.get("loopback").and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            context
                .get("retry_attempts")
                .and_then(|value| value.as_u64()),
            Some(2)
        );
    }
}
