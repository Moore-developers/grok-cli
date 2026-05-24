use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn auth_logout_removes_state_file() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    fs::write(&auth_file, "{}").unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "logout",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"removed\":true"));

    assert!(!auth_file.exists());
}

#[test]
fn auth_login_manual_paste_outputs_authorize_url_and_persists_pending_oauth_state() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--manual-paste",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"login\""))
        .stdout(predicate::str::contains("\"saved\":true"))
        .stdout(predicate::str::contains(
            "\"authorize_url\":\"https://auth.x.ai/oauth2/authorize",
        ))
        .stdout(predicate::str::contains("\"pkce_method\":\"S256\""))
        .stdout(predicate::str::contains("plan=generic"))
        .stdout(predicate::str::contains("referrer=hermes-agent"));

    let raw = fs::read_to_string(&auth_file).unwrap();
    assert!(raw.contains("\"pending_oauth\""));
    assert!(raw.contains("\"code_verifier\""));
    assert!(raw.contains("\"code_challenge\""));
}

#[test]
fn auth_exchange_code_rejects_state_mismatch() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--manual-paste",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "exchange-code",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--code",
            "dummy-code",
            "--state",
            "wrong-state",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"code\":\"auth_state_mismatch\""));
}

#[test]
fn auth_exchange_code_accepts_manual_paste_without_state() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--manual-paste",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "exchange-code",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--code",
            "dummy-code-from-browser",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\"code\":\"auth_token_exchange_failed\"",
        ));
}

#[test]
fn auth_exchange_code_accepts_callback_url_without_explicit_state_flag() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--manual-paste",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "exchange-code",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--code",
            "http://127.0.0.1:56121/callback?code=dummy-code&state=wrong-state",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"code\":\"auth_state_mismatch\""));
}

#[test]
fn auth_refresh_requires_saved_refresh_token() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    fs::write(
        &auth_file,
        r#"{
  "version": 1,
  "provider": "xai-oauth",
  "auth_mode": "oauth_pkce",
  "base_url": "https://api.x.ai/v1",
  "tokens": {
    "access_token": "sample-access-token",
    "refresh_token": null,
    "id_token": null,
    "expires_in": 3600,
    "token_type": "Bearer"
  },
  "discovery": {
    "authorization_endpoint": "https://auth.x.ai/oauth2/authorize",
    "token_endpoint": "https://auth.x.ai/oauth2/token"
  },
  "redirect_uri": "http://127.0.0.1:56121/callback",
  "last_refresh": null,
  "last_auth_error": null,
  "metadata": {}
}"#,
    )
    .unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "refresh",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("\"code\":\"auth_missing\""));
}

#[test]
fn auth_login_times_out_waiting_for_callback() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--no-browser",
            "--timeout",
            "1",
            "--port",
            "56129",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\"code\":\"auth_callback_timeout\"",
        ));
}

#[test]
fn auth_login_falls_back_to_dynamic_callback_port_when_requested_port_is_busy() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let occupied = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let occupied_port = occupied.local_addr().unwrap().port();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--no-browser",
            "--timeout",
            "1",
            "--port",
            &occupied_port.to_string(),
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\"code\":\"auth_callback_timeout\"",
        ));

    let raw = fs::read_to_string(&auth_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let redirect_uri = json["redirect_uri"].as_str().unwrap();
    assert!(redirect_uri.starts_with("http://127.0.0.1:"));
    assert!(redirect_uri.ends_with("/callback"));
    assert!(!redirect_uri.contains(&format!(":{occupied_port}/")));
    assert_eq!(
        json["metadata"]["pending_oauth"]["no_browser"].as_bool(),
        Some(true)
    );
}

#[test]
fn auth_login_loopback_callback_state_mismatch_is_reported() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let port = reserve_free_port();

    let bin_path = std::env::current_dir()
        .unwrap()
        .join("target/debug/grok-cli");
    let mut child = ProcessCommand::new(bin_path)
        .args([
            "login",
            "--json",
            "--no-browser",
            "--timeout",
            "5",
            "--port",
            &port.to_string(),
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let pending_state = wait_for_pending_state_or_child_exit(&auth_file, &mut child, 100)
        .expect("login child did not persist pending OAuth state before exiting");
    assert!(!pending_state.is_empty());

    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .write_all(
            b"GET /callback?code=dummy-code&state=wrong-state HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        )
        .unwrap();
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"code\":\"auth_state_mismatch\""));
    assert!(!stdout.contains(&pending_state));
}

#[test]
fn auth_login_records_last_auth_error_on_timeout() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "login",
            "--json",
            "--no-browser",
            "--timeout",
            "1",
            "--port",
            "56131",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\"code\":\"auth_callback_timeout\"",
        ));

    let raw = fs::read_to_string(&auth_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        json["last_auth_error"]["code"].as_str(),
        Some("auth_callback_timeout")
    );
    assert_eq!(
        json["last_auth_error"]["reason"].as_str(),
        Some("callback_runtime_failure")
    );
}

fn wait_for_pending_state(path: &std::path::Path, attempts: usize) -> Option<String> {
    for _ in 0..attempts {
        if let Ok(raw) = fs::read_to_string(path)
            && let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw)
            && let Some(state) = json["metadata"]["pending_oauth"]["state"].as_str()
            && !state.is_empty()
        {
            return Some(state.to_string());
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

fn wait_for_pending_state_or_child_exit(
    path: &std::path::Path,
    child: &mut Child,
    attempts: usize,
) -> Option<String> {
    for _ in 0..attempts {
        if let Some(state) = wait_for_pending_state(path, 1) {
            return Some(state);
        }
        if matches!(child.try_wait(), Ok(Some(_))) {
            return None;
        }
    }
    None
}

fn reserve_free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}
