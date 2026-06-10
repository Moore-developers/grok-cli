use std::cmp::Ordering;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::app::AppContext;
use crate::args::UpdateOptions;
use crate::cli::CommandResult;
use crate::error::{AppError, CommandError, ErrorCode};
use crate::output;

const COMMAND: &str = "update";
const CONFIG_VERSION: u32 = 1;
const DEFAULT_RELEASE_URL: &str =
    "https://api.github.com/repos/Moore-developers/grok-cli/releases/latest";
const REPOSITORY_URL: &str = "https://github.com/Moore-developers/grok-cli.git";
const RELEASE_PAGE_URL: &str = "https://github.com/Moore-developers/grok-cli/releases/latest";
const UPDATE_CACHE_TTL_SECONDS: i64 = 24 * 60 * 60;
const PASSIVE_CHECK_TIMEOUT_SECONDS: u64 = 2;
const ACTIVE_CHECK_TIMEOUT_SECONDS: u64 = 20;
const DOWNLOAD_TIMEOUT_SECONDS: u64 = 300;
const ENV_DISABLE_UPDATE_CHECK: &str = "GROK_CLI_NO_UPDATE_CHECK";
const ENV_RELEASE_URL: &str = "GROK_CLI_UPDATE_RELEASE_URL";
const ENV_CONFIG_FILE: &str = "GROK_CLI_UPDATE_STATE_FILE";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateConfig {
    version: u32,
    auto_check_enabled: bool,
    #[serde(default)]
    last_checked_at: Option<String>,
    #[serde(default)]
    latest_version: Option<String>,
    #[serde(default)]
    latest_tag: Option<String>,
    #[serde(default)]
    latest_release_url: Option<String>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            auto_check_enabled: true,
            last_checked_at: None,
            latest_version: None,
            latest_tag: None,
            latest_release_url: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    #[serde(rename = "tag_name")]
    tag_name: String,
    #[serde(default, alias = "htmlUrl", alias = "html_url")]
    html_url: Option<String>,
    #[serde(default)]
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    #[serde(default, alias = "browserDownloadUrl", alias = "browser_download_url")]
    browser_download_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateSettingData {
    auto_check_enabled: bool,
    update_config_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateCheckData {
    current_version: String,
    latest_version: String,
    latest_tag: String,
    update_available: bool,
    release_url: String,
    install_strategy: String,
    asset_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateInstallData {
    current_version: String,
    latest_version: String,
    latest_tag: String,
    update_available: bool,
    installed: bool,
    install_strategy: String,
    asset_name: Option<String>,
    release_url: String,
    message: String,
}

#[derive(Debug, Clone)]
struct ReleaseInfo {
    tag: String,
    version: String,
    url: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone)]
struct ReleaseAsset {
    name: String,
    download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InstallStrategy {
    MacOsAarch64Release,
    WindowsX64Release,
    CargoSource,
}

impl InstallStrategy {
    fn as_str(&self) -> &'static str {
        match self {
            Self::MacOsAarch64Release => "github_release_macos_aarch64",
            Self::WindowsX64Release => "github_release_windows_x64",
            Self::CargoSource => "cargo_source",
        }
    }

    fn asset_name(&self) -> Option<&'static str> {
        match self {
            Self::MacOsAarch64Release => Some("grok-cli-macos-aarch64-apple-darwin.tar.gz"),
            Self::WindowsX64Release => Some("grok-cli-windows-x86_64-pc-windows-msvc.zip"),
            Self::CargoSource => None,
        }
    }
}

#[derive(Debug, Clone)]
enum InstallOutcome {
    Installed,
    Started,
}

pub fn execute(ctx: &AppContext, opts: UpdateOptions) -> CommandResult {
    if opts.no_update_check {
        let path = update_config_path();
        let mut config = load_update_config_lossy(&path);
        config.auto_check_enabled = false;
        write_update_config(&path, &config)
            .map_err(|error| CommandError::new(COMMAND, opts.json, error))?;
        let data = UpdateSettingData {
            auto_check_enabled: false,
            update_config_path: path.display().to_string(),
        };
        return print_setting_result(opts.json, &data, "Update checks disabled.");
    }

    if opts.enable_update_check {
        let path = update_config_path();
        let mut config = load_update_config_lossy(&path);
        config.auto_check_enabled = true;
        write_update_config(&path, &config)
            .map_err(|error| CommandError::new(COMMAND, opts.json, error))?;
        let data = UpdateSettingData {
            auto_check_enabled: true,
            update_config_path: path.display().to_string(),
        };
        return print_setting_result(opts.json, &data, "Update checks enabled.");
    }

    let release = fetch_latest_release(ctx, Duration::from_secs(ACTIVE_CHECK_TIMEOUT_SECONDS))
        .map_err(|error| CommandError::new(COMMAND, opts.json, error))?;
    let check = build_update_check_data(&release);
    remember_release_check(&release);

    if opts.check {
        if opts.json {
            output::print_json_success(COMMAND, &check);
        } else {
            print_human_check(&check);
        }
        return Ok(());
    }

    if !check.update_available && !opts.force {
        let data = UpdateInstallData {
            current_version: check.current_version,
            latest_version: check.latest_version,
            latest_tag: check.latest_tag,
            update_available: false,
            installed: false,
            install_strategy: check.install_strategy,
            asset_name: check.asset_name,
            release_url: check.release_url,
            message: "grok-cli is already up to date.".to_string(),
        };
        if opts.json {
            output::print_json_success(COMMAND, &data);
        } else {
            println!("grok-cli is already up to date ({}).", data.current_version);
        }
        return Ok(());
    }

    let strategy = select_install_strategy(env::consts::OS, env::consts::ARCH);
    let outcome = install_latest_release(ctx, &release, &strategy)
        .map_err(|error| CommandError::new(COMMAND, opts.json, error))?;
    let data = build_update_install_data(&release, &check, &strategy, outcome);

    if opts.json {
        output::print_json_success(COMMAND, &data);
    } else {
        println!("{}", data.message);
        println!("latest: {}", data.latest_tag);
    }

    Ok(())
}

pub fn maybe_print_passive_update_notice(ctx: &AppContext, allowed_by_command: bool) {
    if !should_run_passive_update_check(allowed_by_command) {
        return;
    }

    let path = update_config_path();
    let config = match read_update_config(&path) {
        Ok(config) => config,
        Err(error) => {
            tracing::debug!(error = %error.message, "skipping passive update check");
            return;
        }
    };

    if !config.auto_check_enabled || !is_update_cache_expired(&config, OffsetDateTime::now_utc()) {
        return;
    }

    let release =
        match fetch_latest_release(ctx, Duration::from_secs(PASSIVE_CHECK_TIMEOUT_SECONDS)) {
            Ok(release) => release,
            Err(error) => {
                tracing::debug!(error = %error.message, "passive update check failed");
                return;
            }
        };

    remember_release_check(&release);

    if is_update_available(current_version(), &release.version) {
        eprintln!(
            "grok-cli v{} is available. Run: grok-cli update",
            release.version
        );
    }
}

fn print_setting_result(
    json: bool,
    data: &UpdateSettingData,
    human_message: &str,
) -> CommandResult {
    if json {
        output::print_json_success(COMMAND, data);
    } else {
        println!("{human_message}");
    }
    Ok(())
}

fn print_human_check(data: &UpdateCheckData) {
    if data.update_available {
        println!(
            "grok-cli v{} is available (current {}).",
            data.latest_version, data.current_version
        );
        println!("release: {}", data.release_url);
        println!("run: grok-cli update");
    } else {
        println!("grok-cli is up to date ({}).", data.current_version);
    }
}

fn build_update_check_data(release: &ReleaseInfo) -> UpdateCheckData {
    let strategy = select_install_strategy(env::consts::OS, env::consts::ARCH);
    UpdateCheckData {
        current_version: current_version().to_string(),
        latest_version: release.version.clone(),
        latest_tag: release.tag.clone(),
        update_available: is_update_available(current_version(), &release.version),
        release_url: release.url.clone(),
        install_strategy: strategy.as_str().to_string(),
        asset_name: strategy.asset_name().map(str::to_string),
    }
}

fn build_update_install_data(
    release: &ReleaseInfo,
    check: &UpdateCheckData,
    strategy: &InstallStrategy,
    outcome: InstallOutcome,
) -> UpdateInstallData {
    let installed = matches!(outcome, InstallOutcome::Installed);
    let message = install_outcome_message(&release.version, outcome);
    UpdateInstallData {
        current_version: current_version().to_string(),
        latest_version: release.version.clone(),
        latest_tag: release.tag.clone(),
        update_available: check.update_available,
        installed,
        install_strategy: strategy.as_str().to_string(),
        asset_name: strategy.asset_name().map(str::to_string),
        release_url: release.url.clone(),
        message,
    }
}

fn install_outcome_message(latest_version: &str, outcome: InstallOutcome) -> String {
    match outcome {
        InstallOutcome::Installed => {
            format!(
                "Updated grok-cli from {} to {latest_version}.",
                current_version()
            )
        }
        InstallOutcome::Started => {
            "Started a background updater. Restart your terminal after it finishes.".to_string()
        }
    }
}

fn fetch_latest_release(ctx: &AppContext, timeout: Duration) -> Result<ReleaseInfo, AppError> {
    let release_url = env::var(ENV_RELEASE_URL).unwrap_or_else(|_| DEFAULT_RELEASE_URL.to_string());
    let response = ctx
        .http_client
        .get(&release_url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .timeout(timeout)
        .send()
        .map_err(|error| map_update_transport_error("latest release request", &error))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("latest release request failed with status {status}: {body}"),
        ));
    }

    let parsed = response.json::<GitHubRelease>().map_err(|error| {
        AppError::new(
            ErrorCode::ResponseDecodeFailed,
            format!("failed to decode latest release response: {error}"),
        )
    })?;

    release_info_from_github(parsed)
}

