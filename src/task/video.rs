use std::fs;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::Serialize;
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::args::{TaskCommonOptions, VideoEditOptions, VideoExtendOptions, VideoGenOptions};
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::tracker;

const DEFAULT_VIDEO_MODEL: &str = "grok-imagine-video";
const VIDEO_GENERATIONS_PATH: &str = "/videos/generations";
const VIDEO_EDITS_PATH: &str = "/videos/edits";
const VIDEO_EXTENSIONS_PATH: &str = "/videos/extensions";
const DEFAULT_POLL_INTERVAL_MILLIS: u64 = 5_000;
const DEFAULT_VIDEO_POLL_TIMEOUT_SECONDS: u64 = 600;
const DEFAULT_VIDEO_DURATION: u64 = 8;
const DEFAULT_VIDEO_EXTENSION_DURATION: u64 = 6;
const DEFAULT_VIDEO_ASPECT_RATIO: &str = "16:9";
const DEFAULT_VIDEO_RESOLUTION: &str = "720p";

#[derive(Debug, Clone, Serialize)]
struct VideoGenData {
    provider: String,
    credential_source: String,
    model: String,
    video: String,
    modality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<u64>,
    extra: Value,
}

type VideoRequestBuilder = dyn Fn(&str) -> Result<Value, AppError>;

struct VideoTaskRequest {
    command: &'static str,
    common: TaskCommonOptions,
    model: Option<String>,
    create_path: &'static str,
    build_request: Box<VideoRequestBuilder>,
    timeout: Option<u64>,
    modality: String,
    aspect_ratio: Option<String>,
    duration: Option<u64>,
    resolution: Option<String>,
}

pub fn execute(ctx: &AppContext, opts: VideoGenOptions) -> CommandResult {
    let command = "video";
    validate_options(&opts).map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let task = video_generation_task(opts, command);
    execute_video_task(ctx, task)
}

pub fn execute_edit(ctx: &AppContext, opts: VideoEditOptions) -> CommandResult {
    let command = "video-edit";
    validate_edit_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let task = video_edit_task(opts, command);
    execute_video_task(ctx, task)
}

pub fn execute_extend(ctx: &AppContext, opts: VideoExtendOptions) -> CommandResult {
    let command = "video-extend";
    validate_extend_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let task = video_extend_task(opts, command);
    execute_video_task(ctx, task)
}

fn execute_video_task(ctx: &AppContext, task: VideoTaskRequest) -> CommandResult {
    let command = task.command;
    let json_output = task.common.json;
    let auth_file = task.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = task.model.clone().unwrap_or_else(|| {
        model::default_model_for_task(state.as_ref(), "video", DEFAULT_VIDEO_MODEL)
    });
    let request = (task.build_request)(&model)
        .map_err(|error| CommandError::new(command, json_output, error))?;
    let created = upstream::post_json_api_with_options(
        ctx,
        task.common.auth_file.as_deref(),
        task.create_path,
        &request,
        Some(upstream::DEFAULT_MEDIA_TIMEOUT_SECONDS),
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, json_output, error))?;

    let request_id = extract_request_id(&created.response)
        .map_err(|error| CommandError::new(command, json_output, error))?;
    let polled = poll_video_result(ctx, &task.common, task.timeout, &request_id)
        .map_err(|error| CommandError::new(command, json_output, error))?;

    let data = parse_video_task_response(&task, &model, &request_id, &polled)
        .map_err(|error| CommandError::new(command, json_output, error))?;
    tracker::record_usage(
        ctx,
        task.common.auth_file.as_deref(),
        &polled.credential_source,
        UsageDelta {
            provider: polled.credential_source.clone(),
            command: command.to_string(),
            model: Some(model.clone()),
            rate_limits: polled.rate_limits.clone(),
            ..UsageDelta::default()
        },
    )
    .map_err(|error| CommandError::new(command, json_output, error))?;

    if json_output {
        output::print_json_success(command, &data);
    } else {
        println!("provider: {}", data.provider);
        println!("credential_source: {}", data.credential_source);
        println!("model: {}", data.model);
        println!("video: {}", data.video);
        println!("modality: {}", data.modality);
        if let Some(aspect_ratio) = &data.aspect_ratio {
            println!("aspect_ratio: {aspect_ratio}");
        }
        if let Some(duration) = data.duration {
            println!("duration: {duration}");
        }
        println!("extra: {}", data.extra);
    }

    Ok(())
}

