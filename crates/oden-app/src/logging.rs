//! Global `tracing` setup for the whole app.

use std::path::PathBuf;

use directories::ProjectDirs;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_FILTER: &str = "warn,oden=debug,oden_core=debug,oden_graph=debug,comrak_gpui=debug";

fn log_dir() -> Option<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "outoforder", "oden")?;
    Some(project_dirs.data_dir().join("logs"))
}

pub fn init() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let console_layer = fmt::layer().with_target(true);

    let (file_layer, guard) = match log_dir() {
        Some(dir) => match std::fs::create_dir_all(&dir) {
            Ok(()) => {
                let file_appender = tracing_appender::rolling::daily(&dir, "oden.log");
                let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
                let layer = fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(true);
                (Some(layer), Some(guard))
            }
            Err(err) => {
                eprintln!("failed to create log directory {}: {err}", dir.display());
                (None, None)
            }
        },
        None => (None, None),
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    if let Some(dir) = log_dir() {
        tracing::info!(path = %dir.display(), "file logging active");
    }

    guard
}
