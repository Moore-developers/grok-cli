use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use tempfile::tempdir;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn usage_returns_local_summary_without_remote_quota() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file, "http://127.0.0.1:9/v1");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--local-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"usage\""))
        .stdout(predicate::str::contains("\"session_store_path\":"))
        .stdout(predicate::str::contains("account_limits").not());
}

#[test]
fn usage_human_output_renders_grouped_sections() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file, "http://127.0.0.1:9/v1");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--local-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Session Usage"))
        .stdout(predicate::str::contains("Usage Breakdown"))
        .stdout(predicate::str::contains("Text"))
        .stdout(predicate::str::contains("Image"))
        .stdout(predicate::str::contains("Video"))
        .stdout(predicate::str::contains("Audio"))
        .stdout(predicate::str::contains("Account limits").not())
        .stdout(predicate::str::contains("Estimated cost:"))
        .stdout(predicate::str::contains("Context:"));
}

#[test]
fn usage_reads_tracked_tokens_after_chat_command() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{
  "output":[{"type":"message","content":[{"type":"output_text","text":"hello from Grok"}]}],
  "usage":{
    "input_tokens":100,
    "output_tokens":25,
    "input_tokens_details":{"cached_tokens":0,"cache_creation_tokens":0},
    "output_tokens_details":{"reasoning_tokens":3}
  }
}"#;
        write_json_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Say hello",
        ])
        .assert()
        .success();

    server.join().unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"input_tokens\":100"))
        .stdout(predicate::str::contains("\"output_tokens\":25"))
        .stdout(predicate::str::contains("\"reasoning_tokens\":3"))
        .stdout(predicate::str::contains("\"request_count\":1"));
}

#[test]
fn usage_does_not_emit_account_limits_when_not_local_only() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file, "https://api.x.ai/v1");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"usage\""))
        .stdout(predicate::str::contains("account_limits").not())
        .stdout(predicate::str::contains("Account limits").not());
}

#[test]
fn usage_json_includes_modality_breakdown() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file, "http://127.0.0.1:9/v1");

    let session_id = "sess_usage_breakdown";
    let session_db = temp.path().join("session.db");

    seed_session_db(
        &session_db,
        session_id,
        &[
            SeedEvent {
                command: "chat",
                input_tokens: 120_000,
                output_tokens: 45_000,
                estimated_cost_micro_usd: 420_000,
            },
            SeedEvent {
                command: "image",
                input_tokens: 0,
                output_tokens: 0,
                estimated_cost_micro_usd: 0,
            },
            SeedEvent {
                command: "video",
                input_tokens: 0,
                output_tokens: 0,
                estimated_cost_micro_usd: 0,
            },
            SeedEvent {
                command: "tts",
                input_tokens: 0,
                output_tokens: 0,
                estimated_cost_micro_usd: 0,
            },
            SeedEvent {
                command: "stt",
                input_tokens: 3_200,
                output_tokens: 600,
                estimated_cost_micro_usd: 0,
            },
        ],
    );

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--session-db",
            session_db.to_str().unwrap(),
            "--session-id",
            session_id,
            "--local-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"breakdown\":"))
        .stdout(predicate::str::contains("\"text\":{\"request_count\":1"))
        .stdout(predicate::str::contains("\"image\":{\"request_count\":1"))
        .stdout(predicate::str::contains("\"video\":{\"request_count\":1"))
        .stdout(predicate::str::contains("\"audio\":{\"request_count\":2"));
}

#[test]
fn usage_human_output_formats_tokens_with_compact_units() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file, "http://127.0.0.1:9/v1");

    let session_id = "sess_usage_compact";
    let session_db = temp.path().join("session.db");

    seed_session_db(
        &session_db,
        session_id,
        &[SeedEvent {
            command: "chat",
            input_tokens: 124_837,
            output_tokens: 45_291,
            estimated_cost_micro_usd: 420_000,
        }],
    );

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--session-db",
            session_db.to_str().unwrap(),
            "--session-id",
            session_id,
            "--local-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("125K"))
        .stdout(predicate::str::contains("45.3K"))
        .stdout(predicate::str::contains("170K"));
}

