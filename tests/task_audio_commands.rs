use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn task_tts_rejects_empty_text() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["tts", "--json", "--text", "   "])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("text must not be empty"));
}

#[test]
fn task_tts_writes_audio_file_from_stubbed_upstream() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let output_file = temp.path().join("voice.mp3");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/tts"));
        assert!(request.contains("\"text\":\"Hello from Grok\""));
        assert!(request.contains("\"voice_id\":\"eve\""));
        assert!(request.contains("\"language\":\"en\""));
        write_binary_response(&mut stream, "200 OK", b"FAKEAUDIO");
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "tts",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--text",
            "Hello from Grok",
            "--output",
            output_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains(output_file.to_str().unwrap()))
        .stdout(predicate::str::contains("\"media_tag\":\"MEDIA:"));

    assert_eq!(fs::read(&output_file).unwrap(), b"FAKEAUDIO");
    server.join().unwrap();
}

#[test]
fn task_tts_sends_output_format_and_advanced_options() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let output_file = temp.path().join("voice.mp3");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/tts"));
        assert!(request.contains("\"language\":\"auto\""));
        assert!(request.contains(
            "\"output_format\":{\"bit_rate\":128000,\"codec\":\"mp3\",\"sample_rate\":24000}"
        ));
        assert!(request.contains("\"optimize_streaming_latency\":\"auto\""));
        assert!(request.contains("\"text_normalization\":\"off\""));
        write_binary_response(&mut stream, "200 OK", b"FAKEAUDIO");
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "tts",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--text",
            "Hello from Grok",
            "--language",
            "auto",
            "--output",
            output_file.to_str().unwrap(),
            "--output-format",
            "mp3",
            "--sample-rate",
            "24000",
            "--bit-rate",
            "128000",
            "--optimize-streaming-latency",
            "auto",
            "--text-normalization",
            "off",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"output_format\""));

    assert_eq!(fs::read(&output_file).unwrap(), b"FAKEAUDIO");
    server.join().unwrap();
}

#[test]
fn task_tts_accepts_text_as_positional_argument() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let output_file = temp.path().join("voice.mp3");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("\"text\":\"Hello positional\""));
        write_binary_response(&mut stream, "200 OK", b"FAKEAUDIO");
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "tts",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
            "Hello positional",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"));

    server.join().unwrap();
}

#[test]
fn task_tts_lists_voices_from_stubbed_upstream() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("GET /v1/tts/voices"));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"voices":[{"voice_id":"eve","name":"Eve","type":"official"},{"voice_id":"custom-1","name":"Custom","type":"custom"}]}"#,
        );
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "tts",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--list-voices",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains("\"voice_id\":\"eve\""))
        .stdout(predicate::str::contains("\"voice_id\":\"custom-1\""));

    server.join().unwrap();
}

#[test]
fn task_stt_rejects_missing_file() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["stt", "--json", "--file", "/tmp/does-not-exist.wav"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("file does not exist"));
}

#[test]
fn task_stt_stream_rejects_missing_file() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["stt-stream", "--json"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("file must not be empty"));
}

#[test]
fn task_stt_stream_help_lists_streaming_parameters() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["stt-stream", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--interim-results"))
        .stdout(predicate::str::contains("--endpointing"))
        .stdout(predicate::str::contains("--encoding"))
        .stdout(predicate::str::contains("--sample-rate"))
        .stdout(predicate::str::contains("--keyterm"));
}

#[test]
fn task_stt_rejects_file_and_url_together() {
    let temp = tempdir().unwrap();
    let input_file = temp.path().join("sample.wav");
    fs::write(&input_file, b"WAVE").unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "stt",
            "--json",
            "--file",
            input_file.to_str().unwrap(),
            "--url",
            "https://example.com/audio.wav",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("\"code\":\"invalid_args\""))
        .stdout(predicate::str::contains("--url cannot be combined"));
}

