use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait,
    sea_query::value::prelude::serde_json::json,
};
use uuid::Uuid;

use async_trait::async_trait;

use crate::entities::item;

#[async_trait]
pub trait ItemRepositoryTrait {
    async fn find_all(&self) -> Result<Vec<item::Model>, DbErr>;
    async fn create_item(&self) -> Result<item::Model, DbErr>;
}

pub struct ItemRepository {
    db: DatabaseConnection,
}

#[cfg(test)]
pub struct MockItemRepository {}

#[async_trait]
#[cfg(test)]
impl ItemRepositoryTrait for MockItemRepository {
    async fn find_all(&self) -> Result<Vec<item::Model>, DbErr> {
        Ok(vec![])
    }
    async fn create_item(&self) -> Result<item::Model, DbErr> {
        let now = Utc::now();
        Ok(item::Model {
            id: Uuid::from_u128(1),
            name: "Untitled".to_string(),
            content: "# Untitled".to_string(),
            kind: item::ItemKind::Note,
            tags: json!([]),
            language: None,
            created_at: now,
            modified_at: now,
        })
    }
}

impl ItemRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ItemRepositoryTrait for ItemRepository {
    #[tracing::instrument(skip(self))]
    async fn find_all(&self) -> Result<Vec<item::Model>, DbErr> {
        let result = item::Entity::find().all(&self.db).await;
        match &result {
            Ok(items) => tracing::debug!(count = items.len(), "find_all succeeded"),
            Err(err) => tracing::error!(error = ?err, "find_all failed"),
        }
        result
    }

    #[tracing::instrument(skip(self))]
    async fn create_item(&self) -> Result<item::Model, DbErr> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let item_instance = item::ActiveModel {
            id: Set(id),
            name: Set("Untitled".to_string()),
            content: Set("# Untitled".to_string()),
            kind: Set(item::ItemKind::Note),
            tags: Set(json!([])),
            language: Set(None),
            created_at: Set(now),
            modified_at: Set(now),
        };
        let result = item_instance.insert(&self.db).await;
        if let Err(err) = &result {
            tracing::error!(error = ?err, item_id = %id, "create_item failed");
        }
        result
    }
}