fn release_info_from_github(release: GitHubRelease) -> Result<ReleaseInfo, AppError> {
    let version = version_from_tag(&release.tag_name).ok_or_else(|| {
        AppError::new(
            ErrorCode::ResponseDecodeFailed,
            format!("release tag is not a version: {}", release.tag_name),
        )
    })?;
    let assets = release
        .assets
        .into_iter()
        .filter_map(|asset| {
            let download_url = asset.browser_download_url.or(asset.url)?;
            Some(ReleaseAsset {
                name: asset.name,
                download_url,
            })
        })
        .collect::<Vec<_>>();

    Ok(ReleaseInfo {
        url: release
            .html_url
            .unwrap_or_else(|| RELEASE_PAGE_URL.to_string()),
        tag: release.tag_name,
        version,
        assets,
    })
}

fn install_latest_release(
    ctx: &AppContext,
    release: &ReleaseInfo,
    strategy: &InstallStrategy,
) -> Result<InstallOutcome, AppError> {
    match strategy {
        InstallStrategy::MacOsAarch64Release => {
            let asset_name = strategy.asset_name().expect("macOS strategy has asset");
            install_unix_tarball(ctx, release, asset_name)?;
            Ok(InstallOutcome::Installed)
        }
        InstallStrategy::WindowsX64Release => {
            let asset_name = strategy.asset_name().expect("Windows strategy has asset");
            install_windows_zip(ctx, release, asset_name)?;
            Ok(InstallOutcome::Started)
        }
        InstallStrategy::CargoSource => {
            install_with_cargo(&release.tag)?;
            Ok(InstallOutcome::Installed)
        }
    }
}

