use serde::Serialize;
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::args::VideoGenOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::tracker;

const DEFAULT_VIDEO_MODEL: &str = "grok-imagine-video";
const VIDEO_GENERATIONS_PATH: &str = "/videos/generations";
const DEFAULT_POLL_INTERVAL_MILLIS: u64 = 5_000;
const DEFAULT_VIDEO_POLL_TIMEOUT_SECONDS: u64 = 600;
const DEFAULT_VIDEO_DURATION: u64 = 8;
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

pub fn execute(ctx: &AppContext, opts: VideoGenOptions) -> CommandResult {
    let command = "video";
    validate_options(&opts).map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts.model.clone().unwrap_or_else(|| {
        model::default_model_for_task(state.as_ref(), "video", DEFAULT_VIDEO_MODEL)
    });
    let request = build_request(&opts, &model);
    let created = upstream::post_json_api_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        VIDEO_GENERATIONS_PATH,
        &request,
        Some(upstream::DEFAULT_MEDIA_TIMEOUT_SECONDS),
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let request_id = extract_request_id(&created.response)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let polled = poll_video_result(ctx, &opts, &request_id)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = parse_video_response(&opts, &model, &request_id, &polled)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    tracker::record_usage(
        ctx,
        opts.common.auth_file.as_deref(),
        &polled.credential_source,
        UsageDelta {
            provider: polled.credential_source.clone(),
            command: command.to_string(),
            model: Some(model.clone()),
            rate_limits: polled.rate_limits.clone(),
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

    if opts.reference_image_urls.len() > 7 {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--reference-image-url supports at most 7 values",
        ));
    }

    if opts.image_url.is_some() && !opts.reference_image_urls.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--image-url cannot be combined with --reference-image-url for xAI video generation",
        ));
    }

    Ok(())
}

fn build_request(opts: &VideoGenOptions, model: &str) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("prompt".to_string(), json!(prompt_text(opts)));
    body.insert(
        "duration".to_string(),
        json!(clamp_duration(
            opts.duration,
            !opts.reference_image_urls.is_empty()
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
    if !opts.reference_image_urls.is_empty() {
        let refs: Vec<Value> = opts
            .reference_image_urls
            .iter()
            .map(|url| json!({ "url": url }))
            .collect();
        body.insert("reference_images".to_string(), Value::Array(refs));
    }

    Value::Object(body)
}

fn prompt_text(opts: &VideoGenOptions) -> &str {
    opts.prompt
        .as_deref()
        .or(opts.prompt_flag.as_deref())
        .unwrap_or("")
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
    opts: &VideoGenOptions,
    request_id: &str,
) -> Result<upstream::UpstreamJsonEnvelope, AppError> {
    let endpoint = format!("/videos/{request_id}");
    let poll_timeout = std::time::Duration::from_secs(video_poll_timeout_seconds(opts.timeout));
    let started_at = std::time::Instant::now();

    while started_at.elapsed() < poll_timeout {
        let remaining = poll_timeout.saturating_sub(started_at.elapsed());
        let request_timeout_seconds = video_poll_request_timeout_seconds(remaining);
        let response = upstream::get_json_api_with_options(
            ctx,
            opts.common.auth_file.as_deref(),
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

fn parse_video_response(
    opts: &VideoGenOptions,
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
            format!("video generation failed for {request_id}: {message}"),
        ));
    }

    let video_url = extract_video_url(&upstream.response)?;
    let modality = if opts.image_url.is_some() || !opts.reference_image_urls.is_empty() {
        "image".to_string()
    } else {
        "text".to_string()
    };

    Ok(VideoGenData {
        provider: "xai".to_string(),
        credential_source: upstream.credential_source.clone(),
        model: model.to_string(),
        video: video_url,
        modality,
        aspect_ratio: Some(normalize_aspect_ratio(opts.aspect_ratio.as_deref()).to_string()),
        duration: Some(
            upstream
                .response
                .get("video")
                .and_then(|value| value.get("duration"))
                .and_then(|value| value.as_u64())
                .unwrap_or_else(|| {
                    clamp_duration(opts.duration, !opts.reference_image_urls.is_empty())
                }),
        ),
        extra: json!({
            "request_id": request_id,
            "resolution": normalize_resolution(opts.resolution.as_deref())
        }),
    })
}

fn clamp_duration(duration: Option<u64>, has_reference_images: bool) -> u64 {
    let mut value = duration.unwrap_or(DEFAULT_VIDEO_DURATION);
    if value < 1 {
        value = 1;
    }
    if value > 15 {
        value = 15;
    }
    if has_reference_images && value > 10 {
        value = 10;
    }
    value
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
        build_request, extract_request_id, extract_video_url, parse_video_response,
        validate_options, video_poll_request_timeout_seconds, video_poll_timeout_seconds,
    };
    use crate::args::{TaskCommonOptions, VideoGenOptions};
    use crate::upstream::UpstreamJsonEnvelope;
    use serde_json::json;

    fn sample_opts() -> VideoGenOptions {
        VideoGenOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            prompt: Some("Animate a futuristic skyline".to_string()),
            prompt_flag: None,
            image_url: None,
            reference_image_urls: vec![],
            duration: Some(8),
            aspect_ratio: Some("16:9".to_string()),
            resolution: Some("720p".to_string()),
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
    fn build_request_includes_video_fields() {
        let mut opts = sample_opts();
        opts.image_url = Some("https://cdn.x.ai/source.png".to_string());
        let request = build_request(&opts, "grok-imagine-video");
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
        let request = build_request(&opts, "grok-imagine-video");
        assert_eq!(
            request["reference_images"][0]["url"],
            "https://cdn.x.ai/ref-1.png"
        );
        assert_eq!(request["duration"], 8);
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