fn validate_options(opts: &VideoGenOptions) -> Result<(), AppError> {
    if prompt_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "prompt must not be empty",
        ));
    }

    let reference_count = opts.reference_image_urls.len() + opts.reference_images.len();
    if reference_count > 7 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--reference-image-url and --reference-image support at most 7 values total",
        ));
    }

    let image_mode_count = usize::from(opts.image_url.is_some())
        + usize::from(opts.image.is_some())
        + usize::from(!opts.reference_image_urls.is_empty() || !opts.reference_images.is_empty());
    if image_mode_count > 1 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--image-url, --image, --reference-image-url, and --reference-image are mutually exclusive input modes for xAI video generation",
        ));
    }

    if let Some(path) = opts.image.as_deref() {
        validate_local_file_exists(path)?;
    }
    for path in &opts.reference_images {
        validate_local_file_exists(path)?;
    }

    Ok(())
}

fn validate_edit_options(opts: &VideoEditOptions) -> Result<(), AppError> {
    if edit_prompt_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "prompt must not be empty",
        ));
    }

    if non_empty_string(opts.video_url.as_deref()).is_none() && opts.video.is_none() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--video-url or --video is required",
        ));
    }
    if opts.video_url.is_some() && opts.video.is_some() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--video-url cannot be combined with --video",
        ));
    }
    if let Some(path) = opts.video.as_deref() {
        validate_local_file_exists(path)?;
    }

    Ok(())
}

fn validate_extend_options(opts: &VideoExtendOptions) -> Result<(), AppError> {
    if extend_prompt_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "prompt must not be empty",
        ));
    }

    if non_empty_string(opts.video_url.as_deref()).is_none() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--video-url is required",
        ));
    }

    Ok(())
}

fn video_generation_task(opts: VideoGenOptions, command: &'static str) -> VideoTaskRequest {
    let common = opts.common.clone();
    let model = opts.model.clone();
    let timeout = opts.timeout;
    let modality = video_generation_modality(&opts);
    let aspect_ratio = Some(normalize_aspect_ratio(opts.aspect_ratio.as_deref()).to_string());
    let duration = Some(clamp_duration(
        opts.duration,
        !opts.reference_image_urls.is_empty(),
    ));
    let resolution = Some(normalize_resolution(opts.resolution.as_deref()).to_string());

    VideoTaskRequest {
        command,
        common,
        model,
        create_path: VIDEO_GENERATIONS_PATH,
        build_request: Box::new(move |model| build_request(&opts, model)),
        timeout,
        modality,
        aspect_ratio,
        duration,
        resolution,
    }
}

fn video_edit_task(opts: VideoEditOptions, command: &'static str) -> VideoTaskRequest {
    let common = opts.common.clone();
    let model = opts.model.clone();
    let timeout = opts.timeout;

    VideoTaskRequest {
        command,
        common,
        model,
        create_path: VIDEO_EDITS_PATH,
        build_request: Box::new(move |model| build_edit_request(&opts, model)),
        timeout,
        modality: "edit".to_string(),
        aspect_ratio: None,
        duration: None,
        resolution: None,
    }
}

fn video_extend_task(opts: VideoExtendOptions, command: &'static str) -> VideoTaskRequest {
    let common = opts.common.clone();
    let model = opts.model.clone();
    let timeout = opts.timeout;
    let duration = Some(clamp_extension_duration(opts.duration));

    VideoTaskRequest {
        command,
        common,
        model,
        create_path: VIDEO_EXTENSIONS_PATH,
        build_request: Box::new(move |model| build_extend_request(&opts, model)),
        timeout,
        modality: "extension".to_string(),
        aspect_ratio: None,
        duration,
        resolution: None,
    }
}

fn build_request(opts: &VideoGenOptions, model: &str) -> Result<Value, AppError> {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(prompt_text(opts)));
    body.insert(
        "duration".to_string(),
        json!(clamp_duration(
            opts.duration,
            !opts.reference_image_urls.is_empty() || !opts.reference_images.is_empty()
        )),
    );
    body.insert(
        "aspect_ratio".to_string(),
        json!(normalize_aspect_ratio(opts.aspect_ratio.as_deref())),
    );
    body.insert(
        "resolution".to_string(),
        json!(normalize_resolution(opts.resolution.as_deref())),
    );

    if let Some(image_url) = opts.image_url.as_deref() {
        body.insert("image".to_string(), json!({ "url": image_url }));
    }
    if let Some(image_path) = opts.image.as_deref() {
        body.insert(
            "image".to_string(),
            json!({ "url": local_file_data_uri(image_path, MediaKind::Image)? }),
        );
    }
    if !opts.reference_image_urls.is_empty() || !opts.reference_images.is_empty() {
        let mut refs: Vec<Value> = opts
            .reference_image_urls
            .iter()
            .map(|url| json!({ "url": url }))
            .collect();
        for path in &opts.reference_images {
            refs.push(json!({
                "url": local_file_data_uri(path, MediaKind::Image)?
            }));
        }
        body.insert("reference_images".to_string(), Value::Array(refs));
    }

    Ok(Value::Object(body))
}