fn install_unix_tarball(
    ctx: &AppContext,
    release: &ReleaseInfo,
    asset_name: &str,
) -> Result<(), AppError> {
    let new_binary = download_and_extract_unix_tarball(ctx, release, asset_name)?;
    replace_current_executable(&new_binary)
}

fn download_and_extract_unix_tarball(
    ctx: &AppContext,
    release: &ReleaseInfo,
    asset_name: &str,
) -> Result<PathBuf, AppError> {
    let temp_dir = create_update_temp_dir()?;
    let asset_bytes = download_verified_asset(ctx, release, asset_name)?;
    let archive_path = temp_dir.join(asset_name);
    fs::write(&archive_path, asset_bytes).map_err(|error| {
        AppError::io(format!(
            "failed to write update archive {}: {error}",
            archive_path.display()
        ))
    })?;

    let extract_dir = temp_dir.join("extract");
    fs::create_dir_all(&extract_dir).map_err(|error| {
        AppError::io(format!(
            "failed to create update extract directory {}: {error}",
            extract_dir.display()
        ))
    })?;

    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&extract_dir)
        .status()
        .map_err(|error| AppError::io(format!("failed to run tar: {error}")))?;
    if !status.success() {
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("failed to extract update archive with status {status}"),
        ));
    }

    Ok(extract_dir.join("grok-cli"))
}

fn install_windows_zip(
    ctx: &AppContext,
    release: &ReleaseInfo,
    asset_name: &str,
) -> Result<(), AppError> {
    let temp_dir = create_update_temp_dir()?;
    let asset_bytes = download_verified_asset(ctx, release, asset_name)?;
    let zip_path = temp_dir.join(asset_name);
    fs::write(&zip_path, asset_bytes).map_err(|error| {
        AppError::io(format!(
            "failed to write update archive {}: {error}",
            zip_path.display()
        ))
    })?;

    let current_exe = env::current_exe()
        .map_err(|error| AppError::io(format!("failed to resolve current executable: {error}")))?;
    let script_path = temp_dir.join("grok-cli-update.ps1");
    let extract_dir = temp_dir.join("extract");
    let script = build_windows_update_script(&zip_path, &extract_dir, &current_exe);
    fs::write(&script_path, script).map_err(|error| {
        AppError::io(format!(
            "failed to write Windows update script {}: {error}",
            script_path.display()
        ))
    })?;

    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path)
        .spawn()
        .map_err(|error| AppError::io(format!("failed to start Windows updater: {error}")))?;
    Ok(())
}

fn install_with_cargo(tag: &str) -> Result<(), AppError> {
    install_with_cargo_program(tag, Path::new("cargo"))
}

fn install_with_cargo_program(tag: &str, program: &Path) -> Result<(), AppError> {
    let status = Command::new(program)
        .args(cargo_install_args(tag))
        .status()
        .map_err(|error| {
            AppError::io(format!(
                "failed to start cargo install. Source updates require Rust/Cargo: {error}"
            ))
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("cargo install failed with status {status}"),
        ))
    }
}

fn cargo_install_args(tag: &str) -> [&str; 7] {
    [
        "install",
        "--git",
        REPOSITORY_URL,
        "--tag",
        tag,
        "--locked",
        "--force",
    ]
}

fn download_verified_asset(
    ctx: &AppContext,
    release: &ReleaseInfo,
    asset_name: &str,
) -> Result<Vec<u8>, AppError> {
    let asset = find_asset(release, asset_name)?;
    let checksum = find_asset(release, &format!("{asset_name}.sha256"))?;
    let asset_bytes = download_bytes(ctx, &asset.download_url)?;
    let checksum_text =
        String::from_utf8(download_bytes(ctx, &checksum.download_url)?).map_err(|error| {
            AppError::new(
                ErrorCode::ResponseDecodeFailed,
                format!("checksum file is not UTF-8: {error}"),
            )
        })?;
    verify_sha256(&asset_bytes, &checksum_text)?;
    Ok(asset_bytes)
}

fn find_asset<'a>(
    release: &'a ReleaseInfo,
    asset_name: &str,
) -> Result<&'a ReleaseAsset, AppError> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::RequestFailed,
                format!(
                    "latest release {} does not include required asset {asset_name}",
                    release.tag
                ),
            )
        })
}

