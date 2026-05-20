use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "grok-cli",
    version,
    about = "OAuth-first CLI for Grok / xAI capabilities",
    long_about = "grok-cli is a local command-line runtime for Grok and xAI OAuth workflows.\n\nIt handles browser OAuth login, token refresh, structured chat/search/media requests, and local session usage accounting from one flat CLI surface.\n\nText commands stream formatted text by default for human use. Use --json for stable non-stream automation output, --no-stream for one final human-readable response, or --raw-stream when your caller needs normalized SSE-style events.\n\nCommon workflows:\n  grok-cli login\n  grok-cli status\n  grok-cli chat \"Summarize today's AI news\"\n  grok-cli chat \"Find AI discussion on X\" --with-x-search\n  grok-cli search \"What are builders saying about Grok today?\"\n  grok-cli image \"A cinematic skyline\"\n  grok-cli usage\n\nScript workflows:\n  grok-cli chat --json --prompt \"Summarize today's AI news\"\n  grok-cli search --json --query \"Grok Hermes latest updates\"\n\nState files:\n  OAuth state:     ~/.grok-cli/auth.json\n  Session usage:   ~/.grok-cli/session.db\n\nUse --json on commands when integrating with scripts, agents, or skills."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: TopLevelCommand,
}

#[derive(Debug, Subcommand)]
pub enum TopLevelCommand {
    /// Start xAI OAuth login in the system browser.
    Login(LoginOptions),
    /// Show whether a usable OAuth session exists.
    Status(StateFileOptions),
    /// Refresh the saved access token.
    Refresh(StateFileOptions),
    /// Delete the saved auth state.
    Logout(StateFileOptions),
    /// Exchange an OAuth authorization code for tokens.
    #[command(name = "exchange-code", hide = true)]
    ExchangeCode(ExchangeCodeOptions),
    /// Show the local redacted auth state.
    State(StateFileOptions),
    /// Show or select default text models for chat and search.
    #[command(alias = "mode")]
    Model(ModelCommand),
    /// Show local session usage.
    Usage(UsageOptions),
    /// Run text chat through the Grok Responses API.
    Chat(ChatOptions),
    /// Search X through the Grok x_search tool.
    Search(XSearchOptions),
    /// Generate an image with Grok Imagine.
    Image(ImageGenOptions),
    /// Generate a video with Grok Imagine.
    Video(VideoGenOptions),
    /// Convert text to speech.
    Tts(TtsOptions),
    /// Transcribe speech to text.
    Stt(SttOptions),
}

#[derive(Debug, Clone, Args)]
#[command(
    about = "Show and select default text models",
    long_about = "Show the current default text model and supported model catalog.\n\nIn an interactive terminal, running `grok-cli model` lets you choose a model with the arrow keys. The selected model is used by both chat and search. For scripts, use `--json` to inspect state or `--model <MODEL>` to save a default model directly."
)]
pub struct ModelCommand {
    #[command(flatten)]
    pub common: StateFileOptions,
    /// Compatibility option. The selected model is shared by chat and search.
    #[arg(long = "command", alias = "task", hide = true)]
    pub task: Option<String>,
    /// Model id to save as the shared default for chat and search.
    #[arg(long)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct StateFileOptions {
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
    /// Override the auth state file path.
    #[arg(long, value_name = "PATH")]
    pub auth_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct TaskCommonOptions {
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
    /// Override the auth state file path.
    #[arg(long, value_name = "PATH")]
    pub auth_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
#[command(
    about = "Show local session usage",
    long_about = "Show local session usage from the SQLite session store.\n\nThe output includes session totals, text/image/video/audio breakdowns, recent rate-limit snapshots, estimated local cost, and context usage. No provider quota lookup is performed."
)]
pub struct UsageOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// Override the session SQLite database path.
    #[arg(long = "session-db", value_name = "PATH")]
    pub session_db: Option<PathBuf>,
    /// Read a specific session id instead of the active session.
    #[arg(long = "session-id")]
    pub session_id: Option<String>,
    /// Compatibility option; account limits are not queried.
    #[arg(long, hide = true)]
    pub timeout: Option<u64>,
    /// Compatibility option; usage is local-only by default.
    #[arg(long = "local-only", hide = true)]
    pub local_only: bool,
}

