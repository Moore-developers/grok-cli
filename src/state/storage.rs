use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::AppError;

use super::model::AuthState;

#[derive(Debug, Clone)]
pub struct StateInspection {
    pub exists: bool,
    pub parsed: Option<AuthState>,
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StateStore;

impl StateStore {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_path(&self, override_path: Option<&Path>) -> PathBuf {
        if let Some(path) = override_path {
            return path.to_path_buf();
        }

        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(".grok-cli").join("auth.json");
        }

        PathBuf::from(".grok-cli").join("auth.json")
    }

    pub fn inspect(&self, path: &Path) -> Result<StateInspection, AppError> {
        let raw = match self.read_raw(path)? {
            Some(raw) => raw,
            None => {
                return Ok(StateInspection {
                    exists: false,
                    parsed: None,
                    problems: vec!["state file missing".to_string()],
                });
            }
        };

        let parsed: AuthState = match serde_json::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(error) => {
                return Ok(StateInspection {
                    exists: true,
                    parsed: None,
                    problems: vec![format!("invalid json: {error}")],
                });
            }
        };

        let problems = parsed.validate();

        Ok(StateInspection {
            exists: true,
            parsed: Some(parsed),
            problems,
        })
    }

    pub fn load_valid_state(&self, path: &Path) -> Result<AuthState, AppError> {
        let inspection = self.inspect(path)?;

        if !inspection.exists {
            return Err(AppError::state_file_missing(path));
        }

        if !inspection.problems.is_empty() {
            return Err(AppError::state_file_invalid(inspection.problems.join("; ")));
        }

        inspection
            .parsed
            .ok_or_else(|| AppError::state_file_invalid("state file could not be parsed"))
    }

    pub fn write_state(&self, path: &Path, state: &AuthState) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::io(format!(
                    "failed to create state directory {}: {error}",
                    parent.display()
                ))
            })?;
        }

        let raw = serde_json::to_string_pretty(state).map_err(|error| {
            AppError::state_file_invalid(format!("failed to serialize state: {error}"))
        })?;
        let temp_path = temp_state_path(path);
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            AppError::io(format!(
                "failed to create temp state file {}: {error}",
                temp_path.display()
            ))
        })?;
        file.write_all(raw.as_bytes()).map_err(|error| {
            AppError::io(format!(
                "failed to write temp state file {}: {error}",
                temp_path.display()
            ))
        })?;
        file.sync_all().map_err(|error| {
            AppError::io(format!(
                "failed to sync temp state file {}: {error}",
                temp_path.display()
            ))
        })?;
        drop(file);

        fs::rename(&temp_path, path).map_err(|error| {
            let _ = fs::remove_file(&temp_path);
            AppError::io(format!(
                "failed to replace state file {} with {}: {error}",
                path.display(),
                temp_path.display()
            ))
        })
    }

    pub fn remove_state(&self, path: &Path) -> Result<bool, AppError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(AppError::io(format!(
                "failed to remove state file {}: {error}",
                path.display()
            ))),
        }
    }

    fn read_raw(&self, path: &Path) -> Result<Option<String>, AppError> {
        match fs::read_to_string(path) {
            Ok(raw) => Ok(Some(raw)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(AppError::io(format!(
                "failed to read state file {}: {error}",
                path.display()
            ))),
        }
    }
}

fn temp_state_path(path: &Path) -> PathBuf {
    let parent = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("auth.json");
    parent.join(format!(".{file_name}.tmp-{}", Uuid::new_v4()))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{StateStore, temp_state_path};
    use crate::state::model::AuthState;

    #[test]
    fn write_state_replaces_existing_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        let store = StateStore::new();

        let mut first = AuthState::empty("https://api.x.ai/v1".to_string());
        first.tokens.access_token = Some("first-token".to_string());
        store.write_state(&path, &first).unwrap();

        let mut second = AuthState::empty("https://api.x.ai/v1".to_string());
        second.tokens.access_token = Some("second-token".to_string());
        store.write_state(&path, &second).unwrap();

        let saved = store.load_valid_state(&path).unwrap();
        assert_eq!(saved.tokens.access_token.as_deref(), Some("second-token"));
    }

    #[test]
    fn temp_state_path_stays_in_same_directory() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        let temp_path = temp_state_path(&path);

        assert_eq!(temp_path.parent(), path.parent());
        assert_ne!(temp_path, path);
    }
}