fn download_bytes(ctx: &AppContext, url: &str) -> Result<Vec<u8>, AppError> {
    let response = ctx
        .http_client
        .get(url)
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECONDS))
        .send()
        .map_err(|error| map_update_transport_error("release asset download", &error))?;
    if !response.status().is_success() {
        let status = response.status();
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("release asset download failed with status {status}: {url}"),
        ));
    }
    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| {
            AppError::new(
                ErrorCode::ResponseDecodeFailed,
                format!("failed to read release asset bytes: {error}"),
            )
        })
}

fn verify_sha256(bytes: &[u8], checksum_text: &str) -> Result<(), AppError> {
    let expected = checksum_text
        .split_whitespace()
        .next()
        .ok_or_else(|| AppError::new(ErrorCode::ResponseDecodeFailed, "empty checksum file"))?;
    let actual = sha256_hex(bytes);
    if !expected.eq_ignore_ascii_case(&actual) {
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!("release asset checksum mismatch: expected {expected}, got {actual}"),
        ));
    }
    Ok(())
}

fn replace_current_executable(new_binary: &Path) -> Result<(), AppError> {
    if !new_binary.exists() {
        return Err(AppError::new(
            ErrorCode::RequestFailed,
            format!(
                "update archive did not contain expected binary {}",
                new_binary.display()
            ),
        ));
    }

    let current_exe = env::current_exe()
        .map_err(|error| AppError::io(format!("failed to resolve current executable: {error}")))?;
    let backup = current_exe.with_extension(format!("old-{}", Uuid::new_v4()));

    fs::rename(&current_exe, &backup).map_err(|error| {
        AppError::io(format!(
            "failed to back up current executable {}: {error}",
            current_exe.display()
        ))
    })?;

    if let Err(error) = copy_executable(new_binary, &current_exe) {
        let _ = fs::rename(&backup, &current_exe);
        return Err(error);
    }

    let _ = fs::remove_file(&backup);
    Ok(())
}

fn copy_executable(from: &Path, to: &Path) -> Result<(), AppError> {
    fs::copy(from, to).map_err(|error| {
        AppError::io(format!(
            "failed to install new executable {}: {error}",
            to.display()
        ))
    })?;
    copy_permissions(from, to)
}

#[cfg(unix)]
fn copy_permissions(from: &Path, to: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;

    let mode = fs::metadata(from)
        .map_err(|error| {
            AppError::io(format!(
                "failed to inspect new executable {}: {error}",
                from.display()
            ))
        })?
        .permissions()
        .mode();
    fs::set_permissions(to, fs::Permissions::from_mode(mode)).map_err(|error| {
        AppError::io(format!(
            "failed to set executable permissions {}: {error}",
            to.display()
        ))
    })
}

#[cfg(not(unix))]
fn copy_permissions(_from: &Path, _to: &Path) -> Result<(), AppError> {
    Ok(())
}

fn create_update_temp_dir() -> Result<PathBuf, AppError> {
    let path = env::temp_dir().join(format!("grok-cli-update-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).map_err(|error| {
        AppError::io(format!(
            "failed to create update temp directory {}: {error}",
            path.display()
        ))
    })?;
    Ok(path)
}

fn read_update_config(path: &Path) -> Result<UpdateConfig, AppError> {
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<UpdateConfig>(&raw).map_err(|error| {
            AppError::state_file_invalid(format!(
                "invalid update config {}: {error}",
                path.display()
            ))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(UpdateConfig::default()),
        Err(error) => Err(AppError::io(format!(
            "failed to read update config {}: {error}",
            path.display()
        ))),
    }
}

fn load_update_config_lossy(path: &Path) -> UpdateConfig {
    read_update_config(path).unwrap_or_default()
}

fn write_update_config(path: &Path, config: &UpdateConfig) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::io(format!(
                "failed to create update config directory {}: {error}",
                parent.display()
            ))
        })?;
    }

    let raw = serde_json::to_string_pretty(config).map_err(|error| {
        AppError::new(
            ErrorCode::OutputSerializationFailed,
            format!("failed to serialize update config: {error}"),
        )
    })?;
    let temp_path = temp_config_path(path);
    let mut file = fs::File::create(&temp_path).map_err(|error| {
        AppError::io(format!(
            "failed to create temp update config {}: {error}",
            temp_path.display()
        ))
    })?;
    file.write_all(raw.as_bytes()).map_err(|error| {
        AppError::io(format!(
            "failed to write temp update config {}: {error}",
            temp_path.display()
        ))
    })?;
    file.sync_all().map_err(|error| {
        AppError::io(format!(
            "failed to sync temp update config {}: {error}",
            temp_path.display()
        ))
    })?;
    drop(file);
    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        AppError::io(format!(
            "failed to replace update config {}: {error}",
            path.display()
        ))
    })
}

fn remember_release_check(release: &ReleaseInfo) {
    let path = update_config_path();
    let mut config = load_update_config_lossy(&path);
    config.last_checked_at = Some(
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    );
    config.latest_version = Some(release.version.clone());
    config.latest_tag = Some(release.tag.clone());
    config.latest_release_url = Some(release.url.clone());
    if let Err(error) = write_update_config(&path, &config) {
        tracing::debug!(error = %error.message, "failed to remember update check");
    }
}

fn update_config_path() -> PathBuf {
    if let Some(path) = env::var_os(ENV_CONFIG_FILE) {
        return PathBuf::from(path);
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".grok-cli").join("update.json");
    }

    PathBuf::from(".grok-cli").join("update.json")
}

