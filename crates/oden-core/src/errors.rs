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
pub enum UpdateItemError {
    #[error("saving is broken")]
    NotFound,
    #[error("saving is broken")]
    Db(#[from] sea_orm::DbErr),
}
