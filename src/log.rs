use anyhow::Context;
use etcetera::{BaseStrategy, choose_base_strategy};
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

const LOG_ENV_VAR: &str = "AGENT1_LOG";
const LOG_FILE_MAX_SIZE_BYTES: u64 = 1024 * 1024;

pub(super) fn setup_logging() -> anyhow::Result<()> {
    if std::env::var(LOG_ENV_VAR).map_or(true, |v| v.is_empty()) {
        return Ok(());
    }

    let log_file_path = get_log_file_path().context("couldn't determine log file path")?;
    cleanup(&log_file_path).context("couldn't clean up log file; do it manually")?;

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .context("failed to open log file")?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env(LOG_ENV_VAR))
        .with_ansi(false)
        .with_writer(log_file)
        .init();

    Ok(())
}

fn get_log_file_path() -> anyhow::Result<PathBuf> {
    let log_dir = get_log_dir()?;
    std::fs::create_dir_all(&log_dir).context("couldn't create log directory")?;

    Ok(log_dir.join("agent1.log"))
}

fn cleanup(path: &Path) -> anyhow::Result<()> {
    match std::fs::metadata(path) {
        Ok(m) => {
            if m.len() > LOG_FILE_MAX_SIZE_BYTES {
                std::fs::File::create(path).context("couldn't truncate log file")?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(e).context("couldn't get file metadata");
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn get_log_dir() -> anyhow::Result<PathBuf> {
    let strategy = choose_base_strategy()?;

    // XDG spec suggests using XDG_STATE_HOME for logs
    // https://specifications.freedesktop.org/basedir-spec/latest/#variables
    let log_dir = strategy
        .state_dir() // this always returns Some on unix, but adding a fallback regardless
        .map(|d| d.join("agent1"))
        .unwrap_or_else(|| strategy.home_dir().join(".agent1"));

    Ok(log_dir)
}

#[cfg(target_os = "windows")]
fn get_log_dir() -> anyhow::Result<PathBuf> {
    let strategy = choose_base_strategy()?;

    let log_dir = strategy.cache_dir().join("agent1");

    Ok(log_dir)
}
