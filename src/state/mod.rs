pub mod model;
pub mod storage;

use serde::Serialize;

use crate::app::AppContext;
use crate::args::StateFileOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError};
use crate::output;

#[derive(Debug, Clone, Serialize)]
struct StateShowData {
    exists: bool,
    path: String,
    state: Option<model::RedactedAuthState>,
}

pub fn show(ctx: &AppContext, opts: StateFileOptions) -> CommandResult {
    let command = "state";
    let path = ctx.state_store.resolve_path(opts.auth_file.as_deref());
    let inspection = ctx
        .state_store
        .inspect(&path)
        .map_err(|error| CommandError::new(command, opts.json, error))?;

    if !inspection.exists {
        let data = StateShowData {
            exists: false,
            path: path.display().to_string(),
            state: None,
        };
        if opts.json {
            output::print_json_success(command, &data);
        } else {
            println!("exists: false");
            println!("path: {}", data.path);
        }
        return Ok(());
    }

    if !inspection.problems.is_empty() {
        let message = inspection.problems.join("; ");
        return Err(CommandError::new(
            command,
            opts.json,
            AppError::state_file_invalid(message),
        ));
    }

    let state = inspection
        .parsed
        .map(|state| state.redacted())
        .ok_or_else(|| {
            CommandError::new(
                command,
                opts.json,
                AppError::state_file_invalid("state file could not be parsed"),
            )
        })?;

    let data = StateShowData {
        exists: true,
        path: path.display().to_string(),
        state: Some(state.clone()),
    };

    if opts.json {
        output::print_json_success(command, &data);
    } else {
        println!("exists: true");
        println!("path: {}", data.path);
        output::print_pretty_json(serde_json::to_value(state).expect("serialize state summary"));
    }

    Ok(())
}
