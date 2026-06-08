use serde::Serialize;
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::args::ChatOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::task::text_stream;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::{pricing, tracker};

const DEFAULT_CHAT_MODEL: &str = "grok-4.3";

#[derive(Debug, Clone, Serialize)]
struct ChatData {
    provider: String,
    model: String,
    protocol: String,
    output_text: String,
    finish_reason: String,
    tool_calls: Vec<Value>,
}

pub fn execute(ctx: &AppContext, opts: ChatOptions) -> CommandResult {
    let command = "chat";
    validate_options(&opts).map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts.model.clone().unwrap_or_else(|| {
        model::default_model_for_task(state.as_ref(), "chat", DEFAULT_CHAT_MODEL)
    });
    let stream = text_stream::should_stream(opts.common.json, opts.stream, opts.no_stream);
    let request = build_request(&opts, &model, stream);

    if stream {
        execute_stream(ctx, opts, &model, request)
    } else {
        execute_non_stream(ctx, opts, &model, request)
    }
}

fn execute_non_stream(
    ctx: &AppContext,
    opts: ChatOptions,
    model: &str,
    request: Value,
) -> CommandResult {
    let command = "chat";
    let upstream = upstream::post_responses_api(
        ctx,
        opts.common.auth_file.as_deref(),
        &request,
        opts.timeout,
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = parse_chat_response(model, &upstream.response)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let estimated_cost_micro_usd = pricing::estimate_text_cost_micro_usd(
        model,
        upstream.usage.input_tokens,
        upstream.usage.output_tokens,
    )
    .unwrap_or(0);
    tracker::record_usage(
        ctx,
        opts.common.auth_file.as_deref(),
        &upstream.credential_source,
        UsageDelta {
            provider: upstream.credential_source.clone(),
            command: command.to_string(),
            model: Some(model.to_string()),
            input_tokens: upstream.usage.input_tokens,
            output_tokens: upstream.usage.output_tokens,
            cache_read_tokens: upstream.usage.cache_read_tokens,
            cache_write_tokens: upstream.usage.cache_write_tokens,
            reasoning_tokens: upstream.usage.reasoning_tokens,
            estimated_cost_micro_usd,
            context_window_tokens: None,
            rate_limits: upstream.rate_limits.clone(),
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if opts.common.json {
        output::print_json_success(command, &data);
    } else {
        print_human_chat_response(&data);
    }

    Ok(())
}

fn execute_stream(
    ctx: &AppContext,
    opts: ChatOptions,
    model: &str,
    request: Value,
) -> CommandResult {
    let command = "chat";
    let outcome = text_stream::execute_responses_stream(
        ctx,
        command,
        opts.common.json,
        opts.raw_stream,
        opts.common.auth_file.as_deref(),
        &request,
        opts.timeout,
    )?;

    let estimated_cost_micro_usd = pricing::estimate_text_cost_micro_usd(
        model,
        outcome.usage.input_tokens,
        outcome.usage.output_tokens,
    )
    .unwrap_or(0);
    tracker::record_usage(
        ctx,
        opts.common.auth_file.as_deref(),
        &outcome.credential_source,
        UsageDelta {
            provider: outcome.credential_source.clone(),
            command: command.to_string(),
            model: Some(model.to_string()),
            input_tokens: outcome.usage.input_tokens,
            output_tokens: outcome.usage.output_tokens,
            cache_read_tokens: outcome.usage.cache_read_tokens,
            cache_write_tokens: outcome.usage.cache_write_tokens,
            reasoning_tokens: outcome.usage.reasoning_tokens,
            estimated_cost_micro_usd,
            context_window_tokens: None,
            rate_limits: outcome.rate_limits,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    Ok(())
}

fn validate_options(opts: &ChatOptions) -> Result<(), AppError> {
    if prompt_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "prompt must not be empty",
        ));
    }

    if opts.allowed_domains.len() > 10 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--allowed-domain supports at most 10 values",
        ));
    }

    if opts.excluded_domains.len() > 10 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--excluded-domain supports at most 10 values",
        ));
    }

    if opts.allowed_x_handles.len() > 10 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--allowed-x-handle supports at most 10 values",
        ));
    }

    if opts.excluded_x_handles.len() > 10 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--excluded-x-handle supports at most 10 values",
        ));
    }

    Ok(())
}

