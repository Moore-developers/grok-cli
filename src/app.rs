use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::args::Cli;
use crate::cli;
use crate::error::CommandError;
use crate::output;
use crate::state::storage::StateStore;
use crate::usage::sqlite::SessionStore;

pub struct AppContext {
    pub state_store: StateStore,
    pub http_client: reqwest::blocking::Client,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            state_store: StateStore::new(),
            http_client: reqwest::blocking::Client::builder()
                // In this environment reqwest/native-tls can take a broken IPv6
                // path to auth.x.ai while curl/browser succeed via IPv4 fallback.
                // Bind outbound HTTP to IPv4 so OAuth token exchange/refresh stays
                // aligned with the working browser path.
                .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                .user_agent("grok-cli/0.1.0")
                .build()
                .expect("build shared HTTP client"),
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

pub fn run() -> i32 {
    init_tracing();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return err.exit_code();
        }
    };

    let ctx = AppContext::new();

    match cli::dispatch(&ctx, cli) {
        Ok(()) => 0,
        Err(report) => {
            emit_command_error(&report);
            report.error.exit_code()
        }
    }
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
