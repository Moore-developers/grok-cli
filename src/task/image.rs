use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::args::{ImageEditOptions, ImageGenOptions};
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::tracker;

const DEFAULT_IMAGE_MODEL: &str = "grok-imagine-image";
const IMAGES_GENERATIONS_PATH: &str = "/images/generations";
const IMAGES_EDITS_PATH: &str = "/images/edits";
const DEFAULT_IMAGE_COUNT: u32 = 1;
const MAX_IMAGE_COUNT: u32 = 10;
const MAX_EDIT_IMAGE_COUNT: usize = 3;
const RESPONSE_FORMAT_URL: &str = "url";
const RESPONSE_FORMAT_B64_JSON: &str = "b64_json";

#[derive(Debug, Clone, Serialize)]
struct ImageGenData {
    provider: String,
    credential_source: String,
    model: String,
    image: String,
    images: Vec<String>,
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
        println!("image_count: {}", data.images.len());
        if data.images.len() > 1 {
            println!("images:");
            for image in &data.images {
                println!("- {image}");
            }
        }
        if let Some(aspect_ratio) = &data.aspect_ratio {
            println!("aspect_ratio: {aspect_ratio}");
        }
        println!("extra: {}", data.extra);
    }

    Ok(())
}

pub fn execute_edit(ctx: &AppContext, opts: ImageEditOptions) -> CommandResult {
    let command = "image-edit";
    validate_edit_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts.model.clone().unwrap_or_else(|| {
        model::default_model_for_task(state.as_ref(), "image", DEFAULT_IMAGE_MODEL)
    });
    let request = build_edit_request(&opts, &model)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let upstream = upstream::post_json_api_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        IMAGES_EDITS_PATH,
        &request,
        opts.timeout,
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = parse_image_edit_response(&opts, &model, &upstream)
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
        println!("image_count: {}", data.images.len());
        if data.images.len() > 1 {
            println!("images:");
            for image in &data.images {
                println!("- {image}");
            }
        }
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

    let count = image_count(opts);
    if !(1..=MAX_IMAGE_COUNT).contains(&count) {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            format!("--count must be between 1 and {MAX_IMAGE_COUNT}"),
        ));
    }

    if opts.output_file.is_some() && count > 1 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--output-file can only be used with --count 1; use --output-dir for multiple images",
        ));
    }

    if opts.output_file.is_some() && opts.output_dir.is_some() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--output-file cannot be combined with --output-dir",
        ));
    }

    if let Some(response_format) = opts.response_format.as_deref() {
        let response_format = response_format.trim();
        if !matches!(
            response_format,
            RESPONSE_FORMAT_URL | RESPONSE_FORMAT_B64_JSON
        ) {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                "--response-format must be url or b64_json",
            ));
        }
        if response_format == RESPONSE_FORMAT_URL
            && (opts.output_file.is_some() || opts.output_dir.is_some())
        {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                "--output-file and --output-dir require --response-format b64_json",
            ));
        }
    }

    Ok(())
}

fn validate_edit_options(opts: &ImageEditOptions) -> Result<(), AppError> {
    if edit_prompt_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "prompt must not be empty",
        ));
    }
    if opts.images.is_empty() {
        return Err(AppError::new(ErrorCode::InvalidArgs, "--image is required"));
    }
    if opts.images.len() > MAX_EDIT_IMAGE_COUNT {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            format!("--image supports at most {MAX_EDIT_IMAGE_COUNT} values"),
        ));
    }
    if let Some(response_format) = opts.response_format.as_deref() {
        let response_format = response_format.trim();
        if !matches!(
            response_format,
            RESPONSE_FORMAT_URL | RESPONSE_FORMAT_B64_JSON
        ) {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                "--response-format must be url or b64_json",
            ));
        }
        if response_format == RESPONSE_FORMAT_URL && opts.output_file.is_some() {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                "--output-file requires --response-format b64_json",
            ));
        }
    }

    Ok(())
}

fn build_request(opts: &ImageGenOptions, model: &str) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(prompt_text(opts)));
    body.insert("n".to_string(), json!(image_count(opts)));

    if let Some(aspect_ratio) = opts.aspect_ratio.as_deref() {
        body.insert("aspect_ratio".to_string(), json!(aspect_ratio));
    }
    if let Some(resolution) = opts.resolution.as_deref() {
        body.insert("resolution".to_string(), json!(resolution));
    }
    if let Some(response_format) = image_response_format(opts) {
        body.insert("response_format".to_string(), json!(response_format));
    }

    Value::Object(body)
}