fn build_request(opts: &ChatOptions, model: &str, stream: bool) -> Value {
    let mut input = Vec::new();
    input.push(json!({
        "role": "user",
        "content": prompt_text(opts)
    }));

    let mut tools = Vec::new();
    if !opts.no_web_search {
        let mut web_search = serde_json::Map::new();
        web_search.insert("type".to_string(), json!("web_search"));

        let mut filters = serde_json::Map::new();
        if !opts.allowed_domains.is_empty() {
            filters.insert("allowed_domains".to_string(), json!(opts.allowed_domains));
        }
        if !opts.excluded_domains.is_empty() {
            filters.insert("excluded_domains".to_string(), json!(opts.excluded_domains));
        }
        if !filters.is_empty() {
            web_search.insert("filters".to_string(), Value::Object(filters));
        }
        if opts.enable_image_understanding {
            web_search.insert("enable_image_understanding".to_string(), json!(true));
        }

        tools.push(Value::Object(web_search));
    }
    if opts.with_x_search {
        let mut x_search = serde_json::Map::new();
        x_search.insert("type".to_string(), json!("x_search"));

        if !opts.allowed_x_handles.is_empty() {
            x_search.insert(
                "allowed_x_handles".to_string(),
                json!(opts.allowed_x_handles),
            );
        }
        if !opts.excluded_x_handles.is_empty() {
            x_search.insert(
                "excluded_x_handles".to_string(),
                json!(opts.excluded_x_handles),
            );
        }
        if let Some(from_date) = opts.from_date.as_deref() {
            x_search.insert("from_date".to_string(), json!(from_date));
        }
        if let Some(to_date) = opts.to_date.as_deref() {
            x_search.insert("to_date".to_string(), json!(to_date));
        }
        if opts.enable_image_understanding {
            x_search.insert("enable_image_understanding".to_string(), json!(true));
        }
        if opts.enable_video_understanding {
            x_search.insert("enable_video_understanding".to_string(), json!(true));
        }

        tools.push(Value::Object(x_search));
    }

    let mut request = json!({
        "model": model,
        "input": input,
        "stream": stream,
        "store": false
    });

    if let Some(system) = opts.system.as_deref() {
        request["instructions"] = json!(system);
    }

    if !tools.is_empty() {
        request["tools"] = Value::Array(tools);
        request["tool_choice"] = json!("auto");
        request["parallel_tool_calls"] = json!(true);
    }

    request
}

fn prompt_text(opts: &ChatOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn parse_chat_response(model: &str, response: &Value) -> Result<ChatData, AppError> {
    let output = response
        .get("output")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                "responses API payload is missing `output` array",
            )
        })?;

    let mut output_text = String::new();
    let mut tool_calls = Vec::new();

    for item in output {
        match item.get("type").and_then(|value| value.as_str()) {
            Some("message") => {
                if let Some(content) = item.get("content").and_then(|value| value.as_array()) {
                    for block in content {
                        if block.get("type").and_then(|value| value.as_str()) == Some("output_text")
                            && let Some(text) = block.get("text").and_then(|value| value.as_str())
                        {
                            output_text.push_str(text);
                        }
                    }
                }
            }
            Some("function_call") => {
                tool_calls.push(normalize_tool_call(item));
            }
            _ => {}
        }
    }

    let finish_reason = if tool_calls.is_empty() {
        "stop".to_string()
    } else {
        "tool_calls".to_string()
    };

    Ok(ChatData {
        provider: "xai-oauth".to_string(),
        model: model.to_string(),
        protocol: "codex_responses".to_string(),
        output_text,
        finish_reason,
        tool_calls,
    })
}

fn normalize_tool_call(item: &Value) -> Value {
    json!({
        "id": item.get("call_id").and_then(|value| value.as_str()).unwrap_or("call_unknown"),
        "type": "function",
        "function": {
            "name": item.get("name").and_then(|value| value.as_str()).unwrap_or_default(),
            "arguments": item.get("arguments").and_then(|value| value.as_str()).unwrap_or("{}")
        }
    })
}

fn print_human_chat_response(data: &ChatData) {
    let text = data.output_text.trim();
    if !text.is_empty() {
        println!("{text}");
    } else if !data.tool_calls.is_empty() {
        println!("Tool calls:");
        for call in &data.tool_calls {
            let name = call
                .get("function")
                .and_then(|function| function.get("name"))
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let arguments = call
                .get("function")
                .and_then(|function| function.get("arguments"))
                .and_then(|value| value.as_str())
                .unwrap_or("{}");
            println!("- {name}: {arguments}");
        }
    } else {
        println!("No response text returned.");
    }

    println!();
    println!("Model: {}", data.model);
    println!("Finish: {}", data.finish_reason);
}

#[cfg(test)]
mod tests {
    use super::{build_request, normalize_tool_call, parse_chat_response, validate_options};
    use crate::args::{ChatOptions, TaskCommonOptions};
    use serde_json::json;