fn temp_config_path(path: &Path) -> PathBuf {
    let parent = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("update.json");
    parent.join(format!(".{file_name}.tmp-{}", Uuid::new_v4()))
}

fn is_update_cache_expired(config: &UpdateConfig, now: OffsetDateTime) -> bool {
    let Some(last_checked_at) = &config.last_checked_at else {
        return true;
    };

    let Ok(last_checked_at) = OffsetDateTime::parse(last_checked_at, &Rfc3339) else {
        return true;
    };

    match now - last_checked_at {
        duration if duration.is_negative() => true,
        duration => duration.whole_seconds() >= UPDATE_CACHE_TTL_SECONDS,
    }
}

fn should_run_passive_update_check(allowed_by_command: bool) -> bool {
    if !allowed_by_command {
        return false;
    }
    if env_flag_enabled(ENV_DISABLE_UPDATE_CHECK) {
        return false;
    }

    passive_output_is_terminal()
}

fn passive_output_is_terminal() -> bool {
    use std::io::IsTerminal;

    std::io::stdout().is_terminal() && std::io::stderr().is_terminal()
}

fn env_flag_enabled(name: &str) -> bool {
    match env::var(name) {
        Ok(value) => {
            let value = value.trim().to_ascii_lowercase();
            !matches!(value.as_str(), "" | "0" | "false" | "no" | "off")
        }
        Err(_) => false,
    }
}

fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn version_from_tag(tag: &str) -> Option<String> {
    let trimmed = tag.trim().trim_start_matches('v');
    let first = trimmed.chars().next()?;
    if !first.is_ascii_digit() {
        return None;
    }
    Some(trimmed.to_string())
}

fn is_update_available(current: &str, latest: &str) -> bool {
    compare_versions(current, latest) == Ordering::Less
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = version_number_parts(left);
    let right_parts = version_number_parts(right);
    let max = left_parts.len().max(right_parts.len()).max(3);

    for index in 0..max {
        let left = *left_parts.get(index).unwrap_or(&0);
        let right = *right_parts.get(index).unwrap_or(&0);
        match left.cmp(&right) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }

    Ordering::Equal
}

fn version_number_parts(version: &str) -> Vec<u64> {
    version
        .trim()
        .trim_start_matches('v')
        .split(|value: char| !value.is_ascii_digit())
        .filter(|value| !value.is_empty())
        .map(|value| value.parse::<u64>().unwrap_or(0))
        .collect()
}

