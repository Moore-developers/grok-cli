use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use reqwest::blocking::multipart::{Form, Part};
use serde::Serialize;
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tungstenite::client::IntoClientRequest;
use tungstenite::{Message, connect};

use crate::app::AppContext;
use crate::args::{SttOptions, SttStreamOptions, TtsOptions};
use crate::auth::resolver::{RuntimeCredentialOptions, resolve_runtime_credentials_with_options};
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::model;
use crate::output;
use crate::upstream;
use crate::usage::model::UsageDelta;
use crate::usage::tracker;

const DEFAULT_TTS_MODEL: &str = "grok-tts";
const DEFAULT_STT_MODEL: &str = "grok-transcribe";
const DEFAULT_STT_STREAM_MODEL: &str = "grok-transcribe";
const TTS_PATH: &str = "/tts";
const TTS_VOICES_PATH: &str = "/tts/voices";
const STT_PATH: &str = "/stt";
const STT_STREAM_PATH: &str = "/stt";
const DEFAULT_AUDIO_EXTENSION: &str = "mp3";
const DEFAULT_TTS_VOICE_ID: &str = "eve";
const DEFAULT_TTS_LANGUAGE: &str = "en";
const DEFAULT_STT_LANGUAGE: &str = "en";
const DEFAULT_TTS_SAMPLE_RATE: u32 = 24_000;
const DEFAULT_STT_STREAM_CHUNK_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize)]
struct TtsData {
    success: bool,
    provider: String,
    credential_source: String,
    file_path: String,
    media_tag: String,
    voice_compatible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_format: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct TtsVoicesData {
    success: bool,
    provider: String,
    credential_source: String,
    voices: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TtsOutputFormat {
    codec: String,
    sample_rate: Option<u32>,
    bit_rate: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
struct SttData {
    success: bool,
    provider: String,
    credential_source: String,
    transcript: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    words: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SttInput {
    File(PathBuf),
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SttFormFields {
    input: SttInput,
    format: bool,
    language: String,
    audio_format: Option<String>,
    sample_rate: Option<u32>,
    multichannel: bool,
    channels: Option<String>,
    diarize: bool,
    keyterms: Vec<String>,
    filler_words: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SttStreamSummaryData {
    success: bool,
    provider: String,
    credential_source: String,
    events: Vec<SttStreamEvent>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct SttStreamEvent {
    event_type: String,
    transcript: Option<String>,
    is_final: bool,
    raw: Value,
}

pub fn execute_tts(ctx: &AppContext, opts: TtsOptions) -> CommandResult {
    let command = "tts";
    validate_tts_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    if opts.list_voices {
        return execute_tts_voices(ctx, &opts);
    }

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
        output_format: tts_output_format(&opts).map(tts_output_format_json),
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
        if let Some(output_format) = &data.output_format {
            println!("output_format: {output_format}");
        }
    }

    Ok(())
}

fn execute_tts_voices(ctx: &AppContext, opts: &TtsOptions) -> CommandResult {
    let command = "tts";
    let upstream = upstream::get_json_api_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        TTS_VOICES_PATH,
        opts.timeout,
        upstream::UpstreamAuthOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let data = TtsVoicesData {
        success: true,
        provider: "xai".to_string(),
        credential_source: upstream.credential_source,
        voices: extract_tts_voices(&upstream.response),
    };

    if opts.common.json {
        output::print_json_success(command, &data);
    } else {
        println!("success: {}", data.success);
        println!("provider: {}", data.provider);
        println!("credential_source: {}", data.credential_source);
        print_tts_voices(&data.voices);
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

    let data = parse_stt_response(&upstream.credential_source, &upstream.response)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

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
        if let Some(language) = &data.language {
            println!("language: {language}");
        }
        if let Some(duration) = data.duration {
            println!("duration: {duration}");
        }
    }

    Ok(())
}

pub fn execute_stt_stream(ctx: &AppContext, opts: SttStreamOptions) -> CommandResult {
    let command = "stt-stream";
    validate_stt_stream_options(&opts)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let credentials = resolve_runtime_credentials_with_options(
        ctx,
        opts.common.auth_file.as_deref(),
        RuntimeCredentialOptions {
            refresh_if_expiring: true,
        },
    )
    .map_err(|error| CommandError::new(command, opts.common.json, error))?;

    let model = opts
        .model
        .clone()
        .unwrap_or_else(|| DEFAULT_STT_STREAM_MODEL.to_string());
    let endpoint = build_stt_stream_url(&credentials.base_url, &opts, &model)
        .map_err(|error| CommandError::new(command, opts.common.json, error))?;
    let file_path = stt_stream_file(&opts);
    let bytes = fs::read(file_path).map_err(|error| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::io(format!(
                "failed to read streaming transcription file {}: {error}",
                file_path.display()
            )),
        )
    })?;

    let mut request = endpoint.into_client_request().map_err(|error| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::new(
                ErrorCode::RequestFailed,
                format!("failed to build streaming STT WebSocket request: {error}"),
            ),
        )
    })?;
    request.headers_mut().insert(
        tungstenite::http::header::AUTHORIZATION,
        tungstenite::http::HeaderValue::from_str(&format!(
            "{} {}",
            credentials.token_type, credentials.access_token
        ))
        .map_err(|error| {
            CommandError::new(
                command,
                opts.common.json,
                AppError::new(
                    ErrorCode::RequestFailed,
                    format!("failed to build authorization header: {error}"),
                ),
            )
        })?,
    );

    let (mut socket, _) = connect(request).map_err(|error| {
        CommandError::new(
            command,
            opts.common.json,
            AppError::new(
                ErrorCode::RequestFailed,
                format!("streaming STT WebSocket connection failed: {error}"),
            ),
        )
    })?;
    for chunk in bytes.chunks(DEFAULT_STT_STREAM_CHUNK_SIZE) {
        socket
            .write(Message::Binary(chunk.to_vec().into()))
            .map_err(|error| {
                CommandError::new(
                    command,
                    opts.common.json,
                    AppError::new(
                        ErrorCode::RequestFailed,
                        format!("failed to stream audio chunk: {error}"),
                    ),
                )
            })?;
    }
    socket
        .write(Message::Text(
            json!({"type": "audio.done"}).to_string().into(),
        ))
        .map_err(|error| {
            CommandError::new(
                command,
                opts.common.json,
                AppError::new(
                    ErrorCode::RequestFailed,
                    format!("failed to finish streaming audio: {error}"),
                ),
            )
        })?;

    let mut events = Vec::new();
    loop {
        match socket.read() {
            Ok(Message::Text(text)) => {
                let event = parse_stt_stream_event(text.as_ref())
                    .map_err(|error| CommandError::new(command, opts.common.json, error))?;
                let is_done = event.event_type == "done";
                if opts.common.json {
                    events.push(event);
                } else {
                    print_stt_stream_event(&event);
                }
                if is_done {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(payload)) => {
                let _ = socket.write(Message::Pong(payload));
            }
            Ok(Message::Binary(_)) | Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}
            Err(tungstenite::Error::Io(error))
                if error.kind() == io::ErrorKind::WouldBlock
                    || error.kind() == io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(error) => {
                return Err(CommandError::new(
                    command,
                    opts.common.json,
                    AppError::new(
                        ErrorCode::RequestFailed,
                        format!("streaming STT WebSocket read failed: {error}"),
                    ),
                ));
            }
        }
    }

    if opts.common.json {
        let data = SttStreamSummaryData {
            success: true,
            provider: "xai".to_string(),
            credential_source: credentials.provider,
            events,
        };
        output::print_json_success(command, &data);
    }

    Ok(())
}

fn validate_tts_options(opts: &TtsOptions) -> Result<(), AppError> {
    if opts.list_voices {
        return Ok(());
    }

    if tts_text(opts).trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "text must not be empty",
        ));
    }

    let output_format = tts_output_format(opts);
    if let (Some(explicit), Some(path_format)) = (
        output_format.as_ref(),
        opts.output
            .as_deref()
            .and_then(|path| path.extension())
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase()),
    ) && explicit.codec != path_format
    {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            format!(
                "--output extension .{path_format} does not match --output-format {}",
                explicit.codec
            ),
        ));
    }

    Ok(())
}