fn build_edit_request(opts: &ImageEditOptions, model: &str) -> Result<Value, AppError> {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(edit_prompt_text(opts)));
    let images = build_edit_image_inputs(&opts.images)?;
    if images.len() == 1 {
        body.insert("image".to_string(), images[0].clone());
    } else {
        body.insert("images".to_string(), Value::Array(images));
    }

    if let Some(aspect_ratio) = opts.aspect_ratio.as_deref() {
        body.insert("aspect_ratio".to_string(), json!(aspect_ratio));
    }
    if let Some(resolution) = opts.resolution.as_deref() {
        body.insert("resolution".to_string(), json!(resolution));
    }
    if let Some(response_format) = image_edit_response_format(opts) {
        body.insert("response_format".to_string(), json!(response_format));
    }

    Ok(Value::Object(body))
}

fn prompt_text(opts: &ImageGenOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn edit_prompt_text(opts: &ImageEditOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn image_count(opts: &ImageGenOptions) -> u32 {
    opts.count.unwrap_or(DEFAULT_IMAGE_COUNT)
}

fn image_response_format(opts: &ImageGenOptions) -> Option<String> {
    opts.response_format
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            (opts.output_file.is_some() || opts.output_dir.is_some())
                .then_some(RESPONSE_FORMAT_B64_JSON.to_string())
        })
}

fn image_edit_response_format(opts: &ImageEditOptions) -> Option<String> {
    opts.response_format
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            opts.output_file
                .is_some()
                .then_some(RESPONSE_FORMAT_B64_JSON.to_string())
        })
}

fn build_edit_image_inputs(values: &[String]) -> Result<Vec<Value>, AppError> {
    values
        .iter()
        .map(|value| build_edit_image_input(value))
        .collect()
}

fn build_edit_image_input(value: &str) -> Result<Value, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--image value must not be empty",
        ));
    }
    if is_remote_or_data_image(value) {
        return Ok(json!({
            "type": "image_url",
            "url": value
        }));
    }

    let path = Path::new(value);
    let bytes = fs::read(path).map_err(|error| {
        AppError::io(format!(
            "failed to read image edit input {}: {error}",
            path.display()
        ))
    })?;
    let mime = image_mime_type(path);
    Ok(json!({
        "type": "image_url",
        "url": format!("data:{mime};base64,{}", BASE64_STANDARD.encode(bytes))
    }))
}

fn is_remote_or_data_image(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://") || value.starts_with("data:")
}

fn image_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/png",
    }
}

fn parse_image_response(
    opts: &ImageGenOptions,
    model: &str,
    upstream: &upstream::UpstreamJsonEnvelope,
) -> Result<ImageGenData, AppError> {
    let images = if let Some(output_file) = opts.output_file.as_deref() {
        let image_b64 = extract_image_b64(&upstream.response)?;
        save_base64_image(output_file, &image_b64)?;
        vec![output_file.display().to_string()]
    } else if let Some(output_dir) = opts.output_dir.as_deref() {
        save_base64_images(output_dir, &extract_image_b64_values(&upstream.response)?)?
    } else {
        extract_image_references(&upstream.response)?
    };
    let image = images.first().cloned().ok_or_else(|| {
        AppError::new(
            ErrorCode::RequestFailed,
            "images API payload did not include an image URL or data",
        )
    })?;

    Ok(ImageGenData {
        provider: "xai".to_string(),
        credential_source: upstream.credential_source.clone(),
        model: model.to_string(),
        image,
        images,
        aspect_ratio: opts.aspect_ratio.clone(),
        extra: json!({
            "resolution": opts.resolution.clone(),
            "count": image_count(opts),
            "response_format": image_response_format(opts)
        }),
    })
}

