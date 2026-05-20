use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{io, io::Write};

use serde_json::{Value, json};

use crate::app::AppContext;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::upstream;
use crate::usage::model::RateLimitsCapture;

pub struct StreamExecutionOutcome {
    pub credential_source: String,
    pub usage: upstream::ResponseUsageSummary,
    pub rate_limits: Option<RateLimitsCapture>,
}

#[derive(Debug, Clone, Copy)]
enum StreamRenderMode {
    HumanText,
    JsonEvents,
}

#[derive(Default)]
struct HumanStreamState {
    printed_text: bool,
    ends_with_newline: bool,
    delta_keys: HashSet<(String, u64)>,
}

pub fn should_stream(json_enabled: bool, force_stream: bool, no_stream: bool) -> bool {
    if no_stream {
        false
    } else if force_stream {
        true
    } else {
        !json_enabled
    }
}

fn stream_render_mode(raw_stream: bool) -> StreamRenderMode {
    if raw_stream {
        StreamRenderMode::JsonEvents
    } else {
        StreamRenderMode::HumanText
    }
}

pub fn execute_responses_stream(
    ctx: &AppContext,
    command: &'static str,
    json_mode: bool,
    raw_stream: bool,
    auth_file: Option<&Path>,
    request: &Value,
    timeout: Option<u64>,
) -> Result<StreamExecutionOutcome, CommandError> {
    let upstream = upstream::post_responses_stream_api(ctx, auth_file, request, timeout)
        .map_err(|error| CommandError::new(command, json_mode, error))?;

    let render_mode = stream_render_mode(raw_stream);
    let reader = BufReader::new(upstream.response);
    let mut current_event = None::<String>;
    let mut data_lines = Vec::<String>::new();
    let mut final_usage = upstream::ResponseUsageSummary::default();
    let mut human_state = HumanStreamState::default();

    for line in reader.lines() {
        let line = line.map_err(|error| {
            CommandError::new(
                command,
                json_mode,
                AppError::new(
                    ErrorCode::RequestFailed,
                    format!("failed to read responses stream: {error}"),
                ),
            )
        })?;

        if let Some(rest) = line.strip_prefix("event:") {
            current_event = Some(rest.trim().to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim().to_string());
            continue;
        }

        if line.trim().is_empty() {
            flush_pending_event(
                &mut current_event,
                &mut data_lines,
                &mut final_usage,
                render_mode,
                &mut human_state,
            )
            .map_err(|error| CommandError::new(command, json_mode, error))?;
        }
    }

    flush_pending_event(
        &mut current_event,
        &mut data_lines,
        &mut final_usage,
        render_mode,
        &mut human_state,
    )
    .map_err(|error| CommandError::new(command, json_mode, error))?;

    Ok(StreamExecutionOutcome {
        credential_source: upstream.credential_source,
        usage: final_usage,
        rate_limits: upstream.rate_limits,
    })
}

fn flush_pending_event(
    current_event: &mut Option<String>,
    data_lines: &mut Vec<String>,
    final_usage: &mut upstream::ResponseUsageSummary,
    render_mode: StreamRenderMode,
    human_state: &mut HumanStreamState,
) -> Result<(), AppError> {
    let Some(event_type) = current_event.take() else {
        data_lines.clear();
        return Ok(());
    };

    let payload = data_lines.join("\n");
    data_lines.clear();

    if event_type == "response.completed"
        && let Ok(parsed) = serde_json::from_str::<Value>(&payload)
        && let Some(response) = parsed.get("response")
    {
        *final_usage = extract_usage_summary_from_stream(response);
    }

    match render_mode {
        StreamRenderMode::HumanText => {
            render_human_stream_event(&event_type, &payload, human_state)
        }
        StreamRenderMode::JsonEvents => emit_json_stream_event(&event_type, &payload),
    }
}

