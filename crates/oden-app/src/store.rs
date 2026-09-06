#![allow(dead_code)]

use std::collections::HashMap;

use gpui::{App, AsyncApp, Global, SharedString};
use gpui_component::link::Link;
use oden_core::entities::item;
use oden_core::repository::{ItemRepository, ItemRepositoryTrait};
use tokio::sync::watch::Sender;
use uuid::Uuid;

#[cfg(any(test, debug_assertions))]
use crate::fixtures::mock_items;
use crate::models::Item;
pub struct ItemStore {
    pub items: HashMap<uuid::Uuid, Item>,
    pub watch_tx: HashMap<Uuid, Sender<SharedString>>,
    links: Vec<Link>,
}

impl Global for ItemStore {}

impl ItemStore {
    pub fn items(&self) -> HashMap<uuid::Uuid, Item> {
        self.items.clone()
    }

    #[cfg(any(debug_assertions, test))]
    pub fn mock_store(cx: &mut App) {
        let mut store = ItemStore {
            items: HashMap::new(),
            watch_tx: HashMap::new(),
            links: Vec::new(),
        };
        for item in mock_items() {
            store.items.insert(item.id, item);
        }
        cx.set_global(store);
    }

    pub async fn init(cx: &mut AsyncApp, repository: &ItemRepository) -> anyhow::Result<()> {
        let items: Vec<item::Model> = repository.find_all().await?;
        cx.update(move |cx| {
            let mut store = ItemStore {
                items: HashMap::new(),
                watch_tx: HashMap::new(),
                links: Vec::new(),
            };
            for item in items {
                store.items.insert(item.id, Item::from(item));
            }
            cx.set_global(store);
        });
        Ok(())
    }

    pub fn get(cx: &mut App) -> &Self {
        cx.global::<ItemStore>()
    }

    pub fn get_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<ItemStore>()
    }
}
