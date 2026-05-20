use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;

#[test]
fn model_persists_default_model_and_json_reads_it() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file);

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "model",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--model",
            "grok-4.3",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"model\":\"grok-4.3\""))
        .stdout(predicate::str::contains("\"chat\":\"grok-4.3\""))
        .stdout(predicate::str::contains("\"search\":\"grok-4.3\""));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "model",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"selected_model\":\"grok-4.3\""))
        .stdout(predicate::str::contains("\"text\":\"grok-4.3\""))
        .stdout(predicate::str::contains("\"chat\":\"grok-4.3\""))
        .stdout(predicate::str::contains("\"search\":\"grok-4.3\""));
}

#[test]
fn model_rejects_non_text_task_keys() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file);

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "model",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--command",
            "tts",
            "--model",
            "grok-tts",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains(
            "--command must be one of: chat, search",
        ));
}

#[test]
fn model_human_output_lists_shared_model_catalog() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file);

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["model", "--auth-file", auth_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Grok Models"))
        .stdout(predicate::str::contains("Selected"))
        .stdout(predicate::str::contains("grok-4.3"))
        .stdout(predicate::str::contains("grok-4.20-reasoning"))
        .stdout(predicate::str::contains("grok-4.20-0309-reasoning"))
        .stdout(predicate::str::contains("exit"))
        .stdout(predicate::str::contains("show").not())
        .stdout(predicate::str::contains("set").not());
}

#[test]
fn mode_alias_matches_model_command() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    write_auth_state(&auth_file);

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["mode", "--json", "--auth-file", auth_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"model\""))
        .stdout(predicate::str::contains("\"catalog\":[\"grok-4.3\""));
}

fn write_auth_state(path: &std::path::Path) {
    fs::write(
        path,
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
}
