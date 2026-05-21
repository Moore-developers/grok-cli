use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::path::Path;

fn package_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[test]
fn top_level_help_lists_all_primary_command_groups() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("chat"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("image"))
        .stdout(predicate::str::contains("image-edit"))
        .stdout(predicate::str::contains("video"))
        .stdout(predicate::str::contains("video-edit"))
        .stdout(predicate::str::contains("video-extend"))
        .stdout(predicate::str::contains("tts"))
        .stdout(predicate::str::contains("stt"))
        .stdout(predicate::str::contains("stt-stream"))
        .stdout(predicate::str::contains("state"))
        .stdout(predicate::str::contains("usage"))
        .stdout(predicate::str::contains("print-authorize-url").not())
        .stdout(predicate::str::contains("exchange-code").not());
}

#[test]
fn media_help_lists_direct_media_commands() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("image"))
        .stdout(predicate::str::contains("image-edit"))
        .stdout(predicate::str::contains("video"))
        .stdout(predicate::str::contains("video-edit"))
        .stdout(predicate::str::contains("video-extend"))
        .stdout(predicate::str::contains("tts"))
        .stdout(predicate::str::contains("stt"))
        .stdout(predicate::str::contains("stt-stream"));
}

#[test]
fn top_level_help_lists_model_command_group() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("model"));
}

#[test]
fn task_chat_help_lists_search_mode_flags() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["chat", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-web-search"))
        .stdout(predicate::str::contains("--with-x-search"));
}

#[test]
fn model_help_is_single_command_surface() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["model", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--model"))
        .stdout(predicate::str::contains("show").not())
        .stdout(predicate::str::contains("list").not())
        .stdout(predicate::str::contains("set").not());
}

