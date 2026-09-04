use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait,
    sea_query::value::prelude::serde_json::json,
};
use uuid::Uuid;

use async_trait::async_trait;

use crate::{entities::item, errors::UpdateItemError};

#[async_trait]
pub trait ItemRepositoryTrait {
    async fn find_all(&self) -> Result<Vec<item::Model>, DbErr>;
    async fn create_item(&self) -> Result<item::Model, DbErr>;
    async fn update_item(&self, id: Uuid, content: String) -> Result<(), UpdateItemError>;
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

    async fn update_item(&self, _id: Uuid, _content: String) -> Result<(), UpdateItemError> {
        Ok(())
    }
}

impl ItemRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ItemRepositoryTrait for ItemRepository {
    async fn find_all(&self) -> Result<Vec<item::Model>, DbErr> {
        item::Entity::find().all(&self.db).await
    }

    async fn create_item(&self) -> Result<item::Model, DbErr> {
        let now = Utc::now();
        let item_instance = item::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("Untitled".to_string()),
            content: Set("# Untitled".to_string()),
            kind: Set(item::ItemKind::Note),
            tags: Set(json!([])),
            language: Set(None),
            created_at: Set(now),
            modified_at: Set(now),
        };
        item_instance.insert(&self.db).await
    }

    async fn update_item(&self, id: Uuid, content: String) -> Result<(), UpdateItemError> {
        let item_maybe = item::Entity::find_by_id(id).one(&self.db).await?;
        if let Some(item) = item_maybe {
            let mut item: item::ActiveModel = item.into();
            item.content = Set(content);
        } else {
            return Err(UpdateItemError::NotFound);
        }
        Ok(())
    }
}
