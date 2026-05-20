use std::io::IsTerminal;

use dialoguer::{Select, theme::ColorfulTheme};
use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::app::AppContext;
use crate::args::ModelCommand;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::output;
use crate::state::model::AuthState;

const SHARED_TEXT_MODEL_KEY: &str = "text";
const SWITCHABLE_TASK_KEYS: &[&str] = &["chat", "search"];
const MODEL_CATALOG: &[&str] = &[
    "grok-4.3",
    "grok-4.20-reasoning",
    "grok-4.20-0309-reasoning",
];

#[derive(Debug, Clone, Serialize)]
struct ModelCommandData {
    provider: String,
    selected_model: String,
    selected: Map<String, Value>,
    catalog: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ModelSelectionData {
    provider: String,
    model: String,
    selected: Map<String, Value>,
    catalog: Vec<String>,
}

pub fn execute(ctx: &AppContext, opts: ModelCommand) -> CommandResult {
    let command = "model";
    let (mut state, path) = load_state(ctx, opts.common.auth_file.as_deref())
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if let Some(task) = opts.task.as_deref() {
        normalize_task_key(Some(task))
            .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    }

    if let Some(model) = opts.model.as_deref() {
        let model = normalize_model(model)
            .map_err(|error| CommandError::new(command, opts.common.json, error))?;
        set_shared_text_model(&mut state, model);
        ctx.state_store
            .write_state(&path, &state)
            .map_err(|error| CommandError::new(command, opts.common.json, error))?;

        let data = ModelSelectionData {
            provider: state.provider.clone(),
            model: model.to_string(),
            selected: selected_models(&state),
            catalog: catalog_entries(),
        };

        if opts.common.json {
            output::print_json_success(command, &data);
        } else {
            print_selected_model(&data);
        }
        return Ok(());
    }

    let data = ModelCommandData {
        provider: state.provider.clone(),
        selected_model: default_text_model(Some(&state)),
        selected: selected_models(&state),
        catalog: catalog_entries(),
    };

    if opts.common.json {
        output::print_json_success(command, &data);
        return Ok(());
    }

    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        if let Some(model) = prompt_for_model_selection(&data)
            .map_err(|error| CommandError::new(command, opts.common.json, error))?
        {
            set_shared_text_model(&mut state, &model);
            ctx.state_store
                .write_state(&path, &state)
                .map_err(|error| CommandError::new(command, opts.common.json, error))?;
            println!("Model switched to {model}.");
        }
    } else {
        print_model_catalog(&data);
    }

    Ok(())
}

pub fn default_model_for_task(state: Option<&AuthState>, task: &str, fallback: &str) -> String {
    let normalized_task = normalize_model_key_alias(task);
    if !SWITCHABLE_TASK_KEYS.contains(&normalized_task) {
        return fallback.to_string();
    }

    let Some(state) = state else {
        return fallback.to_string();
    };

    model_from_state(state, normalized_task).unwrap_or_else(|| fallback.to_string())
}

fn default_text_model(state: Option<&AuthState>) -> String {
    state
        .and_then(|state| model_from_state(state, "chat"))
        .unwrap_or_else(|| MODEL_CATALOG[0].to_string())
}

fn model_from_state(state: &AuthState, task: &str) -> Option<String> {
    state
        .metadata
        .get("default_models")
        .and_then(Value::as_object)
        .and_then(|models| {
            models
                .get(SHARED_TEXT_MODEL_KEY)
                .or_else(|| models.get(task))
                .or_else(|| legacy_model_key(task).and_then(|legacy| models.get(legacy)))
        })
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| MODEL_CATALOG.contains(value))
        .map(ToOwned::to_owned)
}

fn load_state(
    ctx: &AppContext,
    auth_file: Option<&std::path::Path>,
) -> Result<(AuthState, std::path::PathBuf), AppError> {
    let path = ctx.state_store.resolve_path(auth_file);
    let state = ctx.state_store.load_valid_state(&path)?;
    Ok((state, path))
}

fn normalize_task_key(task: Option<&str>) -> Result<&str, AppError> {
    let task = task.unwrap_or("chat").trim();
    let task = normalize_model_key_alias(task);
    SWITCHABLE_TASK_KEYS
        .iter()
        .copied()
        .find(|candidate| *candidate == task)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::InvalidArgs,
                format!(
                    "--command must be one of: {}",
                    SWITCHABLE_TASK_KEYS.join(", ")
                ),
            )
        })
}

fn normalize_model_key_alias(task: &str) -> &str {
    match task {
        "x-search" => "search",
        "image-gen" => "image",
        "video-gen" => "video",
        _ => task,
    }
}