#[test]
fn task_stt_returns_transcript_from_stubbed_upstream() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let input_file = temp.path().join("sample.wav");
    fs::write(&input_file, b"WAVE").unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/stt"));
        assert!(request.contains("multipart/form-data"));
        assert!(request.contains("name=\"format\""));
        assert!(!request.contains("name=\"model\""));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"text":"hello transcript","language":"en","duration":1.5,"words":[{"word":"hello","start":0.0,"end":0.5}],"channels":[{"channel":0,"text":"hello transcript"}]}"#,
        );
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "stt",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--file",
            input_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\":true"))
        .stdout(predicate::str::contains(
            "\"transcript\":\"hello transcript\"",
        ))
        .stdout(predicate::str::contains("\"language\":\"en\""))
        .stdout(predicate::str::contains("\"duration\":1.5"))
        .stdout(predicate::str::contains(
            "\"words\":[{\"end\":0.5,\"start\":0.0,\"word\":\"hello\"}]",
        ))
        .stdout(predicate::str::contains(
            "\"channels\":[{\"channel\":0,\"text\":\"hello transcript\"}]",
        ));

    server.join().unwrap();
}

#[test]
fn task_stt_sends_url_and_advanced_options_to_stubbed_upstream() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/stt"));
        assert!(request.contains("multipart/form-data"));
        assert!(request.contains("name=\"url\""));
        assert!(request.contains("https://example.com/audio.wav"));
        assert!(request.contains("name=\"format\""));
        assert!(request.contains("\r\nfalse\r\n"));
        assert!(request.contains("name=\"language\""));
        assert!(request.contains("\r\nauto\r\n"));
        assert!(request.contains("name=\"audio_format\""));
        assert!(request.contains("pcm_s16le"));
        assert!(request.contains("name=\"sample_rate\""));
        assert!(request.contains("\r\n16000\r\n"));
        assert!(request.contains("name=\"multichannel\""));
        assert!(request.contains("name=\"channels\""));
        assert!(request.contains("\r\n0,1\r\n"));
        assert!(request.contains("name=\"diarize\""));
        assert!(request.contains("name=\"keyterm\""));
        assert!(request.contains("\r\nGrok\r\n"));
        assert!(request.contains("\r\nxAI\r\n"));
        assert!(request.contains("name=\"filler_words\""));
        assert!(!request.contains("name=\"file\""));
        write_json_response(&mut stream, "200 OK", r#"{"text":"url transcript"}"#);
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "stt",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            "--url",
            "https://example.com/audio.wav",
            "--format",
            "false",
            "--language",
            "auto",
            "--audio-format",
            "pcm_s16le",
            "--sample-rate",
            "16000",
            "--multichannel",
            "--channels",
            "0,1",
            "--diarize",
            "--keyterm",
            "Grok",
            "--keyterm",
            "xAI",
            "--filler-words",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"transcript\":\"url transcript\"",
        ));

    server.join().unwrap();
}

#[test]
fn task_stt_accepts_file_as_positional_argument() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("auth.json");
    let input_file = temp.path().join("sample.wav");
    fs::write(&input_file, b"WAVE").unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    write_auth_state(&auth_file, &format!("http://127.0.0.1:{port}/v1"));

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("POST /v1/stt"));
        write_json_response(
            &mut stream,
            "200 OK",
            r#"{"text":"hello positional transcript"}"#,
        );
    });

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "stt",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
            input_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello positional transcript"));

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
            let header_end = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|index| index + 4)
                .unwrap();
            let header_text = String::from_utf8_lossy(&request[..header_end]).to_string();
            let content_length = header_text
                .lines()
                .find_map(|line| {
                    let lower = line.to_ascii_lowercase();
                    lower
                        .strip_prefix("content-length: ")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            let body_bytes = request.len().saturating_sub(header_end);
            if body_bytes >= content_length {
                break;
            }
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

fn write_binary_response(stream: &mut std::net::TcpStream, status: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: audio/mpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    stream.flush().unwrap();
}