fn build_edit_request(opts: &VideoEditOptions, model: &str) -> Result<Value, AppError> {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(edit_prompt_text(opts)));
    if let Some(video_url) = non_empty_string(opts.video_url.as_deref()) {
        body.insert("video".to_string(), json!({ "url": video_url }));
    }
    if let Some(video_path) = opts.video.as_deref() {
        body.insert(
            "video".to_string(),
            json!({ "url": local_file_data_uri(video_path, MediaKind::Video)? }),
        );
    }
    Ok(Value::Object(body))
}

fn build_extend_request(opts: &VideoExtendOptions, model: &str) -> Result<Value, AppError> {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(extend_prompt_text(opts)));
    body.insert(
        "duration".to_string(),
        json!(clamp_extension_duration(opts.duration)),
    );
    if let Some(video_url) = non_empty_string(opts.video_url.as_deref()) {
        body.insert("video".to_string(), json!({ "url": video_url }));
    }
    Ok(Value::Object(body))
}

fn prompt_text(opts: &VideoGenOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn edit_prompt_text(opts: &VideoEditOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn extend_prompt_text(opts: &VideoExtendOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
}

fn non_empty_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Clone, Copy)]
enum MediaKind {
    Image,
    Video,
}

fn validate_local_file_exists(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::InvalidArgs,
            format!("file does not exist: {}", path.display()),
        ))
    }
}

fn local_file_data_uri(path: &Path, media_kind: MediaKind) -> Result<String, AppError> {
    let bytes = fs::read(path).map_err(|error| {
        AppError::new(
            ErrorCode::InvalidArgs,
            format!("failed to read file {}: {error}", path.display()),
        )
    })?;
    let mime = match media_kind {
        MediaKind::Image => image_mime_type(path),
        MediaKind::Video => video_mime_type(path),
    };
    Ok(format!(
        "data:{mime};base64,{}",
        BASE64_STANDARD.encode(bytes)
    ))
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

fn video_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        _ => "video/mp4",
    }
}

fn extract_request_id(response: &Value) -> Result<String, AppError> {
    response
        .get("id")
        .or_else(|| response.get("request_id"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                "video generations payload did not include a request id",
            )
        })
}

fn poll_video_result(
    ctx: &AppContext,
    common: &TaskCommonOptions,
    timeout: Option<u64>,
    request_id: &str,
) -> Result<upstream::UpstreamJsonEnvelope, AppError> {
    let endpoint = format!("/videos/{request_id}");
    let poll_timeout = std::time::Duration::from_secs(video_poll_timeout_seconds(timeout));
    let started_at = std::time::Instant::now();

    while started_at.elapsed() < poll_timeout {
        let remaining = poll_timeout.saturating_sub(started_at.elapsed());
        let request_timeout_seconds = video_poll_request_timeout_seconds(remaining);
        let response = upstream::get_json_api_with_options(
            ctx,
            common.auth_file.as_deref(),
            &endpoint,
            Some(request_timeout_seconds),
            upstream::UpstreamAuthOptions {
                refresh_if_expiring: true,
            },
        )?;

        if is_terminal_video_status(&response.response) {
            return Ok(response);
        }

        let remaining_after_request = poll_timeout.saturating_sub(started_at.elapsed());
        let sleep_for = remaining_after_request.min(std::time::Duration::from_millis(
            DEFAULT_POLL_INTERVAL_MILLIS,
        ));
        if sleep_for.is_zero() {
            break;
        }
        std::thread::sleep(sleep_for);
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        format!(
            "timed out waiting for video generation result for {request_id} after {} seconds",
            poll_timeout.as_secs()
        ),
    ))
}

fn video_poll_timeout_seconds(timeout: Option<u64>) -> u64 {
    timeout.unwrap_or(DEFAULT_VIDEO_POLL_TIMEOUT_SECONDS)
}