fn legacy_model_key(task: &str) -> Option<&'static str> {
    match task {
        "search" => Some("x-search"),
        _ => None,
    }
}

fn normalize_model(model: &str) -> Result<&str, AppError> {
    let model = model.trim();
    if model.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--model must not be empty",
        ));
    }
    if !MODEL_CATALOG.contains(&model) {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            format!("--model must be one of: {}", MODEL_CATALOG.join(", ")),
        ));
    }
    Ok(model)
}

fn set_shared_text_model(state: &mut AuthState, model: &str) {
    let default_models = state
        .metadata
        .entry("default_models".to_string())
        .or_insert_with(|| json!({}));

    if !default_models.is_object() {
        *default_models = json!({});
    }

    if let Some(models) = default_models.as_object_mut() {
        models.insert(SHARED_TEXT_MODEL_KEY.to_string(), json!(model));
        models.insert("chat".to_string(), json!(model));
        models.insert("search".to_string(), json!(model));
        models.remove("x-search");
    }
}

fn selected_models(state: &AuthState) -> Map<String, Value> {
    let selected = default_text_model(Some(state));
    let mut data = Map::new();
    data.insert(SHARED_TEXT_MODEL_KEY.to_string(), json!(selected));
    data.insert("chat".to_string(), json!(selected));
    data.insert("search".to_string(), json!(selected));
    data
}

fn catalog_entries() -> Vec<String> {
    MODEL_CATALOG
        .iter()
        .map(|model| (*model).to_string())
        .collect()
}

fn print_model_catalog(data: &ModelCommandData) {
    println!("Grok Models");
    println!("Provider: {}", data.provider);
    println!("Selected: {}", data.selected_model);
    println!();
    for model in MODEL_CATALOG {
        let marker = if *model == data.selected_model {
            "*"
        } else {
            " "
        };
        println!("{marker} {model}");
    }
    println!("  exit");
    println!();
    println!("Run `grok-cli model --model <MODEL>` to save a shared chat/search model.");
}

fn print_selected_model(data: &ModelSelectionData) {
    println!("Model switched to {}.", data.model);
    println!("Provider: {}", data.provider);
    println!("Applies to: chat, search");
}

fn prompt_for_model_selection(data: &ModelCommandData) -> Result<Option<String>, AppError> {
    let mut items: Vec<String> = MODEL_CATALOG
        .iter()
        .map(|model| (*model).to_string())
        .collect();
    items.push("exit".to_string());
    let default_index = MODEL_CATALOG
        .iter()
        .position(|model| *model == data.selected_model)
        .unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Grok model")
        .items(&items)
        .default(default_index)
        .interact_opt()
        .map_err(|error| AppError::io(error.to_string()))?;

    let Some(index) = selection else {
        return Ok(None);
    };
    let Some(model) = items.get(index) else {
        return Ok(None);
    };
    if model == "exit" {
        return Ok(None);
    }

    Ok(Some(model.clone()))
}

#[cfg(test)]
mod tests {
    use super::{default_model_for_task, normalize_task_key};
    use crate::state::model::AuthState;
    use serde_json::json;

    #[test]
    fn normalize_task_key_rejects_unknown_task() {
        let error = normalize_task_key(Some("unknown")).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
    }

    #[test]
    fn default_model_for_task_reads_shared_text_override() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.metadata.insert(
            "default_models".to_string(),
            json!({
                "text": "grok-4.3",
                "chat": "grok-4.20-reasoning",
                "search": "grok-4.20-reasoning"
            }),
        );

        assert_eq!(
            default_model_for_task(Some(&state), "chat", "grok-4.20-reasoning"),
            "grok-4.3"
        );
        assert_eq!(
            default_model_for_task(Some(&state), "search", "grok-4.20-reasoning"),
            "grok-4.3"
        );
    }

    #[test]
    fn default_model_for_task_ignores_non_switchable_task_override() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.metadata.insert(
            "default_models".to_string(),
            json!({
                "text": "grok-4.3",
                "tts": "grok-tts"
            }),
        );

        assert_eq!(
            default_model_for_task(Some(&state), "tts", "grok-tts"),
            "grok-tts"
        );
    }

    #[test]
    fn default_model_for_task_reads_legacy_x_search_override() {
        let mut state = AuthState::empty("https://api.x.ai/v1".to_string());
        state.metadata.insert(
            "default_models".to_string(),
            json!({
                "x-search": "grok-4.3"
            }),
        );

        assert_eq!(
            default_model_for_task(Some(&state), "search", "grok-4.20-reasoning"),
            "grok-4.3"
        );
    }
}
