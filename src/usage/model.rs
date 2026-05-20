use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct UsageCommandData {
    pub provider: String,
    pub session: SessionSummary,
    pub local_usage: LocalUsageSummary,
    pub breakdown: UsageBreakdown,
    pub recent_rate_limits: RateLimitsData,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SessionSummary {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity_at: Option<String>,
    pub duration_seconds: u64,
    pub request_count: u64,
    pub tracked_command_count: u64,
    pub models: Vec<String>,
    pub session_store_path: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LocalUsageSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
    pub pricing_status: String,
    pub pricing_source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_tokens: Option<u64>,
    pub history_turns: u64,
    pub compression_count: u64,
    pub has_unflushed_tracker_data: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UsageBreakdown {
    pub text: UsageCategorySummary,
    pub image: UsageCategorySummary,
    pub video: UsageCategorySummary,
    pub audio: UsageCategorySummary,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UsageCategorySummary {
    pub request_count: u64,
    pub commands: Vec<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RateLimitsData {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_minute: Option<RateLimitBucket>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_hour: Option<RateLimitBucket>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_minute: Option<RateLimitBucket>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_hour: Option<RateLimitBucket>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RateLimitBucket {
    pub limit: u64,
    pub remaining: u64,
    pub used: u64,
    pub reset_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UsageDelta {
    pub provider: String,
    pub command: String,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub estimated_cost_micro_usd: i64,
    pub context_window_tokens: Option<u64>,
    pub rate_limits: Option<RateLimitsCapture>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RateLimitsCapture {
    pub captured_at: String,
    pub provider: String,
    pub requests_per_minute: Option<RateLimitCaptureBucket>,
    pub requests_per_hour: Option<RateLimitCaptureBucket>,
    pub tokens_per_minute: Option<RateLimitCaptureBucket>,
    pub tokens_per_hour: Option<RateLimitCaptureBucket>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RateLimitCaptureBucket {
    pub limit: u64,
    pub remaining: u64,
    pub reset_seconds: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SessionRecord {
    pub session_id: String,
    pub started_at: String,
    pub last_activity_at: String,
    pub provider: String,
    pub active_model: Option<String>,
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub estimated_cost_micro_usd: i64,
    pub context_window_tokens: Option<u64>,
    pub compression_count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct UsageEventSummary {
    pub command: String,
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub estimated_cost_micro_usd: i64,
}
