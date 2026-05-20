use std::fs;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::args::ImageGenOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::tracker;

const DEFAULT_IMAGE_MODEL: &str = "grok-imagine-image";
const IMAGES_GENERATIONS_PATH: &str = "/images/generations";

#[derive(Debug, Clone, Serialize)]
struct ImageGenData {
    provider: String,
    credential_source: String,
    model: String,
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    extra: Value,
}

pub fn execute(ctx: &AppContext, opts: ImageGenOptions) -> CommandResult {
    let command = "image";
    validate_options(&opts).map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts.model.clone().unwrap_or_else(|| {
        model::default_model_for_task(state.as_ref(), "image", DEFAULT_IMAGE_MODEL)
    });
    let request = build_request(&opts, &model);
    let upstream = upstream::post_json_api_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        IMAGES_GENERATIONS_PATH,
        &request,
        opts.timeout,
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = parse_image_response(&opts, &model, &upstream)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    tracker::record_usage(
        ctx,
        opts.common.auth_file.as_deref(),
        &upstream.credential_source,
        UsageDelta {
            provider: upstream.credential_source.clone(),
            command: command.to_string(),
            model: Some(model.clone()),
            rate_limits: upstream.rate_limits.clone(),
            ..UsageDelta::default()
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if opts.common.json {
        output::print_json_success(command, &data);
    } else {
        println!("provider: {}", data.provider);
        println!("credential_source: {}", data.credential_source);
        println!("model: {}", data.model);
        println!("image: {}", data.image);
        if let Some(aspect_ratio) = &data.aspect_ratio {
            println!("aspect_ratio: {aspect_ratio}");
        }
        println!("extra: {}", data.extra);
    }

    Ok(())
}

fn validate_options(opts: &ImageGenOptions) -> Result<(), AppError> {
    if prompt_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "prompt must not be empty",
        ));
    }

    Ok(())
}

fn build_request(opts: &ImageGenOptions, model: &str) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(prompt_text(opts)));

    if let Some(aspect_ratio) = opts.aspect_ratio.as_deref() {
        body.insert("aspect_ratio".to_string(), json!(aspect_ratio));
    }
    if let Some(resolution) = opts.resolution.as_deref() {
        body.insert("resolution".to_string(), json!(resolution));
    }
    if opts.output_file.is_some() {
        body.insert("response_format".to_string(), json!("b64_json"));
    }

    Value::Object(body)
}

fn prompt_text(opts: &ImageGenOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn parse_image_response(
    opts: &ImageGenOptions,
    model: &str,
    upstream: &upstream::UpstreamJsonEnvelope,
) -> Result<ImageGenData, AppError> {
    let image_value = if let Some(output_file) = opts.output_file.as_deref() {
        let image_b64 = extract_image_b64(&upstream.response)?;
        save_base64_image(output_file, &image_b64)?;
        output_file.display().to_string()
    } else {
        extract_image_reference(&upstream.response)?
    };

    Ok(ImageGenData {
        provider: "xai".to_string(),
        credential_source: upstream.credential_source.clone(),
        model: model.to_string(),
        image: image_value,
        aspect_ratio: opts.aspect_ratio.clone(),
        extra: json!({
            "resolution": opts.resolution.clone()
        }),
    })
}

fn extract_image_reference(response: &Value) -> Result<String, AppError> {
    if let Some(url) = response.get("url").and_then(|value| value.as_str()) {
        return Ok(url.to_string());
    }

    if let Some(value) = response
        .get("data")
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
    {
        if let Some(url) = value.get("url").and_then(|value| value.as_str()) {
            return Ok(url.to_string());
        }
        if let Some(b64) = value.get("b64_json").and_then(|value| value.as_str()) {
            return Ok(format!("data:image/png;base64,{b64}"));
        }
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        "images API payload did not include an image URL or data",
    ))
}

