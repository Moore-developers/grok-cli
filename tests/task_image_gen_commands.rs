use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn task_image_gen_rejects_empty_prompt() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["image", "--json", "--prompt", "   "])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("prompt must not be empty"));
}

#[test]
fn task_image_gen_returns_remote_image_url_from_stubbed_upstream() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{"data":[{"url":"https://cdn.x.ai/generated-image.png"}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Draw a futuristic skyline",
            "--aspect-ratio",
            "16:9",
            "--resolution",
            "1k",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"model\":\"grok-imagine-image\""))
        .stdout(predicate::str::contains(
            "\"image\":\"https://cdn.x.ai/generated-image.png\"",
        ))
        .stdout(predicate::str::contains(
            "\"images\":[\"https://cdn.x.ai/generated-image.png\"]",
        ))
        .stdout(predicate::str::contains("\"aspect_ratio\":\"16:9\""));

    server.join().unwrap();
}

#[test]
fn task_image_gen_sends_count_and_response_format() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/images/generations"));
        assert!(request.contains("\"n\":2"));
        assert!(request.contains("\"response_format\":\"url\""));
        let body = r#"{"data":[{"url":"https://cdn.x.ai/image-1.png"},{"url":"https://cdn.x.ai/image-2.png"}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Draw a futuristic skyline",
            "--count",
            "2",
            "--response-format",
            "url",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"image\":\"https://cdn.x.ai/image-1.png\"",
        ))
        .stdout(predicate::str::contains(
            "\"images\":[\"https://cdn.x.ai/image-1.png\",\"https://cdn.x.ai/image-2.png\"]",
        ));

    server.join().unwrap();
}

#[test]
fn task_image_gen_maps_payment_required_to_structured_billing_stop() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/images/generations"));
        let body = r#"{"error":"payment_required","error_description":"Billing required: insufficient credits"}"#;
        write_response(&mut stream, "402 Payment Required", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Draw a futuristic skyline",
        ])
        .assert()
        .code(4)
        .stdout(predicate::str::contains("\"code\":\"billing_required\""))
        .stdout(predicate::str::contains(
            "\"category\":\"billing_required\"",
        ))
        .stdout(predicate::str::contains(
            "\"recovery_action\":\"stop_billing\"",
        ))
        .stdout(predicate::str::contains("\"billing_required\":true"))
        .stdout(predicate::str::contains("\"retryable\":false"));

    server.join().unwrap();
}

#[test]
fn task_image_gen_accepts_prompt_as_positional_argument() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"prompt\":\"Draw a futuristic skyline\""));
        let body = r#"{"data":[{"url":"https://cdn.x.ai/positional-image.png"}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "Draw a futuristic skyline",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "https://cdn.x.ai/positional-image.png",
        ));

    server.join().unwrap();
}

#[test]
fn task_image_gen_writes_output_file_when_requested() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let output_file = temp.path().join("artifacts").join("image.bin");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"response_format\":\"b64_json\""));
        let body = r#"{"data":[{"b64_json":"aGVsbG8="}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Draw a futuristic skyline",
            "--output-file",
            output_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(output_file.to_str().unwrap()));

    assert_eq!(fs::read(&output_file).unwrap(), b"hello");
    server.join().unwrap();
}

#[test]
fn task_image_gen_rejects_output_file_with_multiple_images() {
    let temp = tempdir().unwrap();
    let output_file = temp.path().join("image.png");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--prompt",
            "Draw a futuristic skyline",
            "--count",
            "2",
            "--output-file",
            output_file.to_str().unwrap(),
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains(
            "--output-file can only be used with --count 1",
        ));
}

#[test]
fn task_image_gen_writes_output_dir_when_requested() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let output_dir = temp.path().join("artifacts");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"n\":2"));
        assert!(request.contains("\"response_format\":\"b64_json\""));
        let body = r#"{"data":[{"b64_json":"aGVsbG8="},{"b64_json":"d29ybGQ="}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Draw a futuristic skyline",
            "--count",
            "2",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("image-001.png"))
        .stdout(predicate::str::contains("image-002.png"))
        .stdout(predicate::str::contains("\"images\""));

    assert_eq!(
        fs::read(output_dir.join("image-001.png")).unwrap(),
        b"hello"
    );
    assert_eq!(
        fs::read(output_dir.join("image-002.png")).unwrap(),
        b"world"
    );
    server.join().unwrap();
}

#[test]
fn task_image_edit_rejects_too_many_images() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image-edit",
            "--json",
            "--prompt",
            "Make it cinematic",
            "--image",
            "https://cdn.x.ai/1.png",
            "--image",
            "https://cdn.x.ai/2.png",
            "--image",
            "https://cdn.x.ai/3.png",
            "--image",
            "https://cdn.x.ai/4.png",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains(
            "--image supports at most 3 values",
        ));
}

#[test]
fn task_image_edit_sends_single_image_request() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/images/edits"));
        assert!(request.contains("\"prompt\":\"Make it cinematic\""));
        assert!(request.contains(
            "\"image\":{\"type\":\"image_url\",\"url\":\"https://cdn.x.ai/source.png\"}"
        ));
        assert!(!request.contains("\"images\""));
        let body = r#"{"data":[{"url":"https://cdn.x.ai/edited-image.png"}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image-edit",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Make it cinematic",
            "--image",
            "https://cdn.x.ai/source.png",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"image\":\"https://cdn.x.ai/edited-image.png\"",
        ))
        .stdout(predicate::str::contains(
            "\"images\":[\"https://cdn.x.ai/edited-image.png\"]",
        ));

    server.join().unwrap();
}

#[test]
fn task_image_edit_sends_multi_image_request() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/images/edits"));
        assert!(request.contains("\"images\":["));
        assert!(request.contains("https://cdn.x.ai/1.png"));
        assert!(request.contains("https://cdn.x.ai/2.png"));
        assert!(request.contains("https://cdn.x.ai/3.png"));
        assert!(request.contains("\"response_format\":\"b64_json\""));
        let body = r#"{"data":[{"b64_json":"aGVsbG8="}]}"#;
        write_response(&mut stream, "200 OK", body);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "image-edit",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--prompt",
            "Blend these references",
            "--image",
            "https://cdn.x.ai/1.png",
            "--image",
            "https://cdn.x.ai/2.png",
            "--image",
            "https://cdn.x.ai/3.png",
            "--response-format",
            "b64_json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"image\":\"data:image/png;base64,aGVsbG8=\"",
        ));

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
