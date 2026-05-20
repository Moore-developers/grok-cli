use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

#[test]
fn task_chat_rejects_empty_prompt() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["chat", "--json", "--prompt", "   "])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("prompt must not be empty"));
}

#[test]
fn task_chat_returns_non_stream_text_response() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/responses"));
        assert!(request.contains("\"stream\":false"));
        assert!(request.contains("\"store\":false"));
        assert!(request.contains("\"tool_choice\":\"auto\""));
        assert!(request.contains("\"parallel_tool_calls\":true"));
        assert!(request.contains("\"instructions\":\"You are helpful\""));
        assert!(request.contains("\"tools\":[{\"type\":\"web_search\"}]"));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"hello from Grok"}]}]}"#,
        );
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
            "--system",
            "You are helpful",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"protocol\":\"codex_responses\""))
        .stdout(predicate::str::contains(
            "\"output_text\":\"hello from Grok\"",
        ))
        .stdout(predicate::str::contains("\"finish_reason\":\"stop\""));

    server.join().unwrap();
}

#[test]
fn task_chat_accepts_prompt_as_positional_argument() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"content\":\"Say hello\""));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"hello positional"}]}]}"#,
        );
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Say hello",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"output_text\":\"hello positional\"",
        ));

    server.join().unwrap();
}

#[test]
fn task_chat_can_disable_default_web_search() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":false"));
        assert!(!request.contains("\"type\":\"web_search\""));
        assert!(!request.contains("\"type\":\"x_search\""));
        assert!(!request.contains("\"tool_choice\":\"auto\""));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"plain chat"}]}]}"#,
        );
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
            "--no-web-search",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"output_text\":\"plain chat\""));

    server.join().unwrap();
}

#[test]
fn task_chat_can_enable_combined_x_search_mode() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"type\":\"web_search\""));
        assert!(request.contains("\"type\":\"x_search\""));
        assert!(request.contains("\"tool_choice\":\"auto\""));
        assert!(request.contains("\"parallel_tool_calls\":true"));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"hybrid search chat"}]}]}"#,
        );
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
            "--with-x-search",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"output_text\":\"hybrid search chat\"",
        ));

    server.join().unwrap();
}

#[test]
fn task_chat_returns_tool_calls_in_non_stream_mode() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"output":[{"type":"function_call","call_id":"call_abc123","name":"terminal","arguments":"{\"command\":\"ls\"}"}]}"#,
        );
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "List files",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"finish_reason\":\"tool_calls\""))
        .stdout(predicate::str::contains("\"name\":\"terminal\""))
        .stdout(predicate::str::contains("\"id\":\"call_abc123\""));

    server.join().unwrap();
}

#[test]
fn task_chat_retries_after_transient_server_failure() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_server = Arc::clone(&attempts);

    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = read_request(&mut stream);
            let current = attempts_for_server.fetch_add(1, Ordering::SeqCst);
            if current == 0 {
                write_json_response(
                    &mut stream,
                    "500 Internal Server Error",
                    r#"{"error":"temporary upstream error"}"#,
                );
            } else {
                write_json_response(
                    &mut stream,
                    "200 OK",
                    r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"retried ok"}]}]}"#,
                );
            }
        }
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Retry test",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"output_text\":\"retried ok\""));

    server.join().unwrap();
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn task_chat_stream_emits_formatted_text() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":true"));
        assert!(request.contains("\"tool_choice\":\"auto\""));
        let body = concat!(
            "event: response.output_text.delta\r\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hel\"}\r\n",
            "\r\n",
            "event: response.output_text.done\r\n",
            "data: {\"type\":\"response.output_text.done\",\"text\":\"hello\"}\r\n",
            "\r\n",
            "event: response.completed\r\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"hello\"}]}]}}\r\n",
            "\r\n"
        );
        write_sse_response(&mut stream, body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Say hello",
            "--stream",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("hel"))
        .stdout(predicate::str::contains("event: ").not());

    server.join().unwrap();
}

#[test]
fn task_chat_streams_by_default_in_human_mode() {
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
            "event: response.created\r\n",
            "data: {\"type\":\"response.created\",\"response\":{\"status\":\"in_progress\"}}\r\n",
            "\r\n",
            "event: response.output_text.delta\r\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hi\"}\r\n",
            "\r\n",
            "event: response.completed\r\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"usage\":{\"input_tokens\":10,\"output_tokens\":2},\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"hi\"}]}]}}\r\n",
            "\r\n"
        );
        write_sse_response(&mut stream, body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["chat", "--auth-file", auth_file.to_str().unwrap(), "Say hi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hi"))
        .stdout(predicate::str::contains("Thinking...").not())
        .stdout(predicate::str::contains("event: ").not())
        .stderr(predicate::str::contains("Thinking...").not());

    server.join().unwrap();
}

#[test]
fn task_chat_raw_stream_emits_sse_events() {
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
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"json\"}\r\n",
            "\r\n",
            "event: response.completed\r\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"usage\":{\"input_tokens\":10,\"output_tokens\":2},\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"json\"}]}]}}\r\n",
            "\r\n"
        );
        write_sse_response(&mut stream, body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--raw-stream",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Say hi",
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
fn task_chat_no_stream_forces_single_response_in_human_mode() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"stream\":false"));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"output":[{"type":"message","content":[{"type":"output_text","text":"plain final text"}]}]}"#,
        );
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "chat",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--no-stream",
            "Say hi",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("plain final text"))
        .stdout(predicate::str::contains("Model:"))
        .stdout(predicate::str::contains("Finish: stop"))
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

fn write_sse_response(stream: &mut std::net::TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