fn extract_image_b64(response: &Value) -> Result<String, AppError> {
    if let Some(b64) = response.get("b64_json").and_then(|value| value.as_str()) {
        return Ok(b64.to_string());
    }

    if let Some(b64) = response
        .get("data")
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
        .and_then(|item| item.get("b64_json"))
        .and_then(|value| value.as_str())
    {
        return Ok(b64.to_string());
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        "images API payload did not include `b64_json` for file output",
    ))
}

fn save_base64_image(path: &Path, image_b64: &str) -> Result<(), AppError> {
    let bytes = BASE64_STANDARD.decode(image_b64).map_err(|error| {
        AppError::new(
            ErrorCode::RequestFailed,
            format!("failed to decode base64 image payload: {error}"),
        )
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::io(format!(
                "failed to create image output directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    fs::write(path, bytes).map_err(|error| {
        AppError::io(format!(
            "failed to write image output file {}: {error}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_request, extract_image_b64, extract_image_reference, parse_image_response,
        validate_options,
    };
    use crate::args::{ImageGenOptions, TaskCommonOptions};
    use crate::upstream::{ResponseUsageSummary, UpstreamJsonEnvelope};
    use serde_json::json;
    use tempfile::tempdir;

    fn sample_opts() -> ImageGenOptions {
        ImageGenOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("Draw a skyline".to_string()),
            prompt_flag: None,
            model: Some("grok-imagine-image".to_string()),
            aspect_ratio: Some("16:9".to_string()),
            resolution: Some("1k".to_string()),
            output_file: None,
            timeout: Some(60),
        }
    }

    #[test]
    fn validate_options_rejects_empty_prompt() {
        let mut opts = sample_opts();
        opts.prompt = Some("   ".to_string());
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("prompt must not be empty"));
    }

    #[test]
    fn build_request_uses_remote_url_by_default() {
        let opts = sample_opts();
        let request = build_request(&opts, "grok-imagine-image");
        assert_eq!(request["model"], "grok-imagine-image");
        assert_eq!(request["prompt"], "Draw a skyline");
        assert_eq!(request["aspect_ratio"], "16:9");
        assert_eq!(request["resolution"], "1k");
        assert!(request["response_format"].is_null());
    }

    #[test]
    fn build_request_switches_to_b64_when_output_file_is_requested() {
        let mut opts = sample_opts();
        opts.output_file = Some(std::path::PathBuf::from("/tmp/image.png"));
        let request = build_request(&opts, "grok-imagine-image");
        assert_eq!(request["response_format"], "b64_json");
    }

    #[test]
    fn parse_image_response_prefers_url_when_not_saving_file() {
        let opts = sample_opts();
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "data": [{
                    "url": "https://cdn.x.ai/image.png"
                }]
            }),
            usage: ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed = parse_image_response(&opts, "grok-imagine-image", &upstream).unwrap();
        assert_eq!(parsed.image, "https://cdn.x.ai/image.png");
        assert_eq!(parsed.credential_source, "xai-oauth");
    }

    #[test]
    fn parse_image_response_writes_output_file_when_requested() {
        let temp = tempdir().unwrap();
        let image_path = temp.path().join("image.bin");
        let mut opts = sample_opts();
        opts.output_file = Some(image_path.clone());
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "data": [{
                    "b64_json": "aGVsbG8="
                }]
            }),
            usage: ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed = parse_image_response(&opts, "grok-imagine-image", &upstream).unwrap();
        assert_eq!(parsed.image, image_path.display().to_string());
        assert_eq!(std::fs::read(&image_path).unwrap(), b"hello");
    }

    #[test]
    fn extract_image_reference_rejects_missing_payload() {
        let error = extract_image_reference(&json!({"data": []})).unwrap_err();
        assert_eq!(error.code.as_str(), "request_failed");
    }

    #[test]
    fn extract_image_b64_rejects_missing_payload() {
        let error = extract_image_b64(&json!({"data": []})).unwrap_err();
        assert_eq!(error.code.as_str(), "request_failed");
    }
}