fn extract_usage_summary_from_stream(response: &Value) -> upstream::ResponseUsageSummary {
    let usage = response.get("usage").cloned().unwrap_or(Value::Null);
    upstream::ResponseUsageSummary {
        input_tokens: usage
            .get("input_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(0),
        output_tokens: usage
            .get("output_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(0),
        cache_read_tokens: usage
            .get("input_tokens_details")
            .and_then(|details| details.get("cached_tokens"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0),
        cache_write_tokens: usage
            .get("input_tokens_details")
            .and_then(|details| details.get("cache_creation_tokens"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0),
        reasoning_tokens: usage
            .get("output_tokens_details")
            .and_then(|details| details.get("reasoning_tokens"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0),
    }
}

fn render_human_stream_event(
    event_type: &str,
    payload: &str,
    state: &mut HumanStreamState,
) -> Result<(), AppError> {
    if payload == "[DONE]" {
        if state.printed_text && !state.ends_with_newline {
            println!();
        }
        return Ok(());
    }

    match event_type {
        "response.created"
        | "response.in_progress"
        | "response.output_item.added"
        | "response.custom_tool_call_input.delta"
        | "response.custom_tool_call_input.done" => Ok(()),
        "response.output_text.delta" => {
            let parsed = serde_json::from_str::<Value>(payload).map_err(|error| {
                AppError::new(
                    ErrorCode::RequestFailed,
                    format!("failed to decode stream event payload: {error}"),
                )
            })?;
            let delta = parsed
                .get("delta")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if !delta.is_empty() {
                let key = stream_text_key(&parsed);
                state.delta_keys.insert(key);
                print!("{delta}");
                io::stdout().flush().map_err(|error| {
                    AppError::new(
                        ErrorCode::RequestFailed,
                        format!("failed to flush stream output: {error}"),
                    )
                })?;
                state.printed_text = true;
                state.ends_with_newline = delta.ends_with('\n');
            }
            Ok(())
        }
        "response.output_text.done" => {
            let parsed = serde_json::from_str::<Value>(payload).map_err(|error| {
                AppError::new(
                    ErrorCode::RequestFailed,
                    format!("failed to decode stream event payload: {error}"),
                )
            })?;
            let key = stream_text_key(&parsed);
            if state.delta_keys.contains(&key) {
                return Ok(());
            }

            let text = parsed
                .get("text")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if !text.is_empty() {
                print!("{text}");
                io::stdout().flush().map_err(|error| {
                    AppError::new(
                        ErrorCode::RequestFailed,
                        format!("failed to flush stream output: {error}"),
                    )
                })?;
                state.printed_text = true;
                state.ends_with_newline = text.ends_with('\n');
            }
            Ok(())
        }
        "response.completed" => {
            if state.printed_text && !state.ends_with_newline {
                println!();
                state.ends_with_newline = true;
            }
            Ok(())
        }
        "response.failed" => {
            if state.printed_text && !state.ends_with_newline {
                println!();
            }

            let parsed = serde_json::from_str::<Value>(payload).map_err(|error| {
                AppError::new(
                    ErrorCode::RequestFailed,
                    format!("failed to decode stream event payload: {error}"),
                )
            })?;
            let message = parsed
                .get("response")
                .and_then(|value| value.get("error"))
                .and_then(|value| value.get("message"))
                .and_then(|value| value.as_str())
                .or_else(|| {
                    parsed
                        .get("error")
                        .and_then(|value| value.get("message"))
                        .and_then(|value| value.as_str())
                })
                .unwrap_or("responses stream failed");

            Err(AppError::new(ErrorCode::RequestFailed, message))
        }
        _ => Ok(()),
    }
}

fn stream_text_key(payload: &Value) -> (String, u64) {
    let item_id = payload
        .get("item_id")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let content_index = payload
        .get("content_index")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    (item_id, content_index)
}

fn emit_json_stream_event(event_type: &str, payload: &str) -> Result<(), AppError> {
    if payload == "[DONE]" {
        return Ok(());
    }

    let parsed = serde_json::from_str::<Value>(payload).map_err(|error| {
        AppError::new(
            ErrorCode::RequestFailed,
            format!("failed to decode stream event payload: {error}"),
        )
    })?;

    let normalized = match event_type {
        "response.output_text.delta" => parsed,
        "response.output_text.done" => parsed,
        "response.output_item.done" => {
            if parsed
                .get("item")
                .and_then(|item| item.get("type"))
                .and_then(|value| value.as_str())
                == Some("function_call")
            {
                let mut updated = parsed.clone();
                if let Some(item) = updated.get_mut("item") {
                    *item = json!({
                        "type": "function_call",
                        "call_id": item.get("call_id").and_then(|value| value.as_str()).unwrap_or("call_unknown"),
                        "name": item.get("name").and_then(|value| value.as_str()).unwrap_or_default(),
                        "arguments": item.get("arguments").and_then(|value| value.as_str()).unwrap_or("{}"),
                        "status": item.get("status").and_then(|value| value.as_str()).unwrap_or("completed")
                    });
                }
                updated
            } else {
                parsed
            }
        }
        "response.completed" => parsed,
        "response.failed" => parsed,
        _ => parsed,
    };

    println!("event: {event_type}");
    println!(
        "data: {}",
        serde_json::to_string(&normalized).map_err(|error| {
            AppError::new(
                ErrorCode::RequestFailed,
                format!("failed to serialize normalized stream event: {error}"),
            )
        })?
    );
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        HumanStreamState, StreamRenderMode, render_human_stream_event, should_stream,
        stream_render_mode,
    };

    #[test]
    fn default_human_mode_streams() {
        assert!(should_stream(false, false, false));
    }

    #[test]
    fn json_mode_disables_default_streaming() {
        assert!(!should_stream(true, false, false));
    }

    #[test]
    fn explicit_stream_overrides_json_mode() {
        assert!(should_stream(true, true, false));
    }

    #[test]
    fn explicit_no_stream_wins() {
        assert!(!should_stream(false, true, true));
    }

    #[test]
    fn raw_stream_uses_event_rendering() {
        assert!(matches!(
            stream_render_mode(true),
            StreamRenderMode::JsonEvents
        ));
    }

    #[test]
    fn formatted_stream_uses_text_rendering() {
        assert!(matches!(
            stream_render_mode(false),
            StreamRenderMode::HumanText
        ));
    }

    #[test]
    fn human_stream_renders_delta_text_without_event_wrapper() {
        let mut state = HumanStreamState::default();
        render_human_stream_event(
            "response.output_text.delta",
            r#"{"item_id":"msg_1","content_index":0,"delta":"hello"}"#,
            &mut state,
        )
        .unwrap();
        assert!(state.printed_text);
        assert!(!state.ends_with_newline);
    }

    #[test]
    fn human_stream_ignores_done_text_when_delta_already_printed() {
        let mut state = HumanStreamState::default();
        render_human_stream_event(
            "response.output_text.delta",
            r#"{"item_id":"msg_1","content_index":0,"delta":"hello"}"#,
            &mut state,
        )
        .unwrap();
        render_human_stream_event(
            "response.output_text.done",
            r#"{"item_id":"msg_1","content_index":0,"text":"hello"}"#,
            &mut state,
        )
        .unwrap();
        assert!(state.delta_keys.contains(&(String::from("msg_1"), 0)));
    }
}