fn video_poll_request_timeout_seconds(remaining: std::time::Duration) -> u64 {
    remaining
        .as_secs()
        .clamp(1, upstream::DEFAULT_MEDIA_TIMEOUT_SECONDS)
}

fn is_terminal_video_status(response: &Value) -> bool {
    let status = response
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        status.as_str(),
        "done" | "completed" | "succeeded" | "failed" | "error" | "cancelled" | "expired"
    )
}

fn parse_video_task_response(
    task: &VideoTaskRequest,
    model: &str,
    request_id: &str,
    upstream: &upstream::UpstreamJsonEnvelope,
) -> Result<VideoGenData, AppError> {
    let status = upstream
        .response
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_ascii_lowercase();

    if matches!(
        status.as_str(),
        "failed" | "error" | "cancelled" | "expired"
    ) {
        let message = upstream
            .response
            .get("error")
            .and_then(|value| value.as_str())
            .or_else(|| {
                upstream
                    .response
                    .get("error")
                    .and_then(|value| value.get("message"))
                    .and_then(|value| value.as_str())
            })
            .or_else(|| {
                upstream
                    .response
                    .get("message")
                    .and_then(|value| value.as_str())
            })
            .unwrap_or("video generation did not complete successfully");
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("{} failed for {request_id}: {message}", task.command),
        ));
    }

    let video_url = extract_video_url(&upstream.response)?;

    Ok(VideoGenData {
        provider: "xai".to_string(),
        credential_source: upstream.credential_source.clone(),
        model: model.to_string(),
        video: video_url,
        modality: task.modality.clone(),
        aspect_ratio: task.aspect_ratio.clone(),
        duration: upstream
            .response
            .get("video")
            .and_then(|value| value.get("duration"))
            .and_then(|value| value.as_u64())
            .or(task.duration),
        extra: json!({
            "request_id": request_id,
            "resolution": task.resolution
        }),
    })
}

#[cfg(test)]
fn parse_video_response(
    opts: &VideoGenOptions,
    model: &str,
    request_id: &str,
    upstream: &upstream::UpstreamJsonEnvelope,
) -> Result<VideoGenData, AppError> {
    let task = video_generation_task(opts.clone(), "video");
    parse_video_task_response(&task, model, request_id, upstream)
}

fn video_generation_modality(opts: &VideoGenOptions) -> String {
    if opts.image_url.is_some()
        || opts.image.is_some()
        || !opts.reference_image_urls.is_empty()
        || !opts.reference_images.is_empty()
    {
        "image".to_string()
    } else {
        "text".to_string()
    }
}

fn clamp_duration(duration: Option<u64>, has_reference_images: bool) -> u64 {
    let mut value = duration.unwrap_or(DEFAULT_VIDEO_DURATION);
    value = value.clamp(1, 15);
    if has_reference_images && value > 10 {
        value = 10;
    }
    value
}

fn clamp_extension_duration(duration: Option<u64>) -> u64 {
    duration
        .unwrap_or(DEFAULT_VIDEO_EXTENSION_DURATION)
        .clamp(2, 10)
}

fn normalize_aspect_ratio(value: Option<&str>) -> &'static str {
    match value.unwrap_or(DEFAULT_VIDEO_ASPECT_RATIO).trim() {
        "1:1" => "1:1",
        "16:9" => "16:9",
        "9:16" => "9:16",
        "4:3" => "4:3",
        "3:4" => "3:4",
        "3:2" => "3:2",
        "2:3" => "2:3",
        _ => DEFAULT_VIDEO_ASPECT_RATIO,
    }
}

fn normalize_resolution(value: Option<&str>) -> &'static str {
    match value
        .unwrap_or(DEFAULT_VIDEO_RESOLUTION)
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "480p" => "480p",
        "720p" => "720p",
        _ => DEFAULT_VIDEO_RESOLUTION,
    }
}