#[test]
fn state_has_no_public_subcommands() {
    Command::cargo_bin("grok-cli")
        .unwrap()
        .args(["state", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: grok-cli state [OPTIONS]"))
        .stdout(predicate::str::contains("show").not())
        .stdout(predicate::str::contains("state path").not())
        .stdout(predicate::str::contains("validate").not());
}

#[test]
fn state_reports_invalid_json_as_state_error() {
    let temp = tempdir().unwrap();
    let auth_file = temp.path().join("broken.json");
    fs::write(&auth_file, "{not-json").unwrap();

    Command::cargo_bin("grok-cli")
        .unwrap()
        .args([
            "state",
            "--json",
            "--auth-file",
            auth_file.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"command\":\"state\""))
        .stdout(predicate::str::contains("\"code\":\"state_file_invalid\""))
        .stdout(predicate::str::contains("invalid json:"));
}

#[test]
fn bundled_skill_requires_command_surface_check() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let package_version = package_version();
    let release_tag = format!("v{package_version}");
    let install_command_fragment = format!("--tag {release_tag} --locked --force");
    let release_script_example =
        format!("scripts/package-local-macos-release.sh {release_tag} --upload");
    let skill = fs::read_to_string(root.join("skills/grok-cli/SKILL.md")).unwrap();
    let basic_ref =
        fs::read_to_string(root.join("skills/grok-cli/references/commands-basic.md")).unwrap();
    let install_ref =
        fs::read_to_string(root.join("skills/grok-cli/references/install-and-auth.md")).unwrap();
    let windows_release_workflow =
        fs::read_to_string(root.join(".github/workflows/windows-release.yml")).unwrap();
    let local_macos_release_script =
        fs::read_to_string(root.join("scripts/package-local-macos-release.sh")).unwrap();
    let release_doc = fs::read_to_string(root.join("docs/guides/release.md")).unwrap();
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    let readme_zh = fs::read_to_string(root.join("README.zh-CN.md")).unwrap();
    let advanced_ref =
        fs::read_to_string(root.join("skills/grok-cli/references/commands-advanced.md")).unwrap();
    let errors_ref = fs::read_to_string(root.join("skills/grok-cli/references/errors.md")).unwrap();
    let media_ref =
        fs::read_to_string(root.join("skills/grok-cli/references/commands-media.md")).unwrap();
    let skill_validation =
        fs::read_to_string(root.join("docs/project/skill-validation-cases.md")).unwrap();

    for reference in [
        "references/install-and-auth.md",
        "references/commands-basic.md",
        "references/commands-media.md",
        "references/commands-advanced.md",
        "references/errors.md",
        "references/outputs.md",
    ] {
        assert!(
            skill.contains(reference),
            "SKILL.md should point to {reference}"
        );
    }

    for command in ["image-edit", "video-edit", "video-extend", "stt-stream"] {
        assert!(
            skill.contains(command),
            "SKILL.md should verify the {command} command surface"
        );
        assert!(
            install_ref.contains(command),
            "install reference should verify the {command} command surface"
        );
    }

    assert!(install_ref.contains("grok-cli --help"));
    assert!(install_ref.contains("grok-cli status --json"));
    assert!(install_ref.contains("bad-credentials"));
    assert!(install_ref.contains("grok-cli refresh --json"));
    assert!(install_ref.contains("rustc --version"));
    assert!(install_ref.contains("Rust 1.88 or newer"));
    assert!(install_ref.contains("Rust 1.92.0"));
    assert!(install_ref.contains("rust-version = \"1.88\""));
    assert!(install_ref.contains(&install_command_fragment));
    assert!(install_ref.contains("grok-cli-macos-aarch64-apple-darwin.tar.gz"));
    assert!(install_ref.contains("grok-cli-windows-x86_64-pc-windows-msvc.zip"));
    assert!(install_ref.contains(".sha256"));
    assert!(windows_release_workflow.contains("windows-latest"));
    assert!(windows_release_workflow.contains("x86_64-pc-windows-msvc"));
    assert!(windows_release_workflow.contains("grok-cli-windows-x86_64-pc-windows-msvc.zip"));
    assert!(windows_release_workflow.contains("Get-FileHash"));
    assert!(
        windows_release_workflow.contains("grok-cli-windows-x86_64-pc-windows-msvc.zip.sha256")
    );
    assert!(local_macos_release_script.contains("aarch64-apple-darwin"));
    assert!(local_macos_release_script.contains("grok-cli-macos-${target}.tar.gz"));
    assert!(local_macos_release_script.contains("cargo build --release --locked"));
    assert!(local_macos_release_script.contains("gh release upload"));
    assert!(local_macos_release_script.contains("--clobber"));
    assert!(local_macos_release_script.contains("working tree has uncommitted changes"));
    assert!(release_doc.contains(&release_script_example));
    assert!(release_doc.contains("grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256"));
    assert!(!release_doc.contains("grok-cli-windows_x86_64"));
    assert!(readme.contains("grok-cli-macos-aarch64-apple-darwin.tar.gz"));
    assert!(readme.contains("grok-cli-windows-x86_64-pc-windows-msvc.zip"));
    assert!(readme_zh.contains("grok-cli-macos-aarch64-apple-darwin.tar.gz"));
    assert!(readme_zh.contains("grok-cli-windows-x86_64-pc-windows-msvc.zip"));
    assert!(skill.contains("What Users Can Do Through This Skill"));
    assert!(skill.contains("grok-cli status --json"));
    assert!(skill.contains("bad-credentials"));
    assert!(skill.contains("grok-cli refresh --json"));
    assert!(skill.contains("Do not present an empty or generic answer as a real X discussion summary"));
    assert!(skill.contains("Rust 1.88 or newer"));
    assert!(skill.contains("Rust 1.92.0"));
    assert!(skill.contains("rustc --version"));
    assert!(skill.contains("rust-version = \"1.88\""));
    assert!(skill.contains("Skill Test Prompts"));
    assert!(skill.contains("Common Parameter Cheat Sheet"));
    assert!(skill.contains("Reference Map"));
    assert!(skill.contains("video-extend --video <PATH>"));
    assert!(skill.contains("not supported"));
    assert!(skill.contains("--no-browser"));
    assert!(skill.contains("--manual-paste"));
    assert!(skill.contains("--port 8787"));
    assert!(skill.contains("--auth-file <PATH>"));
    assert!(skill.contains("--session-db <PATH>"));
    assert!(skill.contains("--session-id <ID>"));
    assert!(skill.contains("--allowed-domain example.com"));
    assert!(skill.contains("--allowed-x-handle xAI"));
    assert!(skill.contains("--stream"));
    assert!(skill.contains("--raw-stream"));
    assert!(skill.contains("--count 1-10"));
    assert!(skill.contains("--aspect-ratio 1:1"));
    assert!(skill.contains("--resolution 1k"));
    assert!(skill.contains("--output ./out.mp3"));
    assert!(skill.contains("--format true"));
    assert!(skill.contains("--sample-rate 16000"));
    assert!(media_ref.contains("video-extend"));
    assert!(media_ref.contains("--video-url"));
    assert!(media_ref.contains("--video <PATH>"));
    assert!(media_ref.contains("--reference-image <PATH>"));
    assert!(media_ref.contains("--optimize-streaming-latency"));
    assert!(media_ref.contains("--text-normalization"));
    assert!(media_ref.contains("--endpointing"));
    assert!(media_ref.contains("--encoding"));
    assert!(media_ref.contains("upstream internal error"));
    assert!(basic_ref.contains("--allowed-domain"));
    assert!(basic_ref.contains("--excluded-domain"));
    assert!(basic_ref.contains("--allowed-x-handle"));
    assert!(basic_ref.contains("--excluded-x-handle"));
    assert!(basic_ref.contains("--from-date"));
    assert!(basic_ref.contains("--to-date"));
    assert!(basic_ref.contains("State the exact query and date range used"));
    assert!(basic_ref.contains("empty `data.citations`"));
    assert!(basic_ref.contains("avoid inventing sentiment"));
    assert!(basic_ref.contains("--stream"));
    assert!(basic_ref.contains("--raw-stream"));
    assert!(basic_ref.contains("--allowed-domain"));
    assert!(basic_ref.contains("--allowed-x-handle"));
    assert!(advanced_ref.contains("video-edit"));
    assert!(advanced_ref.contains("video-extend"));
    assert!(advanced_ref.contains("--auth-file"));
    assert!(errors_ref.contains("bad-credentials"));
    assert!(errors_ref.contains("Sparse Search Results"));
    assert!(advanced_ref.contains("stt-stream"));
    assert!(advanced_ref.contains("endpointing"));
    assert!(skill_validation.contains("A1 | `login`"));
    assert!(skill_validation.contains("A27 | `stt-stream`"));
    assert!(skill_validation.contains("P1 | `login`"));
    assert!(skill_validation.contains("P15 | `stt-stream`"));
    assert!(skill_validation.contains("## Local File Scenarios"));
    assert!(skill_validation.contains("Local streaming transcription"));
    assert!(skill_validation.contains("### N1. Do not invent a local video extension command"));
}
