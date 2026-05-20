use std::fs;
use std::path::{Path, PathBuf};

use reqwest::blocking::multipart::{Form, Part};
use serde::Serialize;
use serde_json::json;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::app::AppContext;
use crate::args::{SttOptions, TtsOptions};
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::tracker;

const DEFAULT_TTS_MODEL: &str = "grok-tts";
const DEFAULT_STT_MODEL: &str = "grok-transcribe";
const TTS_PATH: &str = "/tts";
const STT_PATH: &str = "/stt";
const DEFAULT_AUDIO_EXTENSION: &str = "mp3";
const DEFAULT_TTS_VOICE_ID: &str = "eve";
const DEFAULT_TTS_LANGUAGE: &str = "en";
const DEFAULT_STT_LANGUAGE: &str = "en";

#[derive(Debug, Clone, Serialize)]
struct TtsData {
    success: bool,
    provider: String,
    credential_source: String,
    file_path: String,
    media_tag: String,
    voice_compatible: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SttData {
    success: bool,
    provider: String,
    credential_source: String,
    transcript: String,
}

pub fn execute_tts(ctx: &AppContext, opts: TtsOptions) -> CommandResult {
    let command = "tts";
    validate_tts_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts
        .model
        .clone()
        .unwrap_or_else(|| model::default_model_for_task(state.as_ref(), "tts", DEFAULT_TTS_MODEL));
    let request = build_tts_request(&opts, &model);
    let upstream = upstream::post_bytes_api_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        TTS_PATH,
        &request,
        opts.timeout,
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let output_path = resolve_tts_output_path(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    write_audio_file(&output_path, &upstream.bytes)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = TtsData {
        success: true,
        provider: "xai".to_string(),
        credential_source: upstream.credential_source,
        file_path: output_path.display().to_string(),
        media_tag: format!("MEDIA:{}", output_path.display()),
        voice_compatible: false,
    };
    tracker::record_usage(
        ctx,
        opts.common.auth_file.as_deref(),
        &data.credential_source,
        UsageDelta {
            provider: data.credential_source.clone(),
            command: command.to_string(),
            model: Some(model),
            rate_limits: upstream.rate_limits.clone(),
            ..UsageDelta::default()
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if opts.common.json {
        output::print_json_success(command, &data);
    } else {
        println!("success: {}", data.success);
        println!("provider: {}", data.provider);
        println!("credential_source: {}", data.credential_source);
        println!("file_path: {}", data.file_path);
        println!("media_tag: {}", data.media_tag);
        println!("voice_compatible: {}", data.voice_compatible);
    }

    Ok(())
}

pub fn execute_stt(ctx: &AppContext, opts: SttOptions) -> CommandResult {
    let command = "stt";
    validate_stt_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let auth_file = opts.common.auth_file.as_deref();
    let state = auth_file
        .map(|path| ctx.state_store.resolve_path(Some(path)))
        .or_else(|| Some(ctx.state_store.resolve_path(None)))
        .and_then(|path| ctx.state_store.load_valid_state(&path).ok());
    let model = opts
        .model
        .clone()
        .unwrap_or_else(|| model::default_model_for_task(state.as_ref(), "stt", DEFAULT_STT_MODEL));
    let form = build_stt_form(&opts, &model)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let upstream = upstream::post_multipart_api_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        STT_PATH,
        form,
        opts.timeout,
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let transcript = extract_transcript(&upstream.response)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = SttData {
        success: true,
        provider: "xai".to_string(),
        credential_source: upstream.credential_source,
        transcript,
    };
    tracker::record_usage(
        ctx,
        opts.common.auth_file.as_deref(),
        &data.credential_source,
        UsageDelta {
            provider: data.credential_source.clone(),
            command: command.to_string(),
            model: Some(model),
            input_tokens: upstream.usage.input_tokens,
            output_tokens: upstream.usage.output_tokens,
            cache_read_tokens: upstream.usage.cache_read_tokens,
            cache_write_tokens: upstream.usage.cache_write_tokens,
            reasoning_tokens: upstream.usage.reasoning_tokens,
            estimated_cost_micro_usd: 0,
            context_window_tokens: None,
            rate_limits: upstream.rate_limits.clone(),
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if opts.common.json {
        output::print_json_success(command, &data);
    } else {
        println!("success: {}", data.success);
        println!("provider: {}", data.provider);
        println!("credential_source: {}", data.credential_source);
        println!("transcript: {}", data.transcript);
    }

    Ok(())
}

fn validate_tts_options(opts: &TtsOptions) -> Result<(), AppError> {
    if tts_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "text must not be empty",
        ));
    }
    Ok(())
}

fn validate_stt_options(opts: &SttOptions) -> Result<(), AppError> {
    let file = stt_file(opts);
    if !file.exists() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            format!("file does not exist: {}", file.display()),
        ));
    }
    Ok(())
}

