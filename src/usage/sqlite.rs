use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::error::AppError;

use super::model::{
    RateLimitBucket, RateLimitsCapture, RateLimitsData, SessionRecord, SessionSummary, UsageDelta,
    UsageEventSummary,
};

const DEFAULT_SESSION_FILE: &str = "session.db";
const ACTIVE_SESSION_META_KEY: &str = "active_session_id";

#[derive(Debug, Clone)]
pub struct SessionStore {
    pub path: PathBuf,
}

impl SessionStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".grok-cli")
                .join(DEFAULT_SESSION_FILE);
        }
        PathBuf::from(".grok-cli").join(DEFAULT_SESSION_FILE)
    }

    pub fn ensure_schema(&self) -> Result<(), AppError> {
        let connection = self.open()?;
        connection
            .execute_batch(
                r#"
BEGIN;
CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY,
  started_at TEXT NOT NULL,
  last_activity_at TEXT NOT NULL,
  provider TEXT NOT NULL,
  active_model TEXT NULL,
  request_count INTEGER NOT NULL DEFAULT 0,
  input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL DEFAULT 0,
  cache_read_tokens INTEGER NOT NULL DEFAULT 0,
  cache_write_tokens INTEGER NOT NULL DEFAULT 0,
  reasoning_tokens INTEGER NOT NULL DEFAULT 0,
  estimated_cost_micro_usd INTEGER NOT NULL DEFAULT 0,
  context_window_tokens INTEGER NULL,
  compression_count INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT NULL
);
CREATE TABLE IF NOT EXISTS session_events (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  command TEXT NOT NULL,
  provider TEXT NOT NULL,
  model TEXT NULL,
  started_at TEXT NOT NULL,
  completed_at TEXT NOT NULL,
  duration_ms INTEGER NOT NULL,
  input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL DEFAULT 0,
  cache_read_tokens INTEGER NOT NULL DEFAULT 0,
  cache_write_tokens INTEGER NOT NULL DEFAULT 0,
  reasoning_tokens INTEGER NOT NULL DEFAULT 0,
  estimated_cost_micro_usd INTEGER NOT NULL DEFAULT 0,
  context_window_tokens INTEGER NULL,
  request_id TEXT NULL,
  metadata_json TEXT NULL
);
CREATE TABLE IF NOT EXISTS rate_limit_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  event_id TEXT NULL,
  provider TEXT NOT NULL,
  captured_at TEXT NOT NULL,
  requests_per_minute_limit INTEGER NULL,
  requests_per_minute_remaining INTEGER NULL,
  requests_per_minute_reset_seconds INTEGER NULL,
  requests_per_hour_limit INTEGER NULL,
  requests_per_hour_remaining INTEGER NULL,
  requests_per_hour_reset_seconds INTEGER NULL,
  tokens_per_minute_limit INTEGER NULL,
  tokens_per_minute_remaining INTEGER NULL,
  tokens_per_minute_reset_seconds INTEGER NULL,
  tokens_per_hour_limit INTEGER NULL,
  tokens_per_hour_remaining INTEGER NULL,
  tokens_per_hour_reset_seconds INTEGER NULL
);
CREATE TABLE IF NOT EXISTS metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
COMMIT;
"#,
            )
            .map_err(|error| {
                AppError::io(format!("failed to initialize session database: {error}"))
            })?;
        Ok(())
    }

    pub fn active_session_id(&self) -> Result<Option<String>, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                "SELECT value FROM metadata WHERE key = ?1",
                params![ACTIVE_SESSION_META_KEY],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| AppError::io(format!("failed to read active session id: {error}")))
    }

    pub fn set_active_session_id(&self, session_id: &str) -> Result<(), AppError> {
        let connection = self.open()?;
        connection
            .execute(
                "INSERT INTO metadata(key, value) VALUES(?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![ACTIVE_SESSION_META_KEY, session_id],
            )
            .map_err(|error| {
                AppError::io(format!("failed to persist active session id: {error}"))
            })?;
        Ok(())
    }

    pub fn ensure_active_session(
        &self,
        explicit_session_id: Option<&str>,
        provider: &str,
    ) -> Result<SessionRecord, AppError> {
        self.ensure_schema()?;

        if let Some(session_id) = explicit_session_id {
            if let Some(existing) = self.load_session(session_id)? {
                self.set_active_session_id(session_id)?;
                return Ok(existing);
            }
            return self.create_session(session_id, provider);
        }

        if let Some(env_session_id) = std::env::var_os("GROK_CLI_SESSION_ID")
            .and_then(|value| value.into_string().ok())
            .filter(|value| !value.trim().is_empty())
        {
            if let Some(existing) = self.load_session(&env_session_id)? {
                self.set_active_session_id(&env_session_id)?;
                return Ok(existing);
            }
            return self.create_session(&env_session_id, provider);
        }

        if let Some(active) = self.active_session_id()? {
            if let Some(existing) = self.load_session(&active)? {
                return Ok(existing);
            }
        }

        let generated = format!("sess_{}", Uuid::new_v4().simple());
        self.create_session(&generated, provider)
    }

    pub fn create_session(
        &self,
        session_id: &str,
        provider: &str,
    ) -> Result<SessionRecord, AppError> {
        let now = now_rfc3339();
        let connection = self.open()?;
        connection
            .execute(
                "INSERT INTO sessions(
                    session_id, started_at, last_activity_at, provider, active_model, request_count,
                    input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                    reasoning_tokens, estimated_cost_micro_usd, context_window_tokens,
                    compression_count, metadata_json
                ) VALUES(?1, ?2, ?3, ?4, NULL, 0, 0, 0, 0, 0, 0, 0, NULL, 0, NULL)",
                params![session_id, now, now, provider],
            )
            .map_err(|error| {
                AppError::io(format!("failed to create session {session_id}: {error}"))
            })?;
        self.set_active_session_id(session_id)?;
        self.load_session(session_id)?.ok_or_else(|| {
            AppError::io(format!("failed to load newly created session {session_id}"))
        })
    }

    pub fn load_session(&self, session_id: &str) -> Result<Option<SessionRecord>, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                "SELECT
                    session_id,
                    started_at,
                    last_activity_at,
                    provider,
                    active_model,
                    request_count,
                    input_tokens,
                    output_tokens,
                    cache_read_tokens,
                    cache_write_tokens,
                    reasoning_tokens,
                    estimated_cost_micro_usd,
                    context_window_tokens,
                    compression_count
                 FROM sessions
                 WHERE session_id = ?1",
                params![session_id],
                |row| {
                    Ok(SessionRecord {
                        session_id: row.get(0)?,
                        started_at: row.get(1)?,
                        last_activity_at: row.get(2)?,
                        provider: row.get(3)?,
                        active_model: row.get(4)?,
                        request_count: row.get::<_, i64>(5)? as u64,
                        input_tokens: row.get::<_, i64>(6)? as u64,
                        output_tokens: row.get::<_, i64>(7)? as u64,
                        cache_read_tokens: row.get::<_, i64>(8)? as u64,
                        cache_write_tokens: row.get::<_, i64>(9)? as u64,
                        reasoning_tokens: row.get::<_, i64>(10)? as u64,
                        estimated_cost_micro_usd: row.get(11)?,
                        context_window_tokens: row
                            .get::<_, Option<i64>>(12)?
                            .map(|value| value as u64),
                        compression_count: row.get::<_, i64>(13)? as u64,
                    })
                },
            )
            .optional()
            .map_err(|error| AppError::io(format!("failed to load session {session_id}: {error}")))
    }

    pub fn record_usage(&self, session_id: &str, delta: &UsageDelta) -> Result<(), AppError> {
        self.ensure_schema()?;
        let connection = self.open()?;
        let now = now_rfc3339();
        let event_id = format!("evt_{}", Uuid::new_v4().simple());

        connection
            .execute(
                "INSERT INTO session_events(
                    event_id, session_id, command, provider, model, started_at, completed_at, duration_ms,
                    input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                    reasoning_tokens, estimated_cost_micro_usd, context_window_tokens, request_id, metadata_json
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9, ?10, ?11, ?12, ?13, ?14, NULL, NULL)",
                params![
                    event_id,
                    session_id,
                    delta.command,
                    delta.provider,
                    delta.model,
                    now,
                    now,
                    delta.input_tokens as i64,
                    delta.output_tokens as i64,
                    delta.cache_read_tokens as i64,
                    delta.cache_write_tokens as i64,
                    delta.reasoning_tokens as i64,
                    delta.estimated_cost_micro_usd,
                    delta.context_window_tokens.map(|value| value as i64),
                ],
            )
            .map_err(|error| AppError::io(format!("failed to record session event: {error}")))?;

        connection
            .execute(
                "UPDATE sessions
                 SET last_activity_at = ?2,
                     provider = ?3,
                     active_model = COALESCE(?4, active_model),
                     request_count = request_count + 1,
                     input_tokens = input_tokens + ?5,
                     output_tokens = output_tokens + ?6,
                     cache_read_tokens = cache_read_tokens + ?7,
                     cache_write_tokens = cache_write_tokens + ?8,
                     reasoning_tokens = reasoning_tokens + ?9,
                     estimated_cost_micro_usd = estimated_cost_micro_usd + ?10,
                     context_window_tokens = COALESCE(?11, context_window_tokens)
                 WHERE session_id = ?1",
                params![
                    session_id,
                    now,
                    delta.provider,
                    delta.model,
                    delta.input_tokens as i64,
                    delta.output_tokens as i64,
                    delta.cache_read_tokens as i64,
                    delta.cache_write_tokens as i64,
                    delta.reasoning_tokens as i64,
                    delta.estimated_cost_micro_usd,
                    delta.context_window_tokens.map(|value| value as i64),
                ],
            )
            .map_err(|error| AppError::io(format!("failed to update session summary: {error}")))?;

        if let Some(rate_limits) = &delta.rate_limits {
            self.record_rate_limits_with_connection(
                &connection,
                session_id,
                &event_id,
                rate_limits,
            )?;
        }

        Ok(())
    }

    pub fn latest_rate_limits(&self, session_id: &str) -> Result<Option<RateLimitsData>, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                "SELECT
                    captured_at,
                    provider,
                    requests_per_minute_limit,
                    requests_per_minute_remaining,
                    requests_per_minute_reset_seconds,
                    requests_per_hour_limit,
                    requests_per_hour_remaining,
                    requests_per_hour_reset_seconds,
                    tokens_per_minute_limit,
                    tokens_per_minute_remaining,
                    tokens_per_minute_reset_seconds,
                    tokens_per_hour_limit,
                    tokens_per_hour_remaining,
                    tokens_per_hour_reset_seconds
                 FROM rate_limit_snapshots
                 WHERE session_id = ?1
                 ORDER BY captured_at DESC
                 LIMIT 1",
                params![session_id],
                |row| {
                    Ok(RateLimitsData {
                        available: true,
                        captured_at: Some(row.get(0)?),
                        provider: Some(row.get(1)?),
                        requests_per_minute: capture_bucket_from_row(
                            row.get::<_, Option<i64>>(2)?,
                            row.get::<_, Option<i64>>(3)?,
                            row.get::<_, Option<i64>>(4)?,
                        ),
                        requests_per_hour: capture_bucket_from_row(
                            row.get::<_, Option<i64>>(5)?,
                            row.get::<_, Option<i64>>(6)?,
                            row.get::<_, Option<i64>>(7)?,
                        ),
                        tokens_per_minute: capture_bucket_from_row(
                            row.get::<_, Option<i64>>(8)?,
                            row.get::<_, Option<i64>>(9)?,
                            row.get::<_, Option<i64>>(10)?,
                        ),
                        tokens_per_hour: capture_bucket_from_row(
                            row.get::<_, Option<i64>>(11)?,
                            row.get::<_, Option<i64>>(12)?,
                            row.get::<_, Option<i64>>(13)?,
                        ),
                    })
                },
            )
            .optional()
            .map_err(|error| {
                AppError::io(format!(
                    "failed to load latest rate-limit snapshot: {error}"
                ))
            })
    }

    pub fn tracked_command_count(&self, session_id: &str) -> Result<u64, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                "SELECT COUNT(*) FROM session_events WHERE session_id = ?1",
                params![session_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count as u64)
            .map_err(|error| AppError::io(format!("failed to count session events: {error}")))
    }

    pub fn session_models(&self, session_id: &str) -> Result<Vec<String>, AppError> {
        let connection = self.open()?;
        let mut statement = connection
            .prepare(
                "SELECT DISTINCT model
                 FROM session_events
                 WHERE session_id = ?1 AND model IS NOT NULL
                 ORDER BY model",
            )
            .map_err(|error| {
                AppError::io(format!("failed to prepare session model query: {error}"))
            })?;
        let rows = statement
            .query_map(params![session_id], |row| row.get::<_, String>(0))
            .map_err(|error| AppError::io(format!("failed to read session models: {error}")))?;
        let mut models = Vec::new();
        for row in rows {
            models.push(row.map_err(|error| {
                AppError::io(format!("failed to decode session model row: {error}"))
            })?);
        }
        Ok(models)
    }

    pub fn usage_event_summaries(
        &self,
        session_id: &str,
    ) -> Result<Vec<UsageEventSummary>, AppError> {
        let connection = self.open()?;
        let mut statement = connection
            .prepare(
                "SELECT
                    command,
                    COUNT(*) AS request_count,
                    COALESCE(SUM(input_tokens), 0) AS input_tokens,
                    COALESCE(SUM(output_tokens), 0) AS output_tokens,
                    COALESCE(SUM(cache_read_tokens), 0) AS cache_read_tokens,
                    COALESCE(SUM(cache_write_tokens), 0) AS cache_write_tokens,
                    COALESCE(SUM(reasoning_tokens), 0) AS reasoning_tokens,
                    COALESCE(SUM(estimated_cost_micro_usd), 0) AS estimated_cost_micro_usd
                 FROM session_events
                 WHERE session_id = ?1
                 GROUP BY command
                 ORDER BY command",
            )
            .map_err(|error| {
                AppError::io(format!(
                    "failed to prepare usage event summary query: {error}"
                ))
            })?;

        let rows = statement
            .query_map(params![session_id], |row| {
                Ok(UsageEventSummary {
                    command: row.get(0)?,
                    request_count: row.get::<_, i64>(1)? as u64,
                    input_tokens: row.get::<_, i64>(2)? as u64,
                    output_tokens: row.get::<_, i64>(3)? as u64,
                    cache_read_tokens: row.get::<_, i64>(4)? as u64,
                    cache_write_tokens: row.get::<_, i64>(5)? as u64,
                    reasoning_tokens: row.get::<_, i64>(6)? as u64,
                    estimated_cost_micro_usd: row.get(7)?,
                })
            })
            .map_err(|error| {
                AppError::io(format!("failed to read usage event summaries: {error}"))
            })?;

        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row.map_err(|error| {
                AppError::io(format!("failed to decode usage event summary row: {error}"))
            })?);
        }
        Ok(summaries)
    }

    pub fn build_session_summary(
        &self,
        record: &SessionRecord,
    ) -> Result<SessionSummary, AppError> {
        let tracked_command_count = self.tracked_command_count(&record.session_id)?;
        let models = self.session_models(&record.session_id)?;
        let duration_seconds =
            duration_seconds_between(&record.started_at, &record.last_activity_at);

        Ok(SessionSummary {
            session_id: record.session_id.clone(),
            started_at: Some(record.started_at.clone()),
            last_activity_at: Some(record.last_activity_at.clone()),
            duration_seconds,
            request_count: record.request_count,
            tracked_command_count,
            models,
            session_store_path: self.path.display().to_string(),
        })
    }

    fn record_rate_limits_with_connection(
        &self,
        connection: &Connection,
        session_id: &str,
        event_id: &str,
        rate_limits: &RateLimitsCapture,
    ) -> Result<(), AppError> {
        let snapshot_id = format!("rl_{}", Uuid::new_v4().simple());
        let rpm = rate_limits.requests_per_minute.clone().unwrap_or_default();
        let rph = rate_limits.requests_per_hour.clone().unwrap_or_default();
        let tpm = rate_limits.tokens_per_minute.clone().unwrap_or_default();
        let tph = rate_limits.tokens_per_hour.clone().unwrap_or_default();

        connection
            .execute(
                "INSERT INTO rate_limit_snapshots(
                    snapshot_id, session_id, event_id, provider, captured_at,
                    requests_per_minute_limit, requests_per_minute_remaining, requests_per_minute_reset_seconds,
                    requests_per_hour_limit, requests_per_hour_remaining, requests_per_hour_reset_seconds,
                    tokens_per_minute_limit, tokens_per_minute_remaining, tokens_per_minute_reset_seconds,
                    tokens_per_hour_limit, tokens_per_hour_remaining, tokens_per_hour_reset_seconds
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    snapshot_id,
                    session_id,
                    event_id,
                    rate_limits.provider,
                    rate_limits.captured_at,
                    nonzero_or_null(rpm.limit),
                    nonzero_or_null(rpm.remaining),
                    nonzero_or_null(rpm.reset_seconds),
                    nonzero_or_null(rph.limit),
                    nonzero_or_null(rph.remaining),
                    nonzero_or_null(rph.reset_seconds),
                    nonzero_or_null(tpm.limit),
                    nonzero_or_null(tpm.remaining),
                    nonzero_or_null(tpm.reset_seconds),
                    nonzero_or_null(tph.limit),
                    nonzero_or_null(tph.remaining),
                    nonzero_or_null(tph.reset_seconds),
                ],
            )
            .map_err(|error| AppError::io(format!("failed to record rate-limit snapshot: {error}")))?;
        Ok(())
    }

    fn open(&self) -> Result<Connection, AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::io(format!(
                    "failed to create session database directory {}: {error}",
                    parent.display()
                ))
            })?;
        }
        Connection::open(&self.path).map_err(|error| {
            AppError::io(format!(
                "failed to open session database {}: {error}",
                self.path.display()
            ))
        })
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn duration_seconds_between(started_at: &str, last_activity_at: &str) -> u64 {
    let start = OffsetDateTime::parse(started_at, &Rfc3339).ok();
    let end = OffsetDateTime::parse(last_activity_at, &Rfc3339).ok();
    match (start, end) {
        (Some(start), Some(end)) if end >= start => (end - start).whole_seconds() as u64,
        _ => 0,
    }
}