fn select_install_strategy(os: &str, arch: &str) -> InstallStrategy {
    match (os, arch) {
        ("macos", "aarch64") | ("macos", "arm64") => InstallStrategy::MacOsAarch64Release,
        ("windows", "x86_64") => InstallStrategy::WindowsX64Release,
        _ => InstallStrategy::CargoSource,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn escape_powershell_path(path: &Path) -> String {
    path.display().to_string().replace('"', "`\"")
}

fn build_windows_update_script(zip_path: &Path, extract_dir: &Path, current_exe: &Path) -> String {
    format!(
        r#"
Start-Sleep -Seconds 2
New-Item -ItemType Directory -Force -Path "{extract}" | Out-Null
Expand-Archive -LiteralPath "{zip}" -DestinationPath "{extract}" -Force
Copy-Item -LiteralPath "{extract}\grok-cli.exe" -Destination "{exe}" -Force
"#,
        extract = escape_powershell_path(extract_dir),
        zip = escape_powershell_path(zip_path),
        exe = escape_powershell_path(current_exe),
    )
}

fn map_update_transport_error(operation: &str, error: &reqwest::Error) -> AppError {
    let code = if error.is_timeout() {
        ErrorCode::NetworkTimeout
    } else if error.is_connect() {
        ErrorCode::NetworkConnectFailed
    } else {
        ErrorCode::NetworkTransportFailed
    };
    AppError::new(code, format!("{operation} failed: {error}"))
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, OnceLock};
    use std::thread;
    use time::Duration as TimeDuration;

    use crate::app::AppContext;

    use super::{
        ENV_CONFIG_FILE, ENV_DISABLE_UPDATE_CHECK, GitHubRelease, GitHubReleaseAsset,
        InstallOutcome, InstallStrategy, ReleaseAsset, ReleaseInfo, UpdateConfig,
        build_update_check_data, build_update_install_data, build_windows_update_script,
        cargo_install_args, compare_versions, copy_executable, create_update_temp_dir,
        download_bytes, download_verified_asset, env_flag_enabled, escape_powershell_path,
        find_asset, install_outcome_message, install_with_cargo_program, is_update_available,
        is_update_cache_expired, load_update_config_lossy, read_update_config,
        release_info_from_github, replace_current_executable, select_install_strategy, sha256_hex,
        should_run_passive_update_check, temp_config_path, update_config_path, verify_sha256,
        version_from_tag, write_update_config,
    };

    #[cfg(unix)]
    use super::download_and_extract_unix_tarball;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn parses_release_tags() {
        assert_eq!(version_from_tag("v0.1.5").as_deref(), Some("0.1.5"));
        assert_eq!(version_from_tag("0.2.0").as_deref(), Some("0.2.0"));
        assert_eq!(version_from_tag("release-1").as_deref(), None);
    }

    #[test]
    fn compares_versions_numerically() {
        assert_eq!(compare_versions("0.1.9", "0.1.10"), Ordering::Less);
        assert_eq!(compare_versions("v1.2", "1.2.0"), Ordering::Equal);
        assert_eq!(compare_versions("2.0.0", "1.9.9"), Ordering::Greater);
        assert!(is_update_available("0.1.4", "0.1.5"));
        assert!(!is_update_available("0.1.4", "0.1.4"));
    }

    #[test]
    fn selects_install_strategy_from_platform() {
        assert_eq!(
            select_install_strategy("macos", "aarch64"),
            InstallStrategy::MacOsAarch64Release
        );
        assert_eq!(
            select_install_strategy("macos", "arm64"),
            InstallStrategy::MacOsAarch64Release
        );
        assert_eq!(
            select_install_strategy("windows", "x86_64"),
            InstallStrategy::WindowsX64Release
        );
        assert_eq!(
            select_install_strategy("linux", "x86_64"),
            InstallStrategy::CargoSource
        );
    }

    #[test]
    fn install_strategy_reports_names_and_assets() {
        assert_eq!(
            InstallStrategy::MacOsAarch64Release.as_str(),
            "github_release_macos_aarch64"
        );
        assert_eq!(
            InstallStrategy::MacOsAarch64Release.asset_name(),
            Some("grok-cli-macos-aarch64-apple-darwin.tar.gz")
        );
        assert_eq!(
            InstallStrategy::WindowsX64Release.as_str(),
            "github_release_windows_x64"
        );
        assert_eq!(
            InstallStrategy::WindowsX64Release.asset_name(),
            Some("grok-cli-windows-x86_64-pc-windows-msvc.zip")
        );
        assert_eq!(InstallStrategy::CargoSource.as_str(), "cargo_source");
        assert_eq!(InstallStrategy::CargoSource.asset_name(), None);
    }

    #[test]
    fn release_info_uses_browser_download_urls() {
        let release = GitHubRelease {
            tag_name: "v0.1.5".to_string(),
            html_url: Some("https://example.test/release".to_string()),
            assets: vec![GitHubReleaseAsset {
                name: "asset.tar.gz".to_string(),
                browser_download_url: Some("https://example.test/asset".to_string()),
                url: None,
            }],
        };

        let info = release_info_from_github(release).unwrap();
        assert_eq!(info.tag, "v0.1.5");
        assert_eq!(info.version, "0.1.5");
        assert_eq!(info.url, "https://example.test/release");
        assert_eq!(info.assets[0].download_url, "https://example.test/asset");
    }

    #[test]
    fn release_info_falls_back_to_asset_api_url_and_release_page() {
        let release = GitHubRelease {
            tag_name: "v0.1.5".to_string(),
            html_url: None,
            assets: vec![
                GitHubReleaseAsset {
                    name: "asset.tar.gz".to_string(),
                    browser_download_url: None,
                    url: Some("https://api.example.test/asset".to_string()),
                },
                GitHubReleaseAsset {
                    name: "ignored".to_string(),
                    browser_download_url: None,
                    url: None,
                },
            ],
        };

        let info = release_info_from_github(release).unwrap();
        assert_eq!(
            info.url,
            "https://github.com/Moore-developers/grok-cli/releases/latest"
        );
        assert_eq!(info.assets.len(), 1);
        assert_eq!(
            info.assets[0].download_url,
            "https://api.example.test/asset"
        );
    }

    #[test]
    fn release_info_rejects_non_version_tags() {
        let release = GitHubRelease {
            tag_name: "release-current".to_string(),
            html_url: None,
            assets: vec![],
        };

        let error = release_info_from_github(release).unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::ResponseDecodeFailed);
    }

    #[test]
    fn cache_is_expired_after_ttl() {
        let now = time::OffsetDateTime::now_utc();
        let fresh = UpdateConfig {
            last_checked_at: Some(
                now.format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ),
            ..UpdateConfig::default()
        };
        let stale = UpdateConfig {
            last_checked_at: Some(
                (now - TimeDuration::hours(25))
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ),
            ..UpdateConfig::default()
        };

        assert!(!is_update_cache_expired(&fresh, now));
        assert!(is_update_cache_expired(&stale, now));
        assert!(is_update_cache_expired(&UpdateConfig::default(), now));

        let future = UpdateConfig {
            last_checked_at: Some(
                (now + TimeDuration::minutes(5))
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ),
            ..UpdateConfig::default()
        };
        let invalid = UpdateConfig {
            last_checked_at: Some("not-a-date".to_string()),
            ..UpdateConfig::default()
        };
        assert!(is_update_cache_expired(&future, now));
        assert!(is_update_cache_expired(&invalid, now));
    }

    #[test]
    fn config_round_trips_and_uses_override_path() {
        let _guard = env_lock();
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("update.json");
        unsafe {
            std::env::set_var(ENV_CONFIG_FILE, &path);
        }

        let config = UpdateConfig {
            auto_check_enabled: false,
            latest_version: Some("0.2.0".to_string()),
            ..UpdateConfig::default()
        };
        write_update_config(&path, &config).unwrap();
        let loaded = read_update_config(&path).unwrap();

        assert!(!loaded.auto_check_enabled);
        assert_eq!(loaded.latest_version.as_deref(), Some("0.2.0"));
        assert_eq!(update_config_path(), path);
        assert_ne!(temp_config_path(&path), path);

        unsafe {
            std::env::remove_var(ENV_CONFIG_FILE);
        }
    }

    #[test]
    fn invalid_config_returns_state_error() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("update.json");
        std::fs::write(&path, "{not-json").unwrap();

        let error = read_update_config(&path).unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::StateFileInvalid);
    }

    #[test]
    fn missing_config_loads_default_and_lossy_loader_ignores_invalid_json() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing.json");
        assert!(read_update_config(&missing).unwrap().auto_check_enabled);

        let broken = temp.path().join("broken.json");
        std::fs::write(&broken, "{not-json").unwrap();
        assert!(load_update_config_lossy(&broken).auto_check_enabled);
    }

    #[test]
    fn env_flags_accept_common_false_values() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var(ENV_DISABLE_UPDATE_CHECK, "0");
        }
        assert!(!env_flag_enabled(ENV_DISABLE_UPDATE_CHECK));
        unsafe {
            std::env::set_var(ENV_DISABLE_UPDATE_CHECK, "true");
        }
        assert!(env_flag_enabled(ENV_DISABLE_UPDATE_CHECK));
        unsafe {
            std::env::remove_var(ENV_DISABLE_UPDATE_CHECK);
        }
    }

    #[test]
    fn passive_update_check_respects_command_and_env_gate() {
        let _guard = env_lock();
        assert!(!should_run_passive_update_check(false));
        unsafe {
            std::env::set_var(ENV_DISABLE_UPDATE_CHECK, "yes");
        }
        assert!(!should_run_passive_update_check(true));
        unsafe {
            std::env::remove_var(ENV_DISABLE_UPDATE_CHECK);
        }
    }

    #[test]
    fn verifies_sha256_checksum_text() {
        let bytes = b"hello";
        let digest = sha256_hex(bytes);
        verify_sha256(bytes, &format!("{digest}  asset.tar.gz")).unwrap();
        assert!(verify_sha256(bytes, "deadbeef asset.tar.gz").is_err());
        assert!(verify_sha256(bytes, "").is_err());
    }

    #[test]
    fn download_verified_asset_fetches_asset_and_checksum() {
        let bytes = b"release-bytes".to_vec();
        let checksum = format!("{}  asset.bin\n", sha256_hex(&bytes)).into_bytes();
        let server = spawn_update_test_server(vec![
            ("200 OK".to_string(), bytes.clone()),
            ("200 OK".to_string(), checksum),
        ]);
        let release = ReleaseInfo {
            tag: "v0.1.5".to_string(),
            version: "0.1.5".to_string(),
            url: "https://example.test/release".to_string(),
            assets: vec![
                ReleaseAsset {
                    name: "asset.bin".to_string(),
                    download_url: format!("{}/asset.bin", server.base_url),
                },
                ReleaseAsset {
                    name: "asset.bin.sha256".to_string(),
                    download_url: format!("{}/asset.bin.sha256", server.base_url),
                },
            ],
        };

        let downloaded =
            download_verified_asset(&AppContext::new(), &release, "asset.bin").unwrap();
        assert_eq!(downloaded, bytes);
        server.join();
    }

    #[test]
    fn download_bytes_reports_http_status_failure() {
        let server = spawn_update_test_server(vec![(
            "503 Service Unavailable".to_string(),
            b"unavailable".to_vec(),
        )]);

        let error = download_bytes(
            &AppContext::new(),
            &format!("{}/asset.bin", server.base_url),
        )
        .unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::RequestFailed);
        server.join();
    }

    #[test]
    fn replace_current_executable_rejects_missing_new_binary() {
        let temp = tempfile::tempdir().unwrap();
        let error = replace_current_executable(&temp.path().join("missing-grok-cli")).unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::RequestFailed);
    }

    #[test]
    fn missing_asset_is_reported() {
        let release = ReleaseInfo {
            tag: "v0.1.5".to_string(),
            version: "0.1.5".to_string(),
            url: "https://example.test".to_string(),
            assets: vec![],
        };

        let error = super::find_asset(&release, "missing.tar.gz").unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::RequestFailed);
    }

    #[test]
    fn finds_release_assets_by_exact_name() {
        let release = sample_release("v0.1.5");
        let asset = find_asset(&release, "grok-cli-macos-aarch64-apple-darwin.tar.gz").unwrap();
        assert_eq!(asset.download_url, "https://example.test/macos");
    }

    #[test]
    fn check_and_install_data_reflect_strategy_and_outcome() {
        let release = sample_release("v99.0.0");
        let check = build_update_check_data(&release);
        assert_eq!(check.latest_version, "99.0.0");
        assert!(check.update_available);

        let installed = build_update_install_data(
            &release,
            &check,
            &InstallStrategy::MacOsAarch64Release,
            InstallOutcome::Installed,
        );
        assert!(installed.installed);
        assert_eq!(
            installed.asset_name.as_deref(),
            Some("grok-cli-macos-aarch64-apple-darwin.tar.gz")
        );
        assert!(installed.message.contains("Updated grok-cli"));

        let started = build_update_install_data(
            &release,
            &check,
            &InstallStrategy::WindowsX64Release,
            InstallOutcome::Started,
        );
        assert!(!started.installed);
        assert_eq!(started.install_strategy, "github_release_windows_x64");
        assert!(started.message.contains("background updater"));
    }

    #[test]
    fn install_outcome_messages_are_user_facing() {
        assert!(install_outcome_message("9.9.9", InstallOutcome::Installed).contains("9.9.9"));
        assert!(install_outcome_message("9.9.9", InstallOutcome::Started).contains("Restart"));
    }

    #[test]
    fn cargo_install_args_are_tagged_and_locked() {
        assert_eq!(
            cargo_install_args("v0.1.5"),
            [
                "install",
                "--git",
                "https://github.com/Moore-developers/grok-cli.git",
                "--tag",
                "v0.1.5",
                "--locked",
                "--force",
            ]
        );
    }

    #[test]
    fn cargo_install_program_reports_spawn_failure() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing-cargo");
        let error = install_with_cargo_program("v0.1.5", &missing).unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::IoError);
    }

    #[test]
    fn windows_update_script_escapes_paths() {
        let zip = std::path::Path::new(r#"C:\tmp\grok "cli".zip"#);
        let extract = std::path::Path::new(r#"C:\tmp\extract"#);
        let exe = std::path::Path::new(r#"C:\Program Files\grok-cli.exe"#);

        assert!(escape_powershell_path(zip).contains("`\"cli`\""));
        let script = build_windows_update_script(zip, extract, exe);
        assert!(script.contains("Expand-Archive"));
        assert!(script.contains("Copy-Item"));
        assert!(script.contains("`\"cli`\""));
    }

    #[test]
    fn temp_dir_and_copy_executable_work_for_regular_files() {
        let temp_dir = create_update_temp_dir().unwrap();
        assert!(temp_dir.exists());

        let source = temp_dir.join("source");
        let dest = temp_dir.join("dest");
        std::fs::write(&source, "binary").unwrap();
        copy_executable(&source, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest).unwrap(), "binary");
    }

    #[test]
    fn copy_executable_reports_missing_source() {
        let temp = tempfile::tempdir().unwrap();
        let error =
            copy_executable(&temp.path().join("missing"), &temp.path().join("dest")).unwrap_err();
        assert_eq!(error.code, crate::error::ErrorCode::IoError);
    }

    #[cfg(unix)]
    #[test]
    fn download_and_extract_unix_tarball_returns_extracted_binary_path() {
        let temp = tempfile::tempdir().unwrap();
        let package_dir = temp.path().join("package");
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(package_dir.join("grok-cli"), "new binary").unwrap();
        let archive_path = temp
            .path()
            .join("grok-cli-macos-aarch64-apple-darwin.tar.gz");
        let status = std::process::Command::new("tar")
            .arg("-C")
            .arg(&package_dir)
            .arg("-czf")
            .arg(&archive_path)
            .arg("grok-cli")
            .status()
            .unwrap();
        assert!(status.success());

        let archive_bytes = std::fs::read(&archive_path).unwrap();
        let checksum = format!(
            "{}  grok-cli-macos-aarch64-apple-darwin.tar.gz\n",
            sha256_hex(&archive_bytes)
        )
        .into_bytes();
        let server = spawn_update_test_server(vec![
            ("200 OK".to_string(), archive_bytes),
            ("200 OK".to_string(), checksum),
        ]);
        let release = ReleaseInfo {
            tag: "v0.1.5".to_string(),
            version: "0.1.5".to_string(),
            url: "https://example.test/release".to_string(),
            assets: vec![
                ReleaseAsset {
                    name: "grok-cli-macos-aarch64-apple-darwin.tar.gz".to_string(),
                    download_url: format!("{}/archive", server.base_url),
                },
                ReleaseAsset {
                    name: "grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256".to_string(),
                    download_url: format!("{}/archive.sha256", server.base_url),
                },
            ],
        };

        let extracted = download_and_extract_unix_tarball(
            &AppContext::new(),
            &release,
            "grok-cli-macos-aarch64-apple-darwin.tar.gz",
        )
        .unwrap();
        assert_eq!(std::fs::read_to_string(extracted).unwrap(), "new binary");
        server.join();
    }

    fn sample_release(tag: &str) -> ReleaseInfo {
        ReleaseInfo {
            tag: tag.to_string(),
            version: version_from_tag(tag).unwrap(),
            url: format!("https://example.test/{tag}"),
            assets: vec![
                ReleaseAsset {
                    name: "grok-cli-macos-aarch64-apple-darwin.tar.gz".to_string(),
                    download_url: "https://example.test/macos".to_string(),
                },
                ReleaseAsset {
                    name: "grok-cli-macos-aarch64-apple-darwin.tar.gz.sha256".to_string(),
                    download_url: "https://example.test/macos.sha256".to_string(),
                },
                ReleaseAsset {
                    name: "grok-cli-windows-x86_64-pc-windows-msvc.zip".to_string(),
                    download_url: "https://example.test/windows".to_string(),
                },
            ],
        }
    }

    struct UpdateTestServer {
        base_url: String,
        handle: thread::JoinHandle<()>,
    }

    impl UpdateTestServer {
        fn join(self) {
            self.handle.join().unwrap();
        }
    }

    fn spawn_update_test_server(responses: Vec<(String, Vec<u8>)>) -> UpdateTestServer {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for (status, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let _request = read_update_test_request(&mut stream);
                let header = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(header.as_bytes()).unwrap();
                stream.write_all(&body).unwrap();
                stream.flush().unwrap();
            }
        });

        UpdateTestServer {
            base_url: format!("http://127.0.0.1:{port}"),
            handle,
        }
    }

    fn read_update_test_request(stream: &mut std::net::TcpStream) -> String {
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
}