fn parse_image_edit_response(
    opts: &ImageEditOptions,
    model: &str,
    upstream: &upstream::UpstreamJsonEnvelope,
) -> Result<ImageGenData, AppError> {
    let images = if let Some(output_file) = opts.output_file.as_deref() {
        let image_b64 = extract_image_b64(&upstream.response)?;
        save_base64_image(output_file, &image_b64)?;
        vec![output_file.display().to_string()]
    } else {
        extract_image_references(&upstream.response)?
    };
    let image = images.first().cloned().ok_or_else(|| {
        AppError::new(
            ErrorCode::RequestFailed,
            "images API payload did not include an image URL or data",
        )
    })?;

    Ok(ImageGenData {
        provider: "xai".to_string(),
        credential_source: upstream.credential_source.clone(),
        model: model.to_string(),
        image,
        images,
        aspect_ratio: opts.aspect_ratio.clone(),
        extra: json!({
            "resolution": opts.resolution.clone(),
            "input_count": opts.images.len(),
            "response_format": image_edit_response_format(opts)
        }),
    })
}

#[cfg(test)]
fn extract_image_reference(response: &Value) -> Result<String, AppError> {
    extract_image_references(response)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                "images API payload did not include an image URL or data",
            )
        })
}

fn extract_image_references(response: &Value) -> Result<Vec<String>, AppError> {
    if let Some(url) = response.get("url").and_then(|value| value.as_str()) {
        return Ok(vec![url.to_string()]);
    }

    if let Some(b64) = response.get("b64_json").and_then(|value| value.as_str()) {
        return Ok(vec![format!("data:image/png;base64,{b64}")]);
    }

    let images: Vec<String> = response
        .get("data")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| {
            value
                .get("url")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
                .or_else(|| {
                    value
                        .get("b64_json")
                        .and_then(|value| value.as_str())
                        .map(|b64| format!("data:image/png;base64,{b64}"))
                })
        })
        .collect();
    if !images.is_empty() {
        return Ok(images);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        "images API payload did not include an image URL or data",
    ))
}

fn extract_image_b64(response: &Value) -> Result<String, AppError> {
    extract_image_b64_values(response)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                "images API payload did not include `b64_json` for file output",
            )
        })
}

fn extract_image_b64_values(response: &Value) -> Result<Vec<String>, AppError> {
    if let Some(b64) = response.get("b64_json").and_then(|value| value.as_str()) {
        return Ok(vec![b64.to_string()]);
    }

    let images: Vec<String> = response
        .get("data")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.get("b64_json")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
        })
        .collect();
    if !images.is_empty() {
        return Ok(images);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        "images API payload did not include `b64_json` for file output",
    ))
}

fn save_base64_images(dir: &Path, images_b64: &[String]) -> Result<Vec<String>, AppError> {
    fs::create_dir_all(dir).map_err(|error| {
        AppError::io(format!(
            "failed to create image output directory {}: {error}",
            dir.display()
        ))
    })?;

    images_b64
        .iter()
        .enumerate()
        .map(|(index, image_b64)| {
            let path = numbered_image_path(dir, index);
            save_base64_image(&path, image_b64)?;
            Ok(path.display().to_string())
        })
        .collect()
}