fn build_tts_request(opts: &TtsOptions, _model: &str) -> serde_json::Value {
    let mut body = serde_json::Map::new();
    body.insert("text".to_string(), json!(tts_text(opts)));
    body.insert(
        "voice_id".to_string(),
        json!(opts.voice_id.as_deref().unwrap_or(DEFAULT_TTS_VOICE_ID)),
    );
    body.insert(
        "language".to_string(),
        json!(opts.language.as_deref().unwrap_or(DEFAULT_TTS_LANGUAGE)),
    );

    if opts
        .output
        .as_deref()
        .and_then(|path| path.extension())
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
    {
        body.insert(
            "output_format".to_string(),
            json!({
                "codec": "wav",
                "sample_rate": 24000
            }),
        );
    }

    serde_json::Value::Object(body)
}

fn resolve_tts_output_path(opts: &TtsOptions) -> Result<PathBuf, AppError> {
    if let Some(path) = opts.output.clone() {
        return Ok(path);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::io("HOME is not set; cannot derive default audio output path"))?;
    let directory = home
        .join(".hermes")
        .join("cache")
        .join("audio")
        .join("audio_cache");
    let stamp = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "tts".to_string())
        .replace(':', "-");
    Ok(directory.join(format!("grok-tts-{stamp}.{DEFAULT_AUDIO_EXTENSION}")))
}

fn write_audio_file(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::io(format!(
                "failed to create audio output directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    fs::write(path, bytes).map_err(|error| {
        AppError::io(format!(
            "failed to write audio output file {}: {error}",
            path.display()
        ))
    })
}

fn build_stt_form(opts: &SttOptions, _model: &str) -> Result<Form, AppError> {
    let file = stt_file(opts);
    let bytes = fs::read(file).map_err(|error| {
        AppError::io(format!(
            "failed to read transcription file {}: {error}",
            file.display()
        ))
    })?;

    let file_name = file
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("audio.bin")
        .to_string();

    let file_part = Part::bytes(bytes).file_name(file_name);
    let mut form = Form::new().part("file", file_part).text("format", "true");

    form = form.text(
        "language",
        opts.language
            .as_deref()
            .unwrap_or(DEFAULT_STT_LANGUAGE)
            .to_string(),
    );

    Ok(form)
}

fn tts_text(opts: &TtsOptions) -> &str {
    opts.text
        .as_deref()
        .or(opts.text_flag.as_deref())
        .unwrap_or("")
}

fn stt_file(opts: &SttOptions) -> &Path {
    opts.file
        .as_deref()
        .or(opts.file_flag.as_deref())
        .unwrap_or_else(|| Path::new(""))
}

fn extract_transcript(response: &serde_json::Value) -> Result<String, AppError> {
    response
        .get("text")
        .or_else(|| response.get("transcript"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                "stt payload did not include a transcript",
            )
        })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        build_stt_form, build_tts_request, extract_transcript, resolve_tts_output_path,
        validate_stt_options, validate_tts_options, write_audio_file,
    };
    use crate::args::{SttOptions, TaskCommonOptions, TtsOptions};
    use serde_json::json;
    use tempfile::tempdir;

    fn sample_tts_opts() -> TtsOptions {
        TtsOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            text: Some("Hello world".to_string()),
            text_flag: None,
            voice_id: Some("alloy".to_string()),
            language: Some("en".to_string()),
            output: None,
            model: Some("grok-tts".to_string()),
            timeout: Some(60),
        }
    }

    fn sample_stt_opts(path: std::path::PathBuf) -> SttOptions {
        SttOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            file: Some(path),
            file_flag: None,
            model: Some("grok-transcribe".to_string()),
            language: Some("en".to_string()),
            timeout: Some(60),
        }
    }

    #[test]
    fn validate_tts_options_rejects_empty_text() {
        let mut opts = sample_tts_opts();
        opts.text = Some("   ".to_string());
        let error = validate_tts_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
    }

    #[test]
    fn build_tts_request_includes_fields() {
        let opts = sample_tts_opts();
        let request = build_tts_request(&opts, "grok-tts");
        assert_eq!(request["text"], "Hello world");
        assert_eq!(request["voice_id"], "alloy");
        assert_eq!(request["language"], "en");
    }

    #[test]
    fn resolve_tts_output_path_uses_explicit_output_when_present() {
        let mut opts = sample_tts_opts();
        opts.output = Some(PathBuf::from("/tmp/custom.mp3"));
        let path = resolve_tts_output_path(&opts).unwrap();
        assert_eq!(path, PathBuf::from("/tmp/custom.mp3"));
    }

    #[test]
    fn write_audio_file_persists_bytes() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("voice.mp3");
        write_audio_file(&path, b"audio-bytes").unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"audio-bytes");
    }

    #[test]
    fn validate_stt_options_rejects_missing_file() {
        let opts = sample_stt_opts(PathBuf::from("/tmp/missing.wav"));
        let error = validate_stt_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
    }

    #[test]
    fn build_stt_form_accepts_existing_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("sample.wav");
        std::fs::write(&path, b"wave").unwrap();
        let opts = sample_stt_opts(path);
        let form = build_stt_form(&opts, "grok-transcribe");
        assert!(form.is_ok());
    }

    #[test]
    fn extract_transcript_accepts_text_field() {
        let transcript = extract_transcript(&json!({"text":"hello"})).unwrap();
        assert_eq!(transcript, "hello");
    }
}
