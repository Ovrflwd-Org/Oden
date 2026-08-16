use sea_orm::DbErr;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum DbError {
    #[error("could not determine app data directory")]
    MissingProjectDirs,

    #[error("could not create data directory")]
    CreateDir(#[source] std::io::Error),

    #[error("could not open sqlite db path")]
    OpenDbFile(#[source] std::io::Error),

    #[error("could not connect to database")]
    Connect(#[source] DbErr),

    #[error("invalid db path")]
    InvalidPath,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("could not determine app config directory")]
    MissingProjectDirs,

    #[error("could not create config directory")]
    CreateDir(#[source] std::io::Error),

    #[error("could not read settings file")]
    Read(#[source] std::io::Error),

    #[error("could not write settings file")]
    Write(#[source] std::io::Error),

    #[error("could not parse settings file")]
    Parse(#[source] serde_json::Error),

    #[error("could not serialize settings")]
    Serialize(#[source] serde_json::Error),
}
