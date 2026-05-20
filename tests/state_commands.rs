use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn state_show_redacts_tokens() {
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
    "refresh_token": "sample-refresh-token",
    "id_token": "sample-id-token",
    "expires_in": 3600,
    "token_type": "Bearer"
  },
  "discovery": {},
  "redirect_uri": "http://127.0.0.1:56121/callback",
  "last_refresh": "2026-05-19T17:00:00Z",
  "last_auth_error": null,
  "metadata": {}
}"#,
    )
    .unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "state",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"state\""))
        .stdout(predicate::str::contains("\"exists\":true"))
        .stdout(predicate::str::contains("samp...oken"))
        .stdout(predicate::str::contains("sample-access-token").not());
}

#[test]
fn auth_status_reports_state_file_missing() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("missing.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "status",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"code\":\"state_file_missing\""));
}

#[test]
fn auth_status_human_output_uses_three_column_table() {
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
    "refresh_token": "sample-refresh-token",
    "id_token": null,
    "expires_in": 3600,
    "expires_at": "2099-01-01T00:00:00Z",
    "token_type": "Bearer"
  },
  "discovery": {
    "authorization_endpoint": "https://auth.x.ai/oauth2/authorize",
    "token_endpoint": "https://auth.x.ai/oauth2/token"
  },
  "redirect_uri": "http://127.0.0.1:56121/callback",
  "last_refresh": "2026-05-19T17:00:00Z",
  "last_auth_error": null,
  "metadata": {}
}"#,
    )
    .unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["status", "--auth-file", auth_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("OAuth Status"))
        .stdout(predicate::str::contains("Field"))
        .stdout(predicate::str::contains("Value"))
        .stdout(predicate::str::contains("Description"))
        .stdout(predicate::str::contains("Field                    | Value"))
        .stdout(predicate::str::contains("logged_in"))
        .stdout(predicate::str::contains(
            "Whether grok-cli can use the saved OAuth session.",
        ));
}
