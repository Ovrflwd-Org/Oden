use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use directories::ProjectDirs;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection, EntityTrait,
    sea_query::value::prelude::serde_json::json,
};
use url::Url;
use uuid::Uuid;

use crate::{entities::item, errors::DbError};

// returns the path to the file on disk for the sqlite database.
fn get_db_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("com", "outoforder", "oden").ok_or(DbError::MissingProjectDirs)?;
    // linux: ~/.local/share/oden
    // macOS: ~/Library/Application Support/com.outoforder.oden
    // windows: %LOCALAPPDATA%\\outoforder\\data
    let data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir).map_err(DbError::CreateDir)?;
    Ok(data_dir.join("oden.db"))
}

#[cfg(debug_assertions)]
// seeds the local database with mock data in debug mode.
async fn seed_data_db(db: &DatabaseConnection) -> Result<()> {
    tracing::debug!("seeding database with mock data (debug build)");
    item::Entity::delete_many().exec(db).await?;
    let now = Utc::now();
    let rows = vec![
        item::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Start docker compose".into()),
            content: Set("docker compose up -d".into()),
            kind: Set(item::ItemKind::Command),
            tags: Set(json!(["docker", "devops"])),
            language: Set(None),
            created_at: Set(now),
            modified_at: Set(now),
        },
        item::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Axum handler".into()),
            content: Set(
                r#"async fn handler(State(db): State<DbPool>) -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}"#
                .into(),
            ),
            kind: Set(item::ItemKind::Snippet),
            tags: Set(json!(["rust", "axum"])),
            language: Set(Some("rust".into())),
            created_at: Set(now),
            modified_at: Set(now),
        },
        item::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Architecture notes".into()),
            content: Set("# Architecture\n\nThe app uses GPUI for rendering...".into()),
            kind: Set(item::ItemKind::Note),
            tags: Set(json!(["notes"])),
            language: Set(None),
            created_at: Set(now),
            modified_at: Set(now),
        },
    ];
    let row_count = rows.len();
    for row in rows {
        row.insert(db).await?;
    }
    tracing::debug!(count = row_count, "seeded database rows");
    Ok(())
}

// returns a sea_orm database connection.
pub async fn setup_database() -> Result<DatabaseConnection> {
    let db_path = get_db_path()?;
    tracing::info!(path = %db_path.display(), "opening database");
    if !db_path.exists() {
        tracing::debug!("database file does not exist yet, creating it");
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&db_path)
            .map_err(DbError::OpenDbFile)?;
    }
    let url = Url::from_file_path(db_path).map_err(|_| DbError::InvalidPath)?;
    let db: DatabaseConnection = Database::connect(format!("sqlite://{}", url.path()))
        .await
        .map_err(DbError::Connect)?;
    db.get_schema_registry(module_path!().split("::").next().unwrap());
    db.get_schema_builder()
        .register(item::Entity)
        .sync(&db)
        .await?;
    tracing::debug!("schema sync complete");

    #[cfg(debug_assertions)]
    seed_data_db(&db).await?;

    tracing::info!("database ready");
    Ok(db)
}