#[derive(Debug, Clone, Args)]
#[command(
    about = "Run Grok chat",
    long_about = "Run a Grok chat request through the Responses API.\n\nBy default, chat attaches web_search and streams formatted text for human use. Use --json for stable non-stream automation output, --no-stream to force a single final response in human mode, --raw-stream for normalized SSE events, or --with-x-search to include X search alongside web search."
)]
pub struct ChatOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// User prompt text. You can pass it positionally or with --prompt.
    #[arg(value_name = "PROMPT", conflicts_with = "prompt_flag")]
    pub prompt: Option<String>,
    /// User prompt text for scripts that prefer named flags.
    #[arg(long = "prompt", value_name = "PROMPT", id = "prompt_flag")]
    pub prompt_flag: Option<String>,
    /// Optional system or instruction text.
    #[arg(long)]
    pub system: Option<String>,
    /// Override the model for this request.
    #[arg(long)]
    pub model: Option<String>,
    /// Disable default web search.
    #[arg(long = "no-web-search")]
    pub no_web_search: bool,
    /// Add X search in addition to default web search.
    #[arg(long = "with-x-search")]
    pub with_x_search: bool,
    /// Restrict web search to an allowed domain. Repeatable.
    #[arg(long = "allowed-domain")]
    pub allowed_domains: Vec<String>,
    /// Exclude a web search domain. Repeatable.
    #[arg(long = "excluded-domain")]
    pub excluded_domains: Vec<String>,
    /// Enable image understanding for search tools.
    #[arg(long = "enable-image-understanding")]
    pub enable_image_understanding: bool,
    /// Restrict X search to a handle. Repeatable.
    #[arg(long = "allowed-x-handle")]
    pub allowed_x_handles: Vec<String>,
    /// Exclude an X handle. Repeatable.
    #[arg(long = "excluded-x-handle")]
    pub excluded_x_handles: Vec<String>,
    /// Start date filter, formatted as YYYY-MM-DD.
    #[arg(long = "from-date")]
    pub from_date: Option<String>,
    /// End date filter, formatted as YYYY-MM-DD.
    #[arg(long = "to-date")]
    pub to_date: Option<String>,
    /// Enable video understanding for X search.
    #[arg(long = "enable-video-understanding")]
    pub enable_video_understanding: bool,
    /// Explicitly stream formatted text output.
    #[arg(long, conflicts_with = "no_stream")]
    pub stream: bool,
    /// Disable default streaming and print one final response.
    #[arg(long = "no-stream", conflicts_with_all = ["stream", "raw_stream"])]
    pub no_stream: bool,
    /// Print raw normalized SSE events instead of formatted text.
    #[arg(long = "raw-stream", conflicts_with = "no_stream")]
    pub raw_stream: bool,
    /// Request timeout in seconds. Defaults to 3600 for text requests.
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Args)]
#[command(
    about = "Search X with Grok",
    long_about = "Search X with Grok x_search.\n\nSearch streams formatted text by default for human use. Use --json for stable non-stream automation output, --no-stream to force a single final response in human mode, or --raw-stream for normalized SSE events."
)]
pub struct XSearchOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// Search query. You can pass it positionally or with --query.
    #[arg(value_name = "QUERY", conflicts_with = "query_flag")]
    pub query: Option<String>,
    /// Search query for scripts that prefer named flags.
    #[arg(long = "query", value_name = "QUERY", id = "query_flag")]
    pub query_flag: Option<String>,
    /// Restrict search to an X handle. Repeatable.
    #[arg(long = "allowed-x-handle")]
    pub allowed_x_handles: Vec<String>,
    /// Exclude an X handle. Repeatable.
    #[arg(long = "excluded-x-handle")]
    pub excluded_x_handles: Vec<String>,
    /// Start date filter, formatted as YYYY-MM-DD.
    #[arg(long = "from-date")]
    pub from_date: Option<String>,
    /// End date filter, formatted as YYYY-MM-DD.
    #[arg(long = "to-date")]
    pub to_date: Option<String>,
    /// Enable image understanding.
    #[arg(long = "enable-image-understanding")]
    pub enable_image_understanding: bool,
    /// Enable video understanding.
    #[arg(long = "enable-video-understanding")]
    pub enable_video_understanding: bool,
    /// Override the model for this request.
    #[arg(long)]
    pub model: Option<String>,
    /// Explicitly stream formatted text output.
    #[arg(long, conflicts_with = "no_stream")]
    pub stream: bool,
    /// Disable default streaming and print one final response.
    #[arg(long = "no-stream", conflicts_with_all = ["stream", "raw_stream"])]
    pub no_stream: bool,
    /// Print raw normalized SSE events instead of formatted text.
    #[arg(long = "raw-stream", conflicts_with = "no_stream")]
    pub raw_stream: bool,
    /// Request timeout in seconds. Defaults to 3600 for text requests.
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Args)]
#[command(about = "Generate images with Grok Imagine")]
pub struct ImageGenOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// Image prompt. You can pass it positionally or with --prompt.
    #[arg(value_name = "PROMPT", conflicts_with = "prompt_flag")]
    pub prompt: Option<String>,
    /// Image prompt for scripts that prefer named flags.
    #[arg(long = "prompt", value_name = "PROMPT", id = "prompt_flag")]
    pub prompt_flag: Option<String>,
    /// Override the image model for this request.
    #[arg(long)]
    pub model: Option<String>,
    /// Output aspect ratio, for example 16:9 or 1:1.
    #[arg(long = "aspect-ratio")]
    pub aspect_ratio: Option<String>,
    /// Output resolution, for example 1k.
    #[arg(long)]
    pub resolution: Option<String>,
    /// Number of images to generate, from 1 to 10.
    #[arg(long)]
    pub count: Option<u32>,
    /// Image response format, either url or b64_json.
    #[arg(long = "response-format")]
    pub response_format: Option<String>,
    /// Save base64 image output to a local file.
    #[arg(long = "output-file", value_name = "PATH")]
    pub output_file: Option<PathBuf>,
    /// Save base64 image outputs to a local directory.
    #[arg(long = "output-dir", value_name = "PATH")]
    pub output_dir: Option<PathBuf>,
    /// Request timeout in seconds. Defaults to 120 for image generation.
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Args)]
#[command(about = "Generate videos with Grok Imagine")]
pub struct VideoGenOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// Video prompt. You can pass it positionally or with --prompt.
    #[arg(value_name = "PROMPT", conflicts_with = "prompt_flag")]
    pub prompt: Option<String>,
    /// Video prompt for scripts that prefer named flags.
    #[arg(long = "prompt", value_name = "PROMPT", id = "prompt_flag")]
    pub prompt_flag: Option<String>,
    /// Source image URL for image-to-video.
    #[arg(long = "image-url")]
    pub image_url: Option<String>,
    /// Reference image URL. Repeatable.
    #[arg(long = "reference-image-url")]
    pub reference_image_urls: Vec<String>,
    /// Requested duration in seconds.
    #[arg(long)]
    pub duration: Option<u64>,
    /// Output aspect ratio, for example 16:9 or 1:1.
    #[arg(long = "aspect-ratio")]
    pub aspect_ratio: Option<String>,
    /// Output resolution, for example 720p.
    #[arg(long)]
    pub resolution: Option<String>,
    /// Override the video model for this request.
    #[arg(long)]
    pub model: Option<String>,
    /// Total video polling timeout in seconds. Single HTTP requests stay capped at 120 seconds.
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Args)]
#[command(about = "Convert text to speech")]
pub struct TtsOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// Text to synthesize. You can pass it positionally or with --text.
    #[arg(value_name = "TEXT", conflicts_with = "text_flag")]
    pub text: Option<String>,
    /// Text to synthesize for scripts that prefer named flags.
    #[arg(long = "text", value_name = "TEXT", id = "text_flag")]
    pub text_flag: Option<String>,
    /// List available TTS voices instead of synthesizing audio.
    #[arg(long = "list-voices")]
    pub list_voices: bool,
    /// Voice id to use.
    #[arg(long = "voice-id")]
    pub voice_id: Option<String>,
    /// Language code, defaults to en.
    #[arg(long)]
    pub language: Option<String>,
    /// Output audio file path.
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Output audio format, for example mp3 or wav.
    #[arg(long = "output-format")]
    pub output_format: Option<String>,
    /// Output sample rate in Hz.
    #[arg(long = "sample-rate")]
    pub sample_rate: Option<u32>,
    /// Output bit rate in bits per second.
    #[arg(long = "bit-rate")]
    pub bit_rate: Option<u32>,
    /// Streaming latency optimization mode.
    #[arg(long = "optimize-streaming-latency")]
    pub optimize_streaming_latency: Option<String>,
    /// Text normalization mode.
    #[arg(long = "text-normalization")]
    pub text_normalization: Option<String>,
    /// Override the TTS model for this request.
    #[arg(long)]
    pub model: Option<String>,
    /// Request timeout in seconds. Defaults to 120 for TTS.
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Args)]
#[command(about = "Transcribe speech to text")]
pub struct SttOptions {
    #[command(flatten)]
    pub common: TaskCommonOptions,
    /// Audio file to transcribe. You can pass it positionally or with --file.
    #[arg(value_name = "PATH", conflicts_with = "file_flag")]
    pub file: Option<PathBuf>,
    /// Audio file to transcribe for scripts that prefer named flags.
    #[arg(long = "file", value_name = "PATH", id = "file_flag")]
    pub file_flag: Option<PathBuf>,
    /// Remote audio URL to transcribe instead of a local file.
    #[arg(long, value_name = "URL")]
    pub url: Option<String>,
    /// Override the STT model for this request.
    #[arg(long)]
    pub model: Option<String>,
    /// Language code, defaults to en.
    #[arg(long)]
    pub language: Option<String>,
    /// Ask xAI to return formatted text. Defaults to true.
    #[arg(long = "format")]
    pub format: Option<bool>,
    /// Raw audio format when the input has no detectable container metadata.
    #[arg(long = "audio-format")]
    pub audio_format: Option<String>,
    /// Raw audio sample rate in Hz.
    #[arg(long = "sample-rate")]
    pub sample_rate: Option<u32>,
    /// Treat input as multichannel audio.
    #[arg(long)]
    pub multichannel: bool,
    /// Comma-separated channel list to transcribe, for example 0,1.
    #[arg(long)]
    pub channels: Option<String>,
    /// Enable speaker diarization.
    #[arg(long)]
    pub diarize: bool,
    /// Key term to bias transcription toward. Repeatable.
    #[arg(long = "keyterm")]
    pub keyterms: Vec<String>,
    /// Include filler words in the transcript.
    #[arg(long = "filler-words")]
    pub filler_words: bool,
    /// Request timeout in seconds. Defaults to 120 for STT.
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Args)]
#[command(about = "Start OAuth login")]
pub struct LoginOptions {
    #[command(flatten)]
    pub common: StateFileOptions,
    /// Do not open the browser automatically.
    #[arg(long)]
    pub no_browser: bool,
    /// Use manual paste mode instead of loopback callback only.
    #[arg(long)]
    pub manual_paste: bool,
    /// Login timeout in seconds.
    #[arg(long)]
    pub timeout: Option<u64>,
    /// Loopback callback port.
    #[arg(long)]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Args)]
#[command(about = "Exchange an OAuth authorization code")]
pub struct ExchangeCodeOptions {
    #[command(flatten)]
    pub common: StateFileOptions,
    /// Authorization code from the callback.
    #[arg(long)]
    pub code: Option<String>,
    /// OAuth state value.
    #[arg(long)]
    pub state: Option<String>,
    /// Full callback URL or redirect URI.
    #[arg(long)]
    pub redirect_uri: Option<String>,
}