fn numbered_image_path(dir: &Path, index: usize) -> PathBuf {
    dir.join(format!("image-{:03}.png", index + 1))
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
        build_edit_image_input, build_edit_request, build_request, extract_image_b64,
        extract_image_b64_values, extract_image_reference, extract_image_references,
        parse_image_edit_response, parse_image_response, validate_edit_options, validate_options,
    };
    use crate::args::{ImageEditOptions, ImageGenOptions, TaskCommonOptions};
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
            count: None,
            response_format: None,
            output_file: None,
            output_dir: None,
            timeout: Some(60),
        }
    }

    fn sample_edit_opts() -> ImageEditOptions {
        ImageEditOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("Make it cinematic".to_string()),
            prompt_flag: None,
            images: vec!["https://cdn.x.ai/source.png".to_string()],
            model: Some("grok-imagine-image".to_string()),
            aspect_ratio: Some("16:9".to_string()),
            resolution: Some("1k".to_string()),
            response_format: None,
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
    fn validate_options_rejects_count_out_of_range() {
        let mut opts = sample_opts();
        opts.count = Some(0);
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--count must be between"));

        opts.count = Some(11);
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--count must be between"));
    }

    #[test]
    fn validate_options_rejects_output_file_with_multiple_images() {
        let mut opts = sample_opts();
        opts.count = Some(2);
        opts.output_file = Some(std::path::PathBuf::from("/tmp/image.png"));
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--output-file can only be used"));
    }

    #[test]
    fn validate_options_rejects_url_response_for_file_output() {
        let mut opts = sample_opts();
        opts.response_format = Some("url".to_string());
        opts.output_dir = Some(std::path::PathBuf::from("/tmp/images"));
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("require --response-format b64_json"));
    }

    #[test]
    fn validate_edit_options_rejects_missing_image() {
        let mut opts = sample_edit_opts();
        opts.images = vec![];
        let error = validate_edit_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--image is required"));
    }

    #[test]
    fn validate_edit_options_rejects_too_many_images() {
        let mut opts = sample_edit_opts();
        opts.images = vec![
            "https://cdn.x.ai/1.png".to_string(),
            "https://cdn.x.ai/2.png".to_string(),
            "https://cdn.x.ai/3.png".to_string(),
            "https://cdn.x.ai/4.png".to_string(),
        ];
        let error = validate_edit_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--image supports at most 3 values"));
    }

    #[test]
    fn validate_edit_options_rejects_url_response_for_file_output() {
        let mut opts = sample_edit_opts();
        opts.response_format = Some("url".to_string());
        opts.output_file = Some(std::path::PathBuf::from("/tmp/edit.png"));
        let error = validate_edit_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--output-file requires"));
    }

    #[test]
    fn build_request_uses_remote_url_by_default() {
        let opts = sample_opts();
        let request = build_request(&opts, "grok-imagine-image");
        assert_eq!(request["model"], "grok-imagine-image");
        assert_eq!(request["prompt"], "Draw a skyline");
        assert_eq!(request["aspect_ratio"], "16:9");
        assert_eq!(request["resolution"], "1k");
        assert_eq!(request["n"], 1);
        assert!(request["response_format"].is_null());
    }

    #[test]
    fn build_request_includes_count_and_response_format() {
        let mut opts = sample_opts();
        opts.count = Some(3);
        opts.response_format = Some("b64_json".to_string());
        let request = build_request(&opts, "grok-imagine-image");
        assert_eq!(request["n"], 3);
        assert_eq!(request["response_format"], "b64_json");
    }

    #[test]
    fn build_request_switches_to_b64_when_output_file_is_requested() {
        let mut opts = sample_opts();
        opts.output_file = Some(std::path::PathBuf::from("/tmp/image.png"));
        let request = build_request(&opts, "grok-imagine-image");
        assert_eq!(request["response_format"], "b64_json");
    }

    #[test]
    fn build_request_switches_to_b64_when_output_dir_is_requested() {
        let mut opts = sample_opts();
        opts.count = Some(2);
        opts.output_dir = Some(std::path::PathBuf::from("/tmp/images"));
        let request = build_request(&opts, "grok-imagine-image");
        assert_eq!(request["n"], 2);
        assert_eq!(request["response_format"], "b64_json");
    }

    #[test]
    fn build_edit_request_uses_single_image_field_for_one_input() {
        let opts = sample_edit_opts();
        let request = build_edit_request(&opts, "grok-imagine-image").unwrap();
        assert_eq!(request["model"], "grok-imagine-image");
        assert_eq!(request["prompt"], "Make it cinematic");
        assert_eq!(request["image"]["type"], "image_url");
        assert_eq!(request["image"]["url"], "https://cdn.x.ai/source.png");
        assert!(request["images"].is_null());
        assert_eq!(request["aspect_ratio"], "16:9");
        assert_eq!(request["resolution"], "1k");
    }

    #[test]
    fn build_edit_request_uses_images_field_for_multiple_inputs() {
        let mut opts = sample_edit_opts();
        opts.images = vec![
            "https://cdn.x.ai/1.png".to_string(),
            "https://cdn.x.ai/2.png".to_string(),
            "https://cdn.x.ai/3.png".to_string(),
        ];
        opts.response_format = Some("b64_json".to_string());

        let request = build_edit_request(&opts, "grok-imagine-image").unwrap();
        assert!(request["image"].is_null());
        assert_eq!(request["images"].as_array().unwrap().len(), 3);
        assert_eq!(request["images"][2]["url"], "https://cdn.x.ai/3.png");
        assert_eq!(request["response_format"], "b64_json");
    }

    #[test]
    fn build_edit_image_input_encodes_local_file_as_data_uri() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("source.png");
        std::fs::write(&path, b"hello").unwrap();

        let input = build_edit_image_input(path.to_str().unwrap()).unwrap();
        assert_eq!(input["type"], "image_url");
        assert_eq!(input["url"], "data:image/png;base64,aGVsbG8=");
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
        assert_eq!(parsed.images, vec!["https://cdn.x.ai/image.png"]);
        assert_eq!(parsed.credential_source, "xai-oauth");
    }

    #[test]
    fn parse_image_response_returns_multiple_urls() {
        let mut opts = sample_opts();
        opts.count = Some(2);
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "data": [
                    {"url": "https://cdn.x.ai/image-1.png"},
                    {"url": "https://cdn.x.ai/image-2.png"}
                ]
            }),
            usage: ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed = parse_image_response(&opts, "grok-imagine-image", &upstream).unwrap();
        assert_eq!(parsed.image, "https://cdn.x.ai/image-1.png");
        assert_eq!(
            parsed.images,
            vec![
                "https://cdn.x.ai/image-1.png".to_string(),
                "https://cdn.x.ai/image-2.png".to_string()
            ]
        );
    }

    #[test]
    fn parse_image_response_returns_multiple_b64_data_urls() {
        let mut opts = sample_opts();
        opts.count = Some(2);
        opts.response_format = Some("b64_json".to_string());
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "data": [
                    {"b64_json": "aGVsbG8="},
                    {"b64_json": "d29ybGQ="}
                ]
            }),
            usage: ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed = parse_image_response(&opts, "grok-imagine-image", &upstream).unwrap();
        assert_eq!(parsed.image, "data:image/png;base64,aGVsbG8=");
        assert_eq!(
            parsed.images,
            vec![
                "data:image/png;base64,aGVsbG8=".to_string(),
                "data:image/png;base64,d29ybGQ=".to_string()
            ]
        );
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
        assert_eq!(parsed.images, vec![image_path.display().to_string()]);
        assert_eq!(std::fs::read(&image_path).unwrap(), b"hello");
    }

    #[test]
    fn parse_image_response_writes_multiple_output_files_when_output_dir_requested() {
        let temp = tempdir().unwrap();
        let image_dir = temp.path().join("images");
        let mut opts = sample_opts();
        opts.count = Some(2);
        opts.output_dir = Some(image_dir.clone());
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "data": [
                    {"b64_json": "aGVsbG8="},
                    {"b64_json": "d29ybGQ="}
                ]
            }),
            usage: ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed = parse_image_response(&opts, "grok-imagine-image", &upstream).unwrap();
        let first = image_dir.join("image-001.png");
        let second = image_dir.join("image-002.png");
        assert_eq!(parsed.image, first.display().to_string());
        assert_eq!(
            parsed.images,
            vec![first.display().to_string(), second.display().to_string()]
        );
        assert_eq!(std::fs::read(first).unwrap(), b"hello");
        assert_eq!(std::fs::read(second).unwrap(), b"world");
    }

    #[test]
    fn parse_image_edit_response_writes_output_file_when_requested() {
        let temp = tempdir().unwrap();
        let image_path = temp.path().join("edited.png");
        let mut opts = sample_edit_opts();
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

        let parsed = parse_image_edit_response(&opts, "grok-imagine-image", &upstream).unwrap();
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

    #[test]
    fn extract_image_references_collects_all_urls() {
        let images = extract_image_references(&json!({
            "data": [
                {"url": "https://cdn.x.ai/1.png"},
                {"url": "https://cdn.x.ai/2.png"}
            ]
        }))
        .unwrap();
        assert_eq!(
            images,
            vec![
                "https://cdn.x.ai/1.png".to_string(),
                "https://cdn.x.ai/2.png".to_string()
            ]
        );
    }

    #[test]
    fn extract_image_b64_values_collects_all_values() {
        let images = extract_image_b64_values(&json!({
            "data": [
                {"b64_json": "aGVsbG8="},
                {"b64_json": "d29ybGQ="}
            ]
        }))
        .unwrap();
        assert_eq!(images, vec!["aGVsbG8=".to_string(), "d29ybGQ=".to_string()]);
    }
}
