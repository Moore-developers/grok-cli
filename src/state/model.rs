use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

const ACCESS_TOKEN_EXPIRY_SKEW_SECONDS: i64 = 300;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub version: u64,
    pub provider: String,
    pub auth_mode: String,
    pub base_url: String,
    #[serde(default)]
    pub tokens: TokenState,
    #[serde(default)]
    pub discovery: DiscoveryState,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub last_refresh: Option<String>,
    #[serde(default)]
    pub last_auth_error: Option<LastAuthError>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOAuthState {
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub manual_paste: bool,
    pub no_browser: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl AuthState {
    pub fn empty(base_url: String) -> Self {
        Self {
            version: 1,
            provider: "xai-oauth".to_string(),
            auth_mode: "oauth_pkce".to_string(),
            base_url,
            tokens: TokenState::default(),
            discovery: DiscoveryState::default(),
            redirect_uri: None,
            last_refresh: None,
            last_auth_error: None,
            metadata: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenState {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryState {
    #[serde(default)]
    pub authorization_endpoint: Option<String>,
    #[serde(default)]
    pub token_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastAuthError {
    pub provider: String,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub relogin_required: bool,
    #[serde(default)]
    pub entitlement_denied: bool,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub context: Map<String, Value>,
    pub at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactedAuthState {
    pub version: u64,
    pub provider: String,
    pub auth_mode: String,
    pub base_url: String,
    pub tokens: RedactedTokenState,
    pub discovery: DiscoveryState,
    pub redirect_uri: Option<String>,
    pub last_refresh: Option<String>,
    pub last_auth_error: Option<LastAuthError>,
    pub metadata: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactedTokenState {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_in: Option<i64>,
    pub expires_at: Option<String>,
    pub token_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthStatusData {
    pub logged_in: bool,
    pub provider: String,
    pub auth_mode: String,
    pub access_token_present: bool,
    pub refresh_token_present: bool,
    pub access_token_expiring: bool,
    pub relogin_required: bool,
    pub entitlement_denied: bool,
    pub last_refresh: Option<String>,
    pub auth_store_path: String,
    pub base_url: String,
}

impl AuthState {
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();

        if self.version < 1 {
            problems.push("version must be >= 1".to_string());
        }
        if self.provider.trim().is_empty() {
            problems.push("provider is required".to_string());
        }
        if self.auth_mode.trim().is_empty() {
            problems.push("auth_mode is required".to_string());
        }
        if self.base_url.trim().is_empty() {
            problems.push("base_url is required".to_string());
        }

        problems
    }

    pub fn redacted(&self) -> RedactedAuthState {
        RedactedAuthState {
            version: self.version,
            provider: self.provider.clone(),
            auth_mode: self.auth_mode.clone(),
            base_url: self.base_url.clone(),
            tokens: RedactedTokenState {
                access_token: self.tokens.access_token.as_deref().map(redact_secret),
                refresh_token: self.tokens.refresh_token.as_deref().map(redact_secret),
                id_token: self.tokens.id_token.as_deref().map(redact_secret),
                expires_in: self.tokens.expires_in,
                expires_at: self.tokens.expires_at.clone(),
                token_type: self.tokens.token_type.clone(),
            },
            discovery: self.discovery.clone(),
            redirect_uri: self.redirect_uri.clone(),
            last_refresh: self.last_refresh.clone(),
            last_auth_error: self.last_auth_error.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub fn auth_status_data(&self, auth_store_path: String, now: OffsetDateTime) -> AuthStatusData {
        let access_token_present = self.access_token_present();
        let refresh_token_present = self.refresh_token_present();

        AuthStatusData {
            logged_in: access_token_present || refresh_token_present,
            provider: self.provider.clone(),
            auth_mode: self.auth_mode.clone(),
            access_token_present,
            refresh_token_present,
            access_token_expiring: self.access_token_expiring(now),
            relogin_required: self
                .last_auth_error
                .as_ref()
                .map(|error| error.relogin_required)
                .unwrap_or(false),
            entitlement_denied: self
                .last_auth_error
                .as_ref()
                .map(|error| error.entitlement_denied)
                .unwrap_or(false),
            last_refresh: self.last_refresh.clone(),
            auth_store_path,
            base_url: self.base_url.clone(),
        }
    }

    pub fn access_token_expiring_now(&self, now: OffsetDateTime) -> bool {
        self.access_token_expiring(now)
    }

    fn access_token_present(&self) -> bool {
        self.tokens
            .access_token
            .as_deref()
            .map(|token| !token.trim().is_empty())
            .unwrap_or(false)
    }

    fn refresh_token_present(&self) -> bool {
        self.tokens
            .refresh_token
            .as_deref()
            .map(|token| !token.trim().is_empty())
            .unwrap_or(false)
    }

    fn access_token_expiring(&self, now: OffsetDateTime) -> bool {
        let threshold = now + Duration::seconds(ACCESS_TOKEN_EXPIRY_SKEW_SECONDS);
        self.access_token_expiry()
            .map(|expires_at| expires_at <= threshold)
            .unwrap_or(false)
    }

    fn access_token_expiry(&self) -> Option<OffsetDateTime> {
        if let Some(expires_at) = self.tokens.expires_at.as_deref() {
            return parse_timestamp(expires_at);
        }

        let expires_in = self.tokens.expires_in?;
        let last_refresh = self.last_refresh.as_deref()?;
        let refreshed_at = parse_timestamp(last_refresh)?;
        Some(refreshed_at + Duration::seconds(expires_in))
    }

    pub fn pending_oauth(&self) -> Option<PendingOAuthState> {
        let value = self.metadata.get("pending_oauth")?.clone();
        serde_json::from_value(value).ok()
    }

    pub fn set_pending_oauth(&mut self, pending: PendingOAuthState) {
        self.metadata.insert(
            "pending_oauth".to_string(),
            serde_json::to_value(pending).expect("serialize pending OAuth state"),
        );
    }

    pub fn clear_pending_oauth(&mut self) {
        self.metadata.remove("pending_oauth");
    }
}

fn parse_timestamp(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

fn redact_secret(secret: &str) -> String {
    if secret.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = secret.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}

#[cfg(test)]
mod tests {
    use super::{AuthState, LastAuthError, PendingOAuthState};
    use serde_json::Map;
    use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

    #[test]
    fn validate_reports_required_auth_state_fields() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.version = 0;
        state.provider = " ".to_string();
        state.auth_mode = String::new();
        state.base_url = String::new();

        let problems = state.validate();
        assert!(problems.contains(&"version must be >= 1".to_string()));
        assert!(problems.contains(&"provider is required".to_string()));
        assert!(problems.contains(&"auth_mode is required".to_string()));
        assert!(problems.contains(&"base_url is required".to_string()));
    }

    #[test]
    fn auth_status_uses_token_presence_expiry_and_last_error_flags() {
        let now = OffsetDateTime::now_utc();
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.tokens.access_token = Some("access-token".to_string());
        state.tokens.refresh_token = Some("refresh-token".to_string());
        state.tokens.expires_at = Some((now + Duration::seconds(60)).format(&Rfc3339).unwrap());
        state.last_auth_error = Some(LastAuthError {
            provider: "xai-oauth".to_string(),
            code: "auth_relogin_required".to_string(),
            message: "login again".to_string(),
            reason: Some("test".to_string()),
            relogin_required: true,
            entitlement_denied: false,
            context: Map::new(),
            at: now.format(&Rfc3339).unwrap(),
        });

        let status = state.auth_status_data("/tmp/auth.json".to_string(), now);
        assert!(status.logged_in);
        assert!(status.access_token_present);
        assert!(status.refresh_token_present);
        assert!(status.access_token_expiring);
        assert!(status.relogin_required);
        assert_eq!(status.auth_store_path, "/tmp/auth.json");
    }

    #[test]
    fn pending_oauth_round_trips_and_clears() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        let pending = PendingOAuthState {
            state: "state".to_string(),
            nonce: "nonce".to_string(),
            code_verifier: "verifier".to_string(),
            code_challenge: "challenge".to_string(),
            code_challenge_method: "S256".to_string(),
            manual_paste: false,
            no_browser: true,
            created_at: None,
        };

        state.set_pending_oauth(pending.clone());
        let round_trip = state.pending_oauth().unwrap();
        assert_eq!(round_trip.state, pending.state);
        assert_eq!(round_trip.code_challenge_method, "S256");

        state.clear_pending_oauth();
        assert!(state.pending_oauth().is_none());
    }
}
