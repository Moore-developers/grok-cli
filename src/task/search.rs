use serde::Serialize;
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::args::XSearchOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::task::text_stream;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::{pricing, tracker};

const DEFAULT_X_SEARCH_MODEL: &str = "grok-4.3";

#[derive(Debug, Clone, Serialize)]
struct XSearchData {
    success: bool,
    provider: String,
    credential_source: String,
    tool: String,
    model: String,
    query: String,
    answer: String,
    citations: Vec<String>,
    inline_citations: Vec<String>,
}

pub fn execute(ctx: &AppContext, opts: XSearchOptions) -> CommandResult {
    let command = "search";
    validate_options(&opts).map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts.model.clone().unwrap_or_else(|| {
        model::default_model_for_task(state.as_ref(), "search", DEFAULT_X_SEARCH_MODEL)
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
    opts: XSearchOptions,
    model: &str,
    request: Value,
) -> CommandResult {
    let command = "search";
    let upstream = upstream::post_responses_api(
        ctx,
        opts.common.auth_file.as_deref(),
        &request,
        opts.timeout,
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = parse_x_search_response(&opts, model, &upstream)
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
        print_human_search_response(&data);
    }

    Ok(())
}

fn execute_stream(
    ctx: &AppContext,
    opts: XSearchOptions,
    model: &str,
    request: Value,
) -> CommandResult {
    let command = "search";
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

fn validate_options(opts: &XSearchOptions) -> Result<(), AppError> {
    if query_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "query must not be empty",
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

fn build_request(opts: &XSearchOptions, model: &str, stream: bool) -> Value {
    let mut tool = serde_json::Map::new();
    tool.insert("type".to_string(), json!("x_search"));

    if !opts.allowed_x_handles.is_empty() {
        tool.insert(
            "allowed_x_handles".to_string(),
            json!(opts.allowed_x_handles),
        );
    }
    if !opts.excluded_x_handles.is_empty() {
        tool.insert(
            "excluded_x_handles".to_string(),
            json!(opts.excluded_x_handles),
        );
    }
    if let Some(from_date) = opts.from_date.as_deref() {
        tool.insert("from_date".to_string(), json!(from_date));
    }
    if let Some(to_date) = opts.to_date.as_deref() {
        tool.insert("to_date".to_string(), json!(to_date));
    }
    if opts.enable_image_understanding {
        tool.insert("enable_image_understanding".to_string(), json!(true));
    }
    if opts.enable_video_understanding {
        tool.insert("enable_video_understanding".to_string(), json!(true));
    }

    json!({
        "model": model,
        "stream": stream,
        "input": [{
            "role": "user",
            "content": query_text(opts),
        }],
        "tools": [Value::Object(tool)],
        "tool_choice": "auto",
        "parallel_tool_calls": true,
        "store": false,
    })
}

fn parse_x_search_response(
    opts: &XSearchOptions,
    model: &str,
    upstream: &upstream::UpstreamResponseEnvelope,
) -> Result<XSearchData, AppError> {
    let output = upstream
        .response
        .get("output")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                "responses API payload is missing `output` array",
            )
        })?;

    let mut answer = None;
    let mut citations = Vec::new();

    for item in output {
        if item.get("type").and_then(|value| value.as_str()) != Some("message") {
            continue;
        }

        let Some(content) = item.get("content").and_then(|value| value.as_array()) else {
            continue;
        };
        for block in content {
            if block.get("type").and_then(|value| value.as_str()) != Some("output_text") {
                continue;
            }

            if answer.is_none() {
                answer = block
                    .get("text")
                    .and_then(|value| value.as_str())
                    .map(|value| value.trim().to_string());
            }

            if let Some(annotations) = block.get("annotations").and_then(|value| value.as_array()) {
                for annotation in annotations {
                    if let Some(url) = annotation.get("url").and_then(|value| value.as_str()) {
                        citations.push(url.to_string());
                    }
                }
            }
        }
    }

    let answer = answer.ok_or_else(|| {
        AppError::new(
            ErrorCode::RequestFailed,
            "responses API payload did not include a message answer",
        )
    })?;

    let inline_citations = citations.clone();

    Ok(XSearchData {
        success: true,
        provider: "xai".to_string(),
        credential_source: upstream.credential_source.clone(),
        tool: "x_search".to_string(),
        model: model.to_string(),
        query: query_text(opts).to_string(),
        answer,
        citations,
        inline_citations,
    })
}

fn query_text(opts: &XSearchOptions) -> &str {
    opts.query
        .as_deref()
        .or(opts.query_flag.as_deref())
        .unwrap_or("")
}

fn print_human_search_response(data: &XSearchData) {
    let answer = data.answer.trim();
    if !answer.is_empty() {
        println!("{answer}");
    } else {
        println!("No search answer returned.");
    }

    if !data.citations.is_empty() {
        println!();
        println!("Sources:");
        for (index, citation) in data.citations.iter().enumerate() {
            println!("{}. {citation}", index + 1);
        }
    }

    println!();
    println!("Model: {}", data.model);
    println!("Tool: {}", data.tool);
}

#[cfg(test)]
mod tests {
    use super::{build_request, parse_x_search_response, validate_options};
    use crate::args::{TaskCommonOptions, XSearchOptions};
    use crate::upstream::UpstreamResponseEnvelope;
    use serde_json::json;

    fn sample_opts() -> XSearchOptions {
        XSearchOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            query: Some("What are people saying about xAI on X today?".to_string()),
            query_flag: None,
            allowed_x_handles: vec!["xai".to_string()],
            excluded_x_handles: vec!["spam".to_string()],
            from_date: Some("2026-05-19".to_string()),
            to_date: Some("2026-05-20".to_string()),
            enable_image_understanding: true,
            enable_video_understanding: false,
            model: Some("grok-4.3".to_string()),
            stream: false,
            no_stream: false,
            raw_stream: false,
            timeout: Some(60),
        }
    }

    #[test]
    fn validate_options_rejects_empty_query() {
        let mut opts = sample_opts();
        opts.query = Some("   ".to_string());
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("query must not be empty"));
    }

    #[test]
    fn validate_options_rejects_too_many_allowed_handles() {
        let mut opts = sample_opts();
        opts.allowed_x_handles = (0..11).map(|i| format!("h{i}")).collect();
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--allowed-x-handle"));
    }

    #[test]
    fn build_request_includes_tool_filters() {
        let opts = sample_opts();
        let request = build_request(&opts, "grok-4.3", true);
        assert_eq!(request["model"], "grok-4.3");
        assert_eq!(request["stream"], true);
        assert_eq!(
            request["input"][0]["content"],
            "What are people saying about xAI on X today?"
        );
        assert_eq!(request["tools"][0]["type"], "x_search");
        assert_eq!(request["tools"][0]["allowed_x_handles"][0], "xai");
        assert_eq!(request["tools"][0]["excluded_x_handles"][0], "spam");
        assert_eq!(request["tools"][0]["from_date"], "2026-05-19");
        assert_eq!(request["tools"][0]["to_date"], "2026-05-20");
        assert_eq!(request["tools"][0]["enable_image_understanding"], true);
        assert!(request["tools"][0]["enable_video_understanding"].is_null());
    }

    #[test]
    fn parse_x_search_response_extracts_answer_and_citations() {
        let opts = sample_opts();
        let upstream = UpstreamResponseEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "output": [
                    {
                        "type": "message",
                        "content": [{
                            "type": "output_text",
                            "text": "Answer body[[1]](https://x.com/a)",
                            "annotations": [{
                                "type": "url_citation",
                                "url": "https://x.com/a",
                                "title": "1"
                            }]
                        }]
                    }
                ]
            }),
            usage: crate::upstream::ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed = parse_x_search_response(&opts, "grok-4.3", &upstream).unwrap();
        assert_eq!(parsed.credential_source, "xai-oauth");
        assert_eq!(parsed.tool, "x_search");
        assert_eq!(parsed.model, "grok-4.3");
        assert_eq!(parsed.citations, vec!["https://x.com/a".to_string()]);
        assert_eq!(parsed.inline_citations, vec!["https://x.com/a".to_string()]);
        assert!(parsed.answer.contains("Answer body"));
    }

    #[test]
    fn parse_x_search_response_rejects_missing_answer() {
        let opts = sample_opts();
        let upstream = UpstreamResponseEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "output": [{
                    "type": "reasoning",
                    "summary": []
                }]
            }),
            usage: crate::upstream::ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let error = parse_x_search_response(&opts, "grok-4.3", &upstream).unwrap_err();
        assert_eq!(error.code.as_str(), "request_failed");
        assert!(error.message.contains("did not include a message answer"));
    }
}