fn tts_output_format(opts: &TtsOptions) -> Option<TtsOutputFormat> {
    let codec = opts
        .output_format
        .as_deref()
        .and_then(|value| non_empty_string(Some(value)))
        .or_else(|| {
            opts.output
                .as_deref()
                .and_then(|path| path.extension())
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase())
                .filter(|value| value == "wav")
        });

    let codec = codec?;
    Some(TtsOutputFormat {
        codec,
        sample_rate: opts.sample_rate.or_else(|| {
            opts.output
                .as_deref()
                .and_then(|path| path.extension())
                .and_then(|value| value.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
                .then_some(DEFAULT_TTS_SAMPLE_RATE)
        }),
        bit_rate: opts.bit_rate,
    })
}

fn tts_output_format_json(format: TtsOutputFormat) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("codec".to_string(), json!(format.codec));
    if let Some(sample_rate) = format.sample_rate {
        object.insert("sample_rate".to_string(), json!(sample_rate));
    }
    if let Some(bit_rate) = format.bit_rate {
        object.insert("bit_rate".to_string(), json!(bit_rate));
    }
    Value::Object(object)
}

fn validate_stt_options(opts: &SttOptions) -> Result<(), AppError> {
    build_stt_form_fields(opts).map(|_| ())
}

fn validate_stt_stream_options(opts: &SttStreamOptions) -> Result<(), AppError> {
    if stt_stream_file(opts).as_os_str().is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "file must not be empty",
        ));
    }
    Ok(())
}

