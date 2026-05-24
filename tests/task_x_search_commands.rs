use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn task_x_search_rejects_empty_query() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["search", "--json", "--query", "   "])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("query must not be empty"));
}

#[test]
fn task_x_search_rejects_too_many_allowed_handles() {
    let mut cmd = Command::cargo_bin("grok-cli").unwrap();
    cmd.args(["search", "--json", "--query", "xAI"]);
    for index in 0..11 {
        cmd.args(["--allowed-x-handle", &format!("handle{index}")]);
    }
    cmd.assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains(
            "--allowed-x-handle supports at most 10 values",
        ));
}

#[test]
fn task_x_search_returns_structured_success_from_stubbed_upstream() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"Hermes + Grok update","annotations":[{"type":"url_citation","url":"https://x.com/example/status/1","title":"1"}]}]}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--query",
            "Hermes Grok updates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains(
            "\"credential_source\":\"xai-oauth\"",
        ))
        .stdout(predicate::str::contains(
            "\"answer\":\"Hermes + Grok update\"",
        ))
        .stdout(predicate::str::contains("https://x.com/example/status/1"));

    server.join().unwrap();
}

#[test]
fn task_x_search_accepts_query_as_positional_argument() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"content\":\"Hermes Grok updates\""));
        let body = r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"positional search"}]}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Hermes Grok updates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"answer\":\"positional search\""));

    server.join().unwrap();
}

#[test]
fn task_x_search_maps_forbidden_to_tier_denied() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{"error":"forbidden","error_description":"tier access denied"}"#;
        write_response(&mut stream, "403 Forbidden", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--query",
            "Hermes Grok updates",
        ])
        .assert()
        .code(4)
        .stdout(predicate::str::contains(
            "\"code\":\"xai_oauth_tier_denied\"",
        ))
        .stdout(predicate::str::contains("\"entitlement_denied\":true"));

    server.join().unwrap();
}

#[test]
fn task_x_search_maps_bad_credentials_forbidden_to_auth_expired() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{"error":"forbidden","error_description":"The OAuth2 access token could not be validated. [WKE=unauthenticated:bad-credentials]"}"#;
        write_response(&mut stream, "403 Forbidden", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--query",
            "Hermes Grok updates",
        ])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("\"code\":\"auth_expired\""))
        .stdout(predicate::str::contains("\"entitlement_denied\":false"))
        .stdout(predicate::str::contains("bad-credentials"));

    server.join().unwrap();
}

#[test]
fn task_x_search_reports_request_failed_when_answer_is_missing() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{"output":[{"type":"reasoning","summary":[]}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--query",
            "Hermes Grok updates",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"code\":\"request_failed\""))
        .stdout(predicate::str::contains("did not include a message answer"));

    server.join().unwrap();
}

#[test]
fn task_x_search_streams_by_default_in_human_mode() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":true"));
        let body = concat!(
            "event: response.output_item.added\r\n",
            "data: {\"type\":\"response.output_item.added\",\"item\":{\"type\":\"custom_tool_call\",\"name\":\"x_semantic_search\"}}\r\n",
            "\r\n",
            "event: response.output_text.delta\r\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hermes\"}\r\n",
            "\r\n",
            "event: response.completed\r\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"usage\":{\"input_tokens\":12,\"output_tokens\":3},\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"Hermes update\"}]}]}}\r\n",
            "\r\n"
        );
        write_sse_response(&mut stream, body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Hermes Grok updates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hermes"))
        .stdout(predicate::str::contains("Searching X...").not())
        .stdout(predicate::str::contains("event: ").not())
        .stderr(predicate::str::contains("Searching X...").not());

    server.join().unwrap();
}

#[test]
fn task_x_search_stream_emits_formatted_text() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":true"));
        let body = concat!(
            "event: response.output_text.delta\r\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hermes\"}\r\n",
            "\r\n",
            "event: response.completed\r\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"usage\":{\"input_tokens\":12,\"output_tokens\":3},\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"Hermes update\"}]}]}}\r\n",
            "\r\n"
        );
        write_sse_response(&mut stream, body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--stream",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Hermes Grok updates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hermes"))
        .stdout(predicate::str::contains("event: ").not());

    server.join().unwrap();
}

#[test]
fn task_x_search_raw_stream_emits_sse_events() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":true"));
        let body = concat!(
            "event: response.output_text.delta\r\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hermes\"}\r\n",
            "\r\n",
            "event: response.completed\r\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"usage\":{\"input_tokens\":12,\"output_tokens\":3},\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"Hermes update\"}]}]}}\r\n",
            "\r\n"
        );
        write_sse_response(&mut stream, body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--raw-stream",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Hermes Grok updates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "event: response.output_text.delta",
        ))
        .stdout(predicate::str::contains("event: response.completed"));

    server.join().unwrap();
}

#[test]
fn task_x_search_no_stream_forces_single_response_in_human_mode() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":false"));
        let body = r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"Hermes summary","annotations":[{"type":"url_citation","url":"https://x.com/example/status/2","title":"1"}]}]}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "search",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--no-stream",
            "Hermes Grok updates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hermes summary"))
        .stdout(predicate::str::contains("Sources:"))
        .stdout(predicate::str::contains(
            "1. https://x.com/example/status/2",
        ))
        .stdout(predicate::str::contains("Model:"))
        .stdout(predicate::str::contains("Tool: x_search"))
        .stdout(predicate::str::contains("event: ").not());

    server.join().unwrap();
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
    let mut buffer = [0_u8; 2048];
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

fn write_response(stream: &mut std::net::TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn write_sse_response(stream: &mut std::net::TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
