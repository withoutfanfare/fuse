pub mod aggregate;
pub mod analytics;
pub mod authors;
pub mod batch;
pub mod bookmarks;
pub mod checklist;
pub mod checklist_templates;
pub mod checks;
pub mod comments;
pub mod commits;
pub mod conflicts;
pub mod dependencies;
pub mod deployments;
pub mod diff;
pub mod digest;
pub mod editor;
pub mod filter_presets;
pub mod groups;
pub mod grove;
pub mod handoffs;
pub mod issues;
pub mod label_rules;
pub mod notifications;
pub mod priority_queue;
pub mod pull_requests;
pub mod repositories;
pub mod review_summary;
pub mod reviews;
pub mod sessions;
pub mod settings;
pub mod stale;
pub mod stats;
pub mod sync;
pub mod templates;
pub mod time_tracking;
pub mod workload;

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("GitHub CLI error: {0}")]
    Gh(String),
    #[error("Grove error: {0}")]
    Grove(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Claude CLI error: {0}")]
    Claude(String),
}

// Tauri requires Serialize for error types returned from commands
impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !paths.iter().any(|existing| existing == &candidate) {
        paths.push(candidate);
    }
}

fn cli_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(current_path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&current_path));
    }

    for dir in [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/opt/local/bin",
    ] {
        push_unique_path(&mut paths, PathBuf::from(dir));
    }

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        push_unique_path(&mut paths, home.join(".local/bin"));
        push_unique_path(&mut paths, home.join(".cargo/bin"));
    }

    paths
}

fn augmented_path() -> OsString {
    env::join_paths(cli_search_paths()).unwrap_or_else(|_| env::var_os("PATH").unwrap_or_default())
}

fn resolve_cli_path(binary: &str) -> Option<PathBuf> {
    let binary_path = Path::new(binary);
    if binary_path.components().count() > 1 {
        return Some(binary_path.to_path_buf());
    }

    cli_search_paths()
        .into_iter()
        .map(|dir| dir.join(binary))
        .find(|candidate| candidate.is_file())
}

fn missing_cli_message(binary: &str, label: &str) -> String {
    format!(
        "{label} executable `{binary}` was not found. Fuse checked the app PATH and common macOS install locations like /opt/homebrew/bin, /usr/local/bin, ~/.local/bin, and ~/.cargo/bin. Install it if needed, then fully relaunch the app."
    )
}

pub(crate) fn command_for(
    binary: &str,
    label: &str,
    error_variant: fn(String) -> CommandError,
) -> Result<std::process::Command, CommandError> {
    let resolved = resolve_cli_path(binary)
        .ok_or_else(|| error_variant(missing_cli_message(binary, label)))?;

    let mut command = std::process::Command::new(resolved);
    command.env("PATH", augmented_path());
    Ok(command)
}

pub(crate) fn tokio_command_for(
    binary: &str,
    label: &str,
    error_variant: fn(String) -> CommandError,
) -> Result<TokioCommand, CommandError> {
    let resolved = resolve_cli_path(binary)
        .ok_or_else(|| error_variant(missing_cli_message(binary, label)))?;

    let mut command = TokioCommand::new(resolved);
    command.env("PATH", augmented_path());
    Ok(command)
}