fn build_stt_form_fields(opts: &SttOptions) -> Result<SttFormFields, AppError> {
    let has_file = opts.file.is_some() || opts.file_flag.is_some();
    let url = opts
        .url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if has_file && url.is_some() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--url cannot be combined with PATH or --file",
        ));
    }

    let input = if let Some(url) = url {
        SttInput::Url(url.to_string())
    } else {
        let file = stt_file(opts);
        if file.as_os_str().is_empty() {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                "provide an audio input with PATH, --file, or --url",
            ));
        }
        if !file.exists() {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                format!("file does not exist: {}", file.display()),
            ));
        }
        SttInput::File(file.to_path_buf())
    };

    Ok(SttFormFields {
        input,
        format: opts.format.unwrap_or(true),
        language: opts
            .language
            .as_deref()
            .unwrap_or(DEFAULT_STT_LANGUAGE)
            .to_string(),
        audio_format: non_empty_string(opts.audio_format.as_deref()),
        sample_rate: opts.sample_rate,
        multichannel: opts.multichannel,
        channels: non_empty_string(opts.channels.as_deref()),
        diarize: opts.diarize,
        keyterms: opts
            .keyterms
            .iter()
            .filter_map(|value| non_empty_string(Some(value)))
            .collect(),
        filler_words: opts.filler_words,
    })
}

fn non_empty_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn bool_field(value: bool) -> String {
    if value { "true" } else { "false" }.to_string()
}