fn nonzero_or_null(value: u64) -> Option<i64> {
    if value == 0 { None } else { Some(value as i64) }
}

fn capture_bucket_from_row(
    limit: Option<i64>,
    remaining: Option<i64>,
    reset_seconds: Option<i64>,
) -> Option<RateLimitBucket> {
    let limit = limit?;
    let remaining = remaining.unwrap_or(0);
    let used = (limit - remaining).max(0);
    Some(RateLimitBucket {
        limit: limit as u64,
        remaining: remaining.max(0) as u64,
        used: used as u64,
        reset_seconds: reset_seconds.unwrap_or(0).max(0) as u64,
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::SessionStore;
    use crate::usage::model::{RateLimitCaptureBucket, RateLimitsCapture, UsageDelta};

    #[test]
    fn session_store_creates_and_updates_session() {
        let temp = tempdir().unwrap();
        let store = SessionStore::new(temp.path().join("session.db"));
        let session = store.ensure_active_session(None, "xai-oauth").unwrap();
        assert!(session.session_id.starts_with("sess_"));

        let delta = UsageDelta {
            provider: "xai-oauth".to_string(),
            command: "chat".to_string(),
            model: Some("grok-4.20-reasoning".to_string()),
            input_tokens: 100,
            output_tokens: 50,
            estimated_cost_micro_usd: 700,
            rate_limits: Some(RateLimitsCapture {
                captured_at: "2026-05-20T00:00:00Z".to_string(),
                provider: "xai-oauth".to_string(),
                requests_per_minute: Some(RateLimitCaptureBucket {
                    limit: 60,
                    remaining: 40,
                    reset_seconds: 20,
                }),
                ..RateLimitsCapture::default()
            }),
            ..UsageDelta::default()
        };
        store.record_usage(&session.session_id, &delta).unwrap();

        let updated = store.load_session(&session.session_id).unwrap().unwrap();
        assert_eq!(updated.request_count, 1);
        assert_eq!(updated.input_tokens, 100);
        assert_eq!(updated.output_tokens, 50);

        let latest = store
            .latest_rate_limits(&session.session_id)
            .unwrap()
            .unwrap();
        assert!(latest.available);
        assert_eq!(latest.requests_per_minute.unwrap().remaining, 40);

        let breakdown = store.usage_event_summaries(&session.session_id).unwrap();
        assert_eq!(breakdown.len(), 1);
        assert_eq!(breakdown[0].command, "chat");
        assert_eq!(breakdown[0].request_count, 1);
    }
}