fn extract_video_url(response: &Value) -> Result<String, AppError> {
    if let Some(url) = response
        .get("video")
        .and_then(|value| value.get("url"))
        .and_then(|value| value.as_str())
    {
        return Ok(url.to_string());
    }

    if let Some(url) = response.get("url").and_then(|value| value.as_str()) {
        return Ok(url.to_string());
    }

    Err(AppError::new(
        ErrorCode::RequestFailed,
        "video generation payload did not include a video URL",
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        build_edit_request, build_extend_request, build_request, clamp_extension_duration,
        extract_request_id, extract_video_url, parse_video_response, validate_edit_options,
        validate_extend_options, validate_options, video_poll_request_timeout_seconds,
        video_poll_timeout_seconds,
    };
    use crate::args::{TaskCommonOptions, VideoEditOptions, VideoExtendOptions, VideoGenOptions};
    use crate::upstream::UpstreamJsonEnvelope;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn sample_opts() -> VideoGenOptions {
        VideoGenOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("Animate a futuristic skyline".to_string()),
            prompt_flag: None,
            image_url: None,
            image: None,
            reference_image_urls: vec![],
            reference_images: vec![],
            duration: Some(8),
            aspect_ratio: Some("16:9".to_string()),
            resolution: Some("720p".to_string()),
            model: Some("grok-imagine-video".to_string()),
            timeout: Some(60),
        }
    }

    fn sample_edit_opts() -> VideoEditOptions {
        VideoEditOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("Give the woman a silver necklace".to_string()),
            prompt_flag: None,
            video_url: Some("https://cdn.x.ai/source.mp4".to_string()),
            video: None,
            model: Some("grok-imagine-video".to_string()),
            timeout: Some(60),
        }
    }

    fn sample_extend_opts() -> VideoExtendOptions {
        VideoExtendOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("The camera pans left".to_string()),
            prompt_flag: None,
            video_url: Some("https://cdn.x.ai/source.mp4".to_string()),
            duration: Some(6),
            model: Some("grok-imagine-video".to_string()),
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
    fn validate_options_rejects_too_many_reference_images() {
        let mut opts = sample_opts();
        opts.reference_image_urls = (0..8).map(|i| format!("https://x.ai/{i}.png")).collect();
        let error = validate_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--reference-image-url"));
    }

    #[test]
    fn validate_edit_options_rejects_empty_prompt() {
        let mut opts = sample_edit_opts();
        opts.prompt = Some("   ".to_string());
        let error = validate_edit_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("prompt must not be empty"));
    }

    #[test]
    fn validate_edit_options_rejects_missing_video_url() {
        let mut opts = sample_edit_opts();
        opts.video_url = None;
        let error = validate_edit_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--video-url or --video is required"));
    }

    #[test]
    fn validate_extend_options_rejects_empty_prompt() {
        let mut opts = sample_extend_opts();
        opts.prompt = Some("   ".to_string());
        let error = validate_extend_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("prompt must not be empty"));
    }

    #[test]
    fn validate_extend_options_rejects_missing_video_url() {
        let mut opts = sample_extend_opts();
        opts.video_url = None;
        let error = validate_extend_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--video-url is required"));
    }

    #[test]
    fn build_request_includes_video_fields() {
        let mut opts = sample_opts();
        opts.image_url = Some("https://cdn.x.ai/source.png".to_string());
        let request = build_request(&opts, "grok-imagine-video").unwrap();
        assert_eq!(request["image"]["url"], "https://cdn.x.ai/source.png");
        assert_eq!(request["duration"], 8);
        assert_eq!(request["aspect_ratio"], "16:9");
        assert_eq!(request["resolution"], "720p");
    }

    #[test]
    fn build_request_wraps_reference_images_as_url_objects() {
        let mut opts = sample_opts();
        opts.reference_image_urls = vec![
            "https://cdn.x.ai/ref-1.png".to_string(),
            "https://cdn.x.ai/ref-2.png".to_string(),
        ];
        let request = build_request(&opts, "grok-imagine-video").unwrap();
        assert_eq!(
            request["reference_images"][0]["url"],
            "https://cdn.x.ai/ref-1.png"
        );
        assert_eq!(request["duration"], 8);
    }

    #[test]
    fn build_request_wraps_local_image_as_data_uri() {
        let temp = tempdir().unwrap();
        let image = temp.path().join("source.png");
        fs::write(&image, b"hello").unwrap();

        let mut opts = sample_opts();
        opts.image = Some(image);
        let request = build_request(&opts, "grok-imagine-video").unwrap();
        assert_eq!(request["image"]["url"], "data:image/png;base64,aGVsbG8=");
    }

    #[test]
    fn build_edit_request_wraps_video_url_without_generation_fields() {
        let opts = sample_edit_opts();
        let request = build_edit_request(&opts, "grok-imagine-video").unwrap();
        assert_eq!(request["model"], "grok-imagine-video");
        assert_eq!(request["prompt"], "Give the woman a silver necklace");
        assert_eq!(request["video"]["url"], "https://cdn.x.ai/source.mp4");
        assert!(request["video_url"].is_null());
        assert!(request["duration"].is_null());
        assert!(request["aspect_ratio"].is_null());
        assert!(request["resolution"].is_null());
    }

    #[test]
    fn build_edit_request_wraps_local_video_as_data_uri() {
        let temp = tempdir().unwrap();
        let video = temp.path().join("source.mp4");
        fs::write(&video, b"fake-mp4").unwrap();

        let mut opts = sample_edit_opts();
        opts.video_url = None;
        opts.video = Some(video);
        let request = build_edit_request(&opts, "grok-imagine-video").unwrap();
        assert_eq!(
            request["video"]["url"],
            "data:video/mp4;base64,ZmFrZS1tcDQ="
        );
    }

    #[test]
    fn build_extend_request_wraps_video_url_and_duration_without_generation_fields() {
        let opts = sample_extend_opts();
        let request = build_extend_request(&opts, "grok-imagine-video").unwrap();
        assert_eq!(request["model"], "grok-imagine-video");
        assert_eq!(request["prompt"], "The camera pans left");
        assert_eq!(request["video"]["url"], "https://cdn.x.ai/source.mp4");
        assert_eq!(request["duration"], 6);
        assert!(request["video_url"].is_null());
        assert!(request["aspect_ratio"].is_null());
        assert!(request["resolution"].is_null());
    }

    #[test]
    fn build_extend_request_defaults_and_clamps_duration() {
        let mut opts = sample_extend_opts();
        opts.duration = None;
        assert_eq!(
            build_extend_request(&opts, "grok-imagine-video").unwrap()["duration"],
            6
        );

        opts.duration = Some(1);
        assert_eq!(
            build_extend_request(&opts, "grok-imagine-video").unwrap()["duration"],
            2
        );

        opts.duration = Some(11);
        assert_eq!(
            build_extend_request(&opts, "grok-imagine-video").unwrap()["duration"],
            10
        );
        assert_eq!(clamp_extension_duration(None), 6);
    }

    #[test]
    fn extract_request_id_accepts_request_id_field() {
        let request_id = extract_request_id(&json!({"request_id":"vid_123"})).unwrap();
        assert_eq!(request_id, "vid_123");
    }

    #[test]
    fn parse_video_response_extracts_url() {
        let opts = sample_opts();
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "status": "completed",
                "video": {
                    "url": "https://cdn.x.ai/generated-video.mp4"
                }
            }),
            usage: crate::upstream::ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let parsed =
            parse_video_response(&opts, "grok-imagine-video", "vid_123", &upstream).unwrap();
        assert_eq!(parsed.video, "https://cdn.x.ai/generated-video.mp4");
        assert_eq!(parsed.modality, "text");
    }

    #[test]
    fn parse_video_response_rejects_failed_status() {
        let opts = sample_opts();
        let upstream = UpstreamJsonEnvelope {
            credential_source: "xai-oauth".to_string(),
            response: json!({
                "status": "failed",
                "error": "quota exceeded"
            }),
            usage: crate::upstream::ResponseUsageSummary::default(),
            rate_limits: None,
        };

        let error =
            parse_video_response(&opts, "grok-imagine-video", "vid_123", &upstream).unwrap_err();
        assert_eq!(error.code.as_str(), "request_failed");
        assert!(error.message.contains("quota exceeded"));
    }

    #[test]
    fn extract_video_url_rejects_missing_payload() {
        let error = extract_video_url(&json!({"status":"completed"})).unwrap_err();
        assert_eq!(error.code.as_str(), "request_failed");
    }

    #[test]
    fn video_poll_timeout_defaults_to_ten_minutes_and_allows_override() {
        assert_eq!(video_poll_timeout_seconds(None), 600);
        assert_eq!(video_poll_timeout_seconds(Some(900)), 900);
    }

    #[test]
    fn video_poll_request_timeout_is_capped_to_media_request_timeout() {
        assert_eq!(
            video_poll_request_timeout_seconds(std::time::Duration::from_secs(900)),
            120
        );
        assert_eq!(
            video_poll_request_timeout_seconds(std::time::Duration::from_secs(30)),
            30
        );
        assert_eq!(
            video_poll_request_timeout_seconds(std::time::Duration::from_millis(250)),
            1
        );
    }
}