fn build_stt_stream_url(
    base_url: &str,
    opts: &SttStreamOptions,
    model: &str,
) -> Result<String, AppError> {
    let mut url = url::Url::parse(base_url).map_err(|error| {
        AppError::new(
            ErrorCode::InvalidArgs,
            format!("invalid xAI base URL for streaming STT: {error}"),
        )
    })?;
    url.set_scheme(match url.scheme() {
        "https" => "wss",
        "http" => "ws",
        scheme => {
            return Err(AppError::new(
                ErrorCode::InvalidArgs,
                format!("unsupported xAI base URL scheme for streaming STT: {scheme}"),
            ));
        }
    })
    .map_err(|_| {
        AppError::new(
            ErrorCode::InvalidArgs,
            "failed to convert xAI base URL into WebSocket URL",
        )
    })?;
    let base_path = url.path().trim_end_matches('/');
    let stream_path = format!("{base_path}{STT_STREAM_PATH}");
    url.set_path(&stream_path);
    url.set_query(None);
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("model", model);
        query.append_pair(
            "language",
            opts.language.as_deref().unwrap_or(DEFAULT_STT_LANGUAGE),
        );
        if opts.interim_results {
            query.append_pair("interim_results", "true");
        }
        if let Some(endpointing) = non_empty_string(opts.endpointing.as_deref()) {
            query.append_pair("endpointing", &endpointing);
        }
        if let Some(encoding) = non_empty_string(opts.encoding.as_deref()) {
            query.append_pair("encoding", &encoding);
        }
        if let Some(sample_rate) = opts.sample_rate {
            query.append_pair("sample_rate", &sample_rate.to_string());
        }
        if opts.diarize {
            query.append_pair("diarize", "true");
        }
        if opts.filler_words {
            query.append_pair("filler_words", "true");
        }
        if opts.multichannel {
            query.append_pair("multichannel", "true");
        }
        if let Some(channels) = non_empty_string(opts.channels.as_deref()) {
            query.append_pair("channels", &channels);
        }
        for keyterm in &opts.keyterms {
            if let Some(keyterm) = non_empty_string(Some(keyterm)) {
                query.append_pair("keyterm", &keyterm);
            }
        }
    }
    Ok(url.to_string())
}

fn add_text_if_present(form: Form, key: &'static str, value: Option<String>) -> Form {
    match value {
        Some(value) => form.text(key, value),
        None => form,
    }
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

    if let Some(output_format) = tts_output_format(opts) {
        body.insert(
            "output_format".to_string(),
            tts_output_format_json(output_format),
        );
    }
    if let Some(value) = non_empty_string(opts.optimize_streaming_latency.as_deref()) {
        body.insert("optimize_streaming_latency".to_string(), json!(value));
    }
    if let Some(value) = non_empty_string(opts.text_normalization.as_deref()) {
        body.insert("text_normalization".to_string(), json!(value));
    }

    serde_json::Value::Object(body)
}

fn extract_tts_voices(response: &Value) -> Value {
    response
        .get("voices")
        .or_else(|| response.get("data"))
        .cloned()
        .unwrap_or_else(|| response.clone())
}

