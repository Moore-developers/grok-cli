use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

use std::fs;
use std::path::Path;

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
    let skill = fs::read_to_string(root.join("skills/grok-cli/SKILL.md")).unwrap();
    let install_ref =
        fs::read_to_string(root.join("skills/grok-cli/references/install-and-auth.md")).unwrap();

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
    assert!(install_ref.contains("--tag v0.1.0 --locked --force"));
}
