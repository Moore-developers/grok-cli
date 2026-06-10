use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn package_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[test]
fn update_no_update_check_disables_passive_checks() {
    let temp = tempdir().unwrap();
    let config_file = temp.path().join("update.json");

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env("GROK_CLI_UPDATE_STATE_FILE", &config_file)
        .args(["update", "--no-update-check", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"update\""))
        .stdout(predicate::str::contains("\"auto_check_enabled\":false"));

    let saved = fs::read_to_string(&config_file).unwrap();
    assert!(saved.contains("\"auto_check_enabled\": false"));
}

#[test]
fn update_enable_update_check_reenables_passive_checks() {
    let temp = tempdir().unwrap();
    let config_file = temp.path().join("update.json");
    fs::write(&config_file, r#"{"version":1,"auto_check_enabled":false}"#).unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env("GROK_CLI_UPDATE_STATE_FILE", &config_file)
        .args(["update", "--enable-update-check", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"auto_check_enabled\":true"));

    let saved = fs::read_to_string(&config_file).unwrap();
    assert!(saved.contains("\"auto_check_enabled\": true"));
}

#[test]
fn update_check_reports_available_release() {
    let temp = tempdir().unwrap();
    let config_file = temp.path().join("update.json");
    let server = spawn_release_server(release_json("v99.0.0"));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env("GROK_CLI_UPDATE_STATE_FILE", &config_file)
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--check", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"current_version\""))
        .stdout(predicate::str::contains("\"latest_version\":\"99.0.0\""))
        .stdout(predicate::str::contains("\"latest_tag\":\"v99.0.0\""))
        .stdout(predicate::str::contains("\"update_available\":true"))
        .stdout(predicate::str::contains("\"install_strategy\""));

    server.join();
    let saved = fs::read_to_string(&config_file).unwrap();
    assert!(saved.contains("\"latest_version\": \"99.0.0\""));
}

#[test]
fn update_check_reports_current_release() {
    let temp = tempdir().unwrap();
    let server = spawn_release_server(release_json(&format!("v{}", package_version())));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("grok-cli is up to date"))
        .stdout(predicate::str::contains(package_version()));

    server.join();
}

#[test]
fn update_default_does_not_install_when_current() {
    let temp = tempdir().unwrap();
    let server = spawn_release_server(release_json(&format!("v{}", package_version())));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"installed\":false"))
        .stdout(predicate::str::contains("\"update_available\":false"))
        .stdout(predicate::str::contains("already up to date"));

    server.join();
}

#[test]
fn update_default_human_output_does_not_install_when_current() {
    let temp = tempdir().unwrap();
    let server = spawn_release_server(release_json(&format!("v{}", package_version())));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update"])
        .assert()
        .success()
        .stdout(predicate::str::contains("grok-cli is already up to date"))
        .stdout(predicate::str::contains(package_version()));

    server.join();
}

#[test]
fn update_check_human_reports_available_release() {
    let temp = tempdir().unwrap();
    let server = spawn_release_server(release_json("v99.0.0"));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("grok-cli v99.0.0 is available"))
        .stdout(predicate::str::contains("run: grok-cli update"));

    server.join();
}

#[test]
fn update_check_maps_invalid_release_json_to_json_error() {
    let temp = tempdir().unwrap();
    let server = spawn_raw_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 10\r\nConnection: close\r\n\r\n{not-json}",
    );

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--check", "--json"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\"code\":\"response_decode_failed\"",
        ))
        .stdout(predicate::str::contains(
            "failed to decode latest release response",
        ));

    server.join();
}

#[test]
fn update_check_maps_invalid_release_tag_to_json_error() {
    let temp = tempdir().unwrap();
    let server = spawn_release_server(release_json("release-current"));

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--check", "--json"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "\"code\":\"response_decode_failed\"",
        ))
        .stdout(predicate::str::contains("release tag is not a version"));

    server.join();
}

#[test]
fn update_check_maps_release_http_failure_to_json_error() {
    let temp = tempdir().unwrap();
    let server = spawn_raw_server(
        "HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nContent-Length: 11\r\nConnection: close\r\n\r\nunavailable",
    );

    Command::cargo_bin("grok-cli")
        .unwrap()
        .env(
            "GROK_CLI_UPDATE_STATE_FILE",
            temp.path().join("update.json"),
        )
        .env("GROK_CLI_UPDATE_RELEASE_URL", &server.url)
        .args(["update", "--check", "--json"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"command\":\"update\""))
        .stdout(predicate::str::contains("\"code\":\"request_failed\""))
        .stdout(predicate::str::contains("503 Service Unavailable"));

    server.join();
}

#[test]
fn update_rejects_conflicting_check_and_no_update_check() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["update", "--check", "--no-update-check"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn update_help_lists_check_and_passive_controls() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--check"))
        .stdout(predicate::str::contains("--no-update-check"))
        .stdout(predicate::str::contains("--enable-update-check"));
}

struct StubServer {
    url: String,
    handle: thread::JoinHandle<()>,
}

impl StubServer {
    fn join(self) {
        self.handle.join().unwrap();
    }
}

fn spawn_release_server(body: String) -> StubServer {
    spawn_raw_server(&format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    ))
}

fn spawn_raw_server(response: &str) -> StubServer {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let response = response.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
        assert!(request.contains("GET /release"));
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    StubServer {
        url: format!("http://127.0.0.1:{port}/release"),
        handle,
    }
}

fn release_json(tag: &str) -> String {
    format!(
        r#"{{
  "tag_name": "{tag}",
  "html_url": "https://github.com/Moore-developers/grok-cli/releases/tag/{tag}",
  "assets": [
    {{
      "name": "grok-cli-macos-aarch64-apple-darwin.tar.gz",
      "browser_download_url": "https://example.test/grok-cli-macos-aarch64-apple-darwin.tar.gz"
    }},
    {{
      "name": "grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256",
      "browser_download_url": "https://example.test/grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256"
    }},
    {{
      "name": "grok-cli-windows-x86_64-pc-windows-msvc.zip",
      "browser_download_url": "https://example.test/grok-cli-windows-x86_64-pc-windows-msvc.zip"
    }},
    {{
      "name": "grok-cli-windows-x86_64-pc-windows-msvc.zip.sha256",
      "browser_download_url": "https://example.test/grok-cli-windows-x86_64-pc-windows-msvc.zip.sha256"
    }}
  ]
}}"#
    )
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
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