#[test]
fn usage_human_output_formats_millions_with_compact_units() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file, "http://127.0.0.1:9/v1");

    let session_id = "sess_usage_millions";
    let session_db = temp.path().join("session.db");

    seed_session_db(
        &session_db,
        session_id,
        &[SeedEvent {
            command: "chat",
            input_tokens: 2_800_000,
            output_tokens: 1_200_000,
            estimated_cost_micro_usd: 4_200_000,
        }],
    );

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "usage",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--session-db",
            session_db.to_str().unwrap(),
            "--session-id",
            session_id,
            "--local-only",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2.80M"))
        .stdout(predicate::str::contains("1.20M"))
        .stdout(predicate::str::contains("4.00M"));
}

fn seed_session_db(path: &std::path::Path, session_id: &str, events: &[SeedEvent]) {
    let connection = Connection::open(path).unwrap();
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
COMMIT;
"#,
        )
        .unwrap();

    let mut request_count = 0_i64;
    let mut input_tokens = 0_i64;
    let mut output_tokens = 0_i64;
    let mut estimated_cost_micro_usd = 0_i64;
    for event in events {
        request_count += 1;
        input_tokens += event.input_tokens as i64;
        output_tokens += event.output_tokens as i64;
        estimated_cost_micro_usd += event.estimated_cost_micro_usd;
    }

    connection
        .execute(
            "INSERT INTO sessions(
                session_id, started_at, last_activity_at, provider, active_model, request_count,
                input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                reasoning_tokens, estimated_cost_micro_usd, context_window_tokens,
                compression_count, metadata_json
            ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, 0, ?9, ?10, 0, NULL)",
            params![
                session_id,
                "2026-05-20T10:00:00Z",
                "2026-05-20T10:47:12Z",
                "xai-oauth",
                "grok-4.3",
                request_count,
                input_tokens,
                output_tokens,
                estimated_cost_micro_usd,
                1_000_000_i64,
            ],
        )
        .unwrap();

    for (index, event) in events.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO session_events(
                    event_id, session_id, command, provider, model, started_at, completed_at, duration_ms,
                    input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                    reasoning_tokens, estimated_cost_micro_usd, context_window_tokens, request_id, metadata_json
                ) VALUES(?1, ?2, ?3, 'xai-oauth', 'grok-4.3', ?4, ?5, 0, ?6, ?7, 0, 0, 0, ?8, ?9, NULL, NULL)",
                params![
                    format!("evt_{index}"),
                    session_id,
                    event.command,
                    "2026-05-20T10:00:00Z",
                    "2026-05-20T10:00:01Z",
                    event.input_tokens as i64,
                    event.output_tokens as i64,
                    event.estimated_cost_micro_usd,
                    1_000_000_i64,
                ],
            )
            .unwrap();
    }
}

struct SeedEvent {
    command: &'static str,
    input_tokens: u64,
    output_tokens: u64,
    estimated_cost_micro_usd: i64,
}

fn write_auth_state(path: &std::path::Path, base_url: &str) {
    fs::write(
        path,
        format!(
            r#"{{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "{base_url}",
  "tokens": {{
    "access_token": "sample-access-token",
    "refresh_token": "sample-refresh-token",
    "id_token": null,
    "expires_in": 3600,
    "expires_at": "2099-01-01T00:00:00Z",
    "token_type": "Bearer"
  }},
  "discovery": {{
    "authorization_endpoint": "https://auth.x.ai/oauth2/authorize",
    "token_endpoint": "https://auth.x.ai/oauth2/token"
  }},
  "redirect_uri": "http://127.0.0.1:56121/callback",
  "last_refresh": "2026-05-19T17:00:00Z",
  "last_auth_error": null,
  "metadata": {{}}
}}"#
        ),
    )
    .unwrap();
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let size = stream.read(&mut buffer).unwrap();
        if size == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..size]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    String::from_utf8_lossy(&request).to_string()
}

fn write_json_response(stream: &mut std::net::TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
