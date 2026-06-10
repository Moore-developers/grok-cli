use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::args::Cli;
use crate::cli;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::output;
use crate::state::storage::StateStore;
use crate::usage::sqlite::SessionStore;

pub struct AppContext {
    pub state_store: StateStore,
    pub http_client: reqwest::blocking::Client,
}

impl AppContext {
    pub fn new() -> Self {
        let http_client = build_http_client().unwrap_or_else(|error| {
            tracing::warn!(
                error = %error.message,
                "failed to build configured HTTP client; using reqwest default client"
            );
            reqwest::blocking::Client::default()
        });

        Self {
            state_store: StateStore::new(),
            http_client,
        }
    }

    pub fn session_store(&self, auth_file: Option<&std::path::Path>) -> SessionStore {
        let session_db = match auth_file {
            Some(auth_file) => auth_file
                .parent()
                .map(|parent| parent.join("session.db"))
                .unwrap_or_else(SessionStore::default_path),
            None => SessionStore::default_path(),
        };
        SessionStore::new(session_db)
    }

    pub fn session_store_with_override(
        &self,
        auth_file: Option<&std::path::Path>,
        session_db_override: Option<&std::path::Path>,
    ) -> SessionStore {
        match session_db_override {
            Some(path) => SessionStore::new(path.to_path_buf()),
            None => self.session_store(auth_file),
        }
    }
}

fn build_http_client() -> Result<reqwest::blocking::Client, AppError> {
    build_http_client_from_builder(|| {
        reqwest::blocking::Client::builder()
            // In this environment reqwest/native-tls can take a broken IPv6
            // path to auth.x.ai while curl/browser succeed via IPv4 fallback.
            // Bind outbound HTTP to IPv4 so OAuth token exchange/refresh stays
            // aligned with the working browser path.
            .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
            .user_agent("grok-cli/0.1.0")
    })
}

fn build_http_client_from_builder<F>(
    builder_factory: F,
) -> Result<reqwest::blocking::Client, AppError>
where
    F: Fn() -> reqwest::blocking::ClientBuilder,
{
    match builder_factory().build() {
        Ok(client) => Ok(client),
        Err(primary_error) => {
            tracing::warn!(
                error = %primary_error,
                "failed to build IPv4-bound HTTP client; retrying without local address"
            );
            reqwest::blocking::Client::builder()
                .user_agent("grok-cli/0.1.0")
                .build()
                .map_err(|fallback_error| {
                    AppError::new(
                        ErrorCode::NetworkTransportFailed,
                        format!(
                            "failed to build HTTP client: primary={primary_error}; fallback={fallback_error}"
                        ),
                    )
                })
        }
    }
}

pub fn run() -> i32 {
    init_tracing();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return err.exit_code();
        }
    };

    let allow_passive_update_check = cli.allows_passive_update_check();
    let ctx = AppContext::new();

    let exit_code = match cli::dispatch(&ctx, cli) {
        Ok(()) => 0,
        Err(report) => {
            emit_command_error(&report);
            report.error.exit_code()
        }
    };

    crate::update::maybe_print_passive_update_notice(&ctx, allow_passive_update_check);

    exit_code
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .compact()
        .try_init();
}

fn emit_command_error(report: &CommandError) {
    if report.json {
        output::print_json_error(report.command, &report.error);
    } else {
        output::print_human_error(report.command, &report.error);
    }
}

#[cfg(test)]
mod tests {
    use super::{AppContext, build_http_client, build_http_client_from_builder};
    use crate::error::{AppError, ErrorCategory, ErrorCode, RecoveryAction};

    #[test]
    fn build_http_client_succeeds_with_default_configuration() {
        let client = build_http_client().unwrap();
        let request = client.get("https://api.x.ai/v1").build().unwrap();

        assert_eq!(request.url().as_str(), "https://api.x.ai/v1");
    }

    #[test]
    fn build_http_client_from_custom_builder_returns_client() {
        let client = build_http_client_from_builder(reqwest::blocking::Client::builder).unwrap();
        let request = client.get("https://auth.x.ai").build().unwrap();

        assert_eq!(request.url().host_str(), Some("auth.x.ai"));
    }

    #[test]
    fn app_context_derives_session_store_path_from_auth_file() {
        let ctx = AppContext::new();
        let auth_file = std::path::Path::new("/tmp/grok-cli-test/auth.json");
        let session_store = ctx.session_store(Some(auth_file));
        let override_store = ctx.session_store_with_override(
            Some(auth_file),
            Some(std::path::Path::new("/tmp/custom.db")),
        );

        assert_eq!(
            session_store.path,
            std::path::Path::new("/tmp/grok-cli-test/session.db")
        );
        assert_eq!(override_store.path, std::path::Path::new("/tmp/custom.db"));
    }

    #[test]
    fn http_client_build_failures_use_retryable_transport_error_shape() {
        let error = AppError::new(
            ErrorCode::NetworkTransportFailed,
            "failed to build HTTP client",
        );

        assert_eq!(error.category, ErrorCategory::RequestFailed);
        assert_eq!(error.recovery_action, RecoveryAction::WaitThenRetry);
        assert!(error.retryable);
    }
}
