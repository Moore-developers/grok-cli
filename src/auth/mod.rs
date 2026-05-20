pub mod callback;
pub mod login;
pub mod pkce;
pub mod refresh;
pub mod resolver;
pub mod runtime;

use time::OffsetDateTime;

use crate::app::AppContext;
use crate::args::StateFileOptions;
use crate::cli::CommandResult;
use crate::error::CommandError;
use crate::output;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct LogoutData {
    removed: bool,
    auth_store_path: String,
}

pub fn auth_status(ctx: &AppContext, opts: StateFileOptions) -> CommandResult {
    let command = "status";
    let path = ctx.state_store.resolve_path(opts.auth_file.as_deref());
    let state = ctx
        .state_store
        .load_valid_state(&path)
        .map_err(|error| CommandError::new(command, opts.json, error))?;

    let data = state.auth_status_data(path.display().to_string(), OffsetDateTime::now_utc());

    if opts.json {
        output::print_json_success(command, &data);
    } else {
        print_status_table(&data);
    }

    Ok(())
}

fn print_status_table(data: &crate::state::model::AuthStatusData) {
    println!("OAuth Status");
    println!("{:<24} | {:<42} | Description", "Field", "Value");
    println!(
        "{}-+-{}-+-{}",
        "-".repeat(24),
        "-".repeat(42),
        "-".repeat(48)
    );

    print_status_row(
        "logged_in",
        data.logged_in,
        "Whether grok-cli can use the saved OAuth session.",
    );
    print_status_row(
        "provider",
        &data.provider,
        "Active provider stored in the auth state.",
    );
    print_status_row(
        "auth_mode",
        &data.auth_mode,
        "Authentication flow used for this session.",
    );
    print_status_row(
        "access_token_present",
        data.access_token_present,
        "A bearer access token is saved locally.",
    );
    print_status_row(
        "refresh_token_present",
        data.refresh_token_present,
        "A refresh token is available for renewing access.",
    );
    print_status_row(
        "access_token_expiring",
        data.access_token_expiring,
        "The access token is near expiry and should be refreshed soon.",
    );
    print_status_row(
        "relogin_required",
        data.relogin_required,
        "A previous auth error requires browser login again.",
    );
    print_status_row(
        "entitlement_denied",
        data.entitlement_denied,
        "The account appears to lack the required Grok entitlement.",
    );
    print_status_row(
        "last_refresh",
        data.last_refresh.as_deref().unwrap_or("none"),
        "Last successful token refresh timestamp.",
    );
    print_status_row(
        "auth_store_path",
        &data.auth_store_path,
        "Local auth.json path used by this command.",
    );
    print_status_row(
        "base_url",
        &data.base_url,
        "xAI API base URL used for runtime requests.",
    );
}

fn print_status_row(field: &str, value: impl ToString, description: &str) {
    println!(
        "{:<24} | {:<42} | {}",
        field,
        value.to_string(),
        description
    );
}

pub fn logout(ctx: &AppContext, opts: StateFileOptions) -> CommandResult {
    let command = "logout";
    let path = ctx.state_store.resolve_path(opts.auth_file.as_deref());
    let removed = ctx
        .state_store
        .remove_state(&path)
        .map_err(|error| CommandError::new(command, opts.json, error))?;

    let data = LogoutData {
        removed,
        auth_store_path: path.display().to_string(),
    };

    if opts.json {
        output::print_json_success(command, &data);
    } else {
        println!("removed: {}", data.removed);
        println!("auth_store_path: {}", data.auth_store_path);
    }

    Ok(())
}