fn print_tts_voices(voices: &Value) {
    match voices.as_array() {
        Some(items) if items.is_empty() => println!("voices: []"),
        Some(items) => {
            println!("voices:");
            for item in items {
                let id = item
                    .get("voice_id")
                    .or_else(|| item.get("id"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown");
                let name = item
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or(id);
                let kind = item
                    .get("type")
                    .or_else(|| item.get("source"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("voice");
                println!("- {id}\t{name}\t{kind}");
            }
        }
        _ => println!("voices: {voices}"),
    }
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
    build_stt_form_from_fields(&build_stt_form_fields(opts)?)
}

fn build_stt_form_from_fields(fields: &SttFormFields) -> Result<Form, AppError> {
    let form = match &fields.input {
        SttInput::File(file) => {
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
            Form::new().part("file", file_part)
        }
        SttInput::Url(url) => Form::new().text("url", url.clone()),
    };

    let mut form = form
        .text("format", bool_field(fields.format))
        .text("language", fields.language.clone());
    form = add_text_if_present(form, "audio_format", fields.audio_format.clone());
    if let Some(sample_rate) = fields.sample_rate {
        form = form.text("sample_rate", sample_rate.to_string());
    }
    if fields.multichannel {
        form = form.text("multichannel", "true");
    }
    form = add_text_if_present(form, "channels", fields.channels.clone());
    if fields.diarize {
        form = form.text("diarize", "true");
    }
    for keyterm in &fields.keyterms {
        form = form.text("keyterm", keyterm.clone());
    }
    if fields.filler_words {
        form = form.text("filler_words", "true");
    }

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

fn stt_stream_file(opts: &SttStreamOptions) -> &Path {
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

fn parse_stt_response(credential_source: &str, response: &Value) -> Result<SttData, AppError> {
    Ok(SttData {
        success: true,
        provider: "xai".to_string(),
        credential_source: credential_source.to_string(),
        transcript: extract_transcript(response)?,
        language: response
            .get("language")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        duration: response.get("duration").and_then(|value| value.as_f64()),
        words: response.get("words").cloned(),
        channels: response.get("channels").cloned(),
    })
}

fn parse_stt_stream_event(text: &str) -> Result<SttStreamEvent, AppError> {
    let raw = serde_json::from_str::<Value>(text).map_err(|error| {
        AppError::new(
            ErrorCode::RequestFailed,
            format!("failed to decode streaming STT event: {error}"),
        )
    })?;
    let event_type = raw
        .get("type")
        .or_else(|| raw.get("event"))
        .and_then(|value| value.as_str())
        .unwrap_or("transcript")
        .to_string();
    let transcript = raw
        .get("text")
        .or_else(|| raw.get("transcript"))
        .or_else(|| raw.pointer("/channel/alternatives/0/transcript"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let is_final = raw
        .get("is_final")
        .or_else(|| raw.get("final"))
        .and_then(|value| value.as_bool())
        .unwrap_or(matches!(event_type.as_str(), "final" | "transcript.final"));

    Ok(SttStreamEvent {
        event_type,
        transcript,
        is_final,
        raw,
    })
}

fn print_stt_stream_event(event: &SttStreamEvent) {
    match event.transcript.as_deref() {
        Some(transcript) if !transcript.trim().is_empty() => {
            let kind = if event.is_final { "final" } else { "interim" };
            println!("{kind}: {transcript}");
        }
        _ => println!("event: {}", event.event_type),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        SttInput, build_stt_form, build_stt_form_fields, build_stt_stream_url, build_tts_request,
        extract_transcript, parse_stt_response, parse_stt_stream_event, resolve_tts_output_path,
        validate_stt_options, validate_stt_stream_options, validate_tts_options, write_audio_file,
    };
    use crate::args::{SttOptions, SttStreamOptions, TaskCommonOptions, TtsOptions};
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
            list_voices: false,
            voice_id: Some("alloy".to_string()),
            language: Some("en".to_string()),
            output: None,
            output_format: None,
            sample_rate: None,
            bit_rate: None,
            optimize_streaming_latency: None,
            text_normalization: None,
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
            url: None,
            model: Some("grok-transcribe".to_string()),
            language: Some("en".to_string()),
            format: None,
            audio_format: None,
            sample_rate: None,
            multichannel: false,
            channels: None,
            diarize: false,
            keyterms: vec![],
            filler_words: false,
            timeout: Some(60),
        }
    }

    fn sample_stt_stream_opts(path: std::path::PathBuf) -> SttStreamOptions {
        SttStreamOptions {
            common: TaskCommonOptions {
                json: true,
                auth_file: None,
            },
            file: Some(path),
            file_flag: None,
            model: Some("grok-transcribe".to_string()),
            language: Some("en".to_string()),
            interim_results: false,
            endpointing: None,
            encoding: None,
            sample_rate: None,
            diarize: false,
            filler_words: false,
            multichannel: false,
            channels: None,
            keyterms: vec![],
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
    fn validate_tts_options_allows_list_voices_without_text() {
        let mut opts = sample_tts_opts();
        opts.text = None;
        opts.list_voices = true;
        validate_tts_options(&opts).unwrap();
    }

    #[test]
    fn validate_tts_options_rejects_mismatched_output_extension() {
        let mut opts = sample_tts_opts();
        opts.output = Some(PathBuf::from("/tmp/custom.wav"));
        opts.output_format = Some("mp3".to_string());
        let error = validate_tts_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("does not match --output-format"));
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
    fn build_tts_request_includes_explicit_output_format() {
        let mut opts = sample_tts_opts();
        opts.output_format = Some("mp3".to_string());
        opts.sample_rate = Some(24_000);
        opts.bit_rate = Some(128_000);

        let request = build_tts_request(&opts, "grok-tts");
        assert_eq!(request["output_format"]["codec"], "mp3");
        assert_eq!(request["output_format"]["sample_rate"], 24_000);
        assert_eq!(request["output_format"]["bit_rate"], 128_000);
    }

    #[test]
    fn build_tts_request_infers_wav_output_format_from_extension() {
        let mut opts = sample_tts_opts();
        opts.output = Some(PathBuf::from("/tmp/custom.wav"));

        let request = build_tts_request(&opts, "grok-tts");
        assert_eq!(request["output_format"]["codec"], "wav");
        assert_eq!(request["output_format"]["sample_rate"], 24_000);
    }

    #[test]
    fn build_tts_request_includes_advanced_parameters() {
        let mut opts = sample_tts_opts();
        opts.language = Some("auto".to_string());
        opts.optimize_streaming_latency = Some("auto".to_string());
        opts.text_normalization = Some("off".to_string());

        let request = build_tts_request(&opts, "grok-tts");
        assert_eq!(request["language"], "auto");
        assert_eq!(request["optimize_streaming_latency"], "auto");
        assert_eq!(request["text_normalization"], "off");
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
    fn validate_stt_options_rejects_missing_input() {
        let mut opts = sample_stt_opts(PathBuf::from(""));
        opts.file = None;
        let error = validate_stt_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("PATH, --file, or --url"));
    }

    #[test]
    fn validate_stt_stream_options_rejects_missing_file() {
        let mut opts = sample_stt_stream_opts(PathBuf::from(""));
        opts.file = None;
        let error = validate_stt_stream_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("file must not be empty"));
    }

    #[test]
    fn validate_stt_options_rejects_file_and_url_together() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("sample.wav");
        std::fs::write(&path, b"wave").unwrap();
        let mut opts = sample_stt_opts(path);
        opts.url = Some("https://example.com/audio.wav".to_string());
        let error = validate_stt_options(&opts).unwrap_err();
        assert_eq!(error.code.as_str(), "invalid_args");
        assert!(error.message.contains("--url cannot be combined"));
    }

    #[test]
    fn build_stt_form_fields_accepts_url_without_file() {
        let mut opts = sample_stt_opts(PathBuf::from(""));
        opts.file = None;
        opts.url = Some("https://example.com/audio.wav".to_string());

        let fields = build_stt_form_fields(&opts).unwrap();
        assert_eq!(
            fields.input,
            SttInput::Url("https://example.com/audio.wav".to_string())
        );
    }

    #[test]
    fn build_stt_form_fields_captures_advanced_parameters() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("sample.wav");
        std::fs::write(&path, b"wave").unwrap();
        let mut opts = sample_stt_opts(path.clone());
        opts.format = Some(false);
        opts.language = Some("auto".to_string());
        opts.audio_format = Some("pcm_s16le".to_string());
        opts.sample_rate = Some(16_000);
        opts.multichannel = true;
        opts.channels = Some("0,1".to_string());
        opts.diarize = true;
        opts.keyterms = vec!["Grok".to_string(), "xAI".to_string(), "  ".to_string()];
        opts.filler_words = true;

        let fields = build_stt_form_fields(&opts).unwrap();
        assert_eq!(fields.input, SttInput::File(path));
        assert!(!fields.format);
        assert_eq!(fields.language, "auto");
        assert_eq!(fields.audio_format.as_deref(), Some("pcm_s16le"));
        assert_eq!(fields.sample_rate, Some(16_000));
        assert!(fields.multichannel);
        assert_eq!(fields.channels.as_deref(), Some("0,1"));
        assert!(fields.diarize);
        assert_eq!(fields.keyterms, vec!["Grok", "xAI"]);
        assert!(fields.filler_words);
    }

    #[test]
    fn build_stt_stream_url_includes_query_parameters() {
        let mut opts = sample_stt_stream_opts(PathBuf::from("/tmp/audio.raw"));
        opts.language = Some("zh".to_string());
        opts.interim_results = true;
        opts.endpointing = Some("500".to_string());
        opts.encoding = Some("pcm_s16le".to_string());
        opts.sample_rate = Some(16_000);
        opts.diarize = true;
        opts.filler_words = true;
        opts.multichannel = true;
        opts.channels = Some("0,1".to_string());
        opts.keyterms = vec!["Grok".to_string(), "xAI".to_string()];

        let url = build_stt_stream_url("https://api.x.ai/v1", &opts, "grok-transcribe").unwrap();

        assert!(url.starts_with("wss://api.x.ai/v1/stt?"));
        assert!(url.contains("model=grok-transcribe"));
        assert!(url.contains("language=zh"));
        assert!(url.contains("interim_results=true"));
        assert!(url.contains("endpointing=500"));
        assert!(url.contains("encoding=pcm_s16le"));
        assert!(url.contains("sample_rate=16000"));
        assert!(url.contains("diarize=true"));
        assert!(url.contains("filler_words=true"));
        assert!(url.contains("multichannel=true"));
        assert!(url.contains("channels=0%2C1"));
        assert!(url.contains("keyterm=Grok"));
        assert!(url.contains("keyterm=xAI"));
    }

    #[test]
    fn build_stt_stream_url_defaults_language_and_maps_http_to_ws() {
        let mut opts = sample_stt_stream_opts(PathBuf::from("/tmp/audio.raw"));
        opts.language = None;

        let url =
            build_stt_stream_url("http://127.0.0.1:8080/v1", &opts, "grok-transcribe").unwrap();

        assert!(url.starts_with("ws://127.0.0.1:8080/v1/stt?"));
        assert!(url.contains("language=en"));
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

    #[test]
    fn parse_stt_response_keeps_legacy_text_only_payload() {
        let parsed = parse_stt_response("oauth", &json!({"text":"hello"})).unwrap();
        assert_eq!(parsed.transcript, "hello");
        assert_eq!(parsed.language, None);
        assert_eq!(parsed.duration, None);
        assert!(parsed.words.is_none());
        assert!(parsed.channels.is_none());
    }

    #[test]
    fn parse_stt_response_preserves_structured_fields() {
        let response = json!({
            "text": "hello",
            "language": "en",
            "duration": 1.25,
            "words": [{"word": "hello", "start": 0.0, "end": 0.4}],
            "channels": [{"channel": 0, "text": "hello"}]
        });

        let parsed = parse_stt_response("oauth", &response).unwrap();
        assert_eq!(parsed.transcript, "hello");
        assert_eq!(parsed.language.as_deref(), Some("en"));
        assert_eq!(parsed.duration, Some(1.25));
        assert_eq!(parsed.words.unwrap(), response["words"]);
        assert_eq!(parsed.channels.unwrap(), response["channels"]);
    }

    #[test]
    fn parse_stt_stream_event_handles_final_transcript() {
        let event = parse_stt_stream_event(
            r#"{"type":"transcript.final","text":"Hello Grok","is_final":true}"#,
        )
        .unwrap();
        assert_eq!(event.event_type, "transcript.final");
        assert_eq!(event.transcript.as_deref(), Some("Hello Grok"));
        assert!(event.is_final);
    }

    #[test]
    fn parse_stt_stream_event_handles_nested_transcript() {
        let event = parse_stt_stream_event(
            r#"{"type":"Results","channel":{"alternatives":[{"transcript":"Nested text"}]}}"#,
        )
        .unwrap();
        assert_eq!(event.event_type, "Results");
        assert_eq!(event.transcript.as_deref(), Some("Nested text"));
        assert!(!event.is_final);
    }
}