    fn sample_opts() -> ChatOptions {
        ChatOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("Say hello".to_string()),
            prompt_flag: None,
            system: Some("You are helpful".to_string()),
            model: Some("grok-4.3".to_string()),
            no_web_search: false,
            with_x_search: false,
            allowed_domains: Vec::new(),
            excluded_domains: Vec::new(),
            enable_image_understanding: false,
            allowed_x_handles: Vec::new(),
            excluded_x_handles: Vec::new(),
            from_date: None,
            to_date: None,
            enable_video_understanding: false,
            stream: false,
            no_stream: false,
            raw_stream: false,
            timeout: Some(60),
        }
    }

    #[test]
    fn validate_options_rejects_empty_prompt() {
        let mut opts = sample_opts();
        opts.prompt = Some("   ".to_string());
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
    }

    #[test]
    fn build_request_includes_instructions_stream_and_default_web_search() {
        let opts = sample_opts();
        let request = build_request(&opts, "grok-4.3", true);
        assert_eq!(request["model"], "grok-4.3");
        assert_eq!(request["stream"], true);
        assert_eq!(request["store"], false);
        assert_eq!(request["instructions"], "You are helpful");
        assert_eq!(request["input"][0]["role"], "user");
        assert_eq!(request["tools"][0]["type"], "web_search");
        assert_eq!(request["tool_choice"], "auto");
        assert_eq!(request["parallel_tool_calls"], true);
    }

    #[test]
    fn build_request_omits_tools_when_web_search_is_disabled() {
        let mut opts = sample_opts();
        opts.no_web_search = true;
        let request = build_request(&opts, "grok-4.3", false);
        assert!(request.get("tools").is_none());
    }

    #[test]
    fn build_request_supports_combined_web_and_x_search_tools() {
        let mut opts = sample_opts();
        opts.with_x_search = true;
        let request = build_request(&opts, "grok-4.3", false);
        assert_eq!(request["tools"][0]["type"], "web_search");
        assert_eq!(request["tools"][1]["type"], "x_search");
    }

    #[test]
    fn build_request_maps_web_search_filters_to_xai_shape() {
        let mut opts = sample_opts();
        opts.allowed_domains = vec!["nature.com".to_string()];
        opts.excluded_domains = vec!["example.com".to_string()];
        opts.enable_image_understanding = true;
        let request = build_request(&opts, "grok-4.3", false);
        assert_eq!(
            request["tools"][0]["filters"]["allowed_domains"][0],
            "nature.com"
        );
        assert_eq!(
            request["tools"][0]["filters"]["excluded_domains"][0],
            "example.com"
        );
        assert_eq!(request["tools"][0]["enable_image_understanding"], true);
    }

    #[test]
    fn build_request_maps_x_search_filters_to_xai_shape() {
        let mut opts = sample_opts();
        opts.with_x_search = true;
        opts.allowed_x_handles = vec!["xai".to_string()];
        opts.from_date = Some("2026-05-19".to_string());
        opts.to_date = Some("2026-05-20".to_string());
        opts.enable_video_understanding = true;
        let request = build_request(&opts, "grok-4.3", false);
        assert_eq!(request["tools"][1]["allowed_x_handles"][0], "xai");
        assert_eq!(request["tools"][1]["from_date"], "2026-05-19");
        assert_eq!(request["tools"][1]["to_date"], "2026-05-20");
        assert_eq!(request["tools"][1]["enable_video_understanding"], true);
    }

    #[test]
    fn parse_chat_response_extracts_output_text() {
        let parsed = parse_chat_response(
            "grok-4.3",
            &json!({
                "output": [{
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": "hello"
                    }]
                }]
            }),
        )
        .unwrap();
        assert_eq!(parsed.output_text, "hello");
        assert_eq!(parsed.finish_reason, "stop");
    }

    #[test]
    fn parse_chat_response_normalizes_tool_calls() {
        let parsed = parse_chat_response(
            "grok-4.3",
            &json!({
                "output": [{
                    "type": "function_call",
                    "call_id": "call_123",
                    "name": "terminal",
                    "arguments": "{\"command\":\"ls\"}"
                }]
            }),
        )
        .unwrap();
        assert_eq!(parsed.finish_reason, "tool_calls");
        assert_eq!(parsed.tool_calls[0]["id"], "call_123");
    }

    #[test]
    fn normalize_tool_call_matches_openai_shape() {
        let call = normalize_tool_call(&json!({
            "call_id": "call_1",
            "name": "terminal",
            "arguments": "{\"command\":\"pwd\"}"
        }));
        assert_eq!(call["type"], "function");
        assert_eq!(call["function"]["name"], "terminal");
    }
}
