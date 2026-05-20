use serde::Serialize;

use crate::app::AppContext;
use crate::args::StateFileOptions;
use crate::cli::CommandResult;
use crate::error::CommandError;
use crate::output;

use super::resolver::refresh_state_tokens;

#[derive(Debug, Clone, Serialize)]
struct RefreshData {
    provider: String,
    refreshed: bool,
    last_refresh: String,
}

pub fn refresh(ctx: &AppContext, opts: StateFileOptions) -> CommandResult {
    let command = "refresh";
    let auth_store_path = ctx.state_store.resolve_path(opts.auth_file.as_deref());
    let mut state = ctx
        .state_store
        .load_valid_state(&auth_store_path)
        .map_err(|error| CommandError::new(command, opts.json, error))?;
    let last_refresh_before = state.last_refresh.clone();
    refresh_state_tokens(ctx, &auth_store_path, &mut state)
        .map_err(|error| CommandError::new(command, opts.json, error))?;

    let data = RefreshData {
        provider: state.provider.clone(),
        refreshed: true,
        last_refresh: state
            .last_refresh
            .clone()
            .or(last_refresh_before)
            .unwrap_or_default(),
    };

    if opts.json {
        output::print_json_success(command, &data);
    } else {
        println!("provider: {}", data.provider);
        println!("refreshed: {}", data.refreshed);
        println!("last_refresh: {}", data.last_refresh);
    }

    Ok(())
}
