use std::sync::Arc;

use gpui::{AsyncApp, Global};
use oden_core::repository::ItemRepositoryTrait;

pub struct AppRepository(pub Arc<dyn ItemRepositoryTrait + Send + Sync>);

impl Global for AppRepository {}

impl AppRepository {
    pub fn init(cx: &mut AsyncApp, repository: Arc<dyn ItemRepositoryTrait + Send + Sync>) {
        let repo = Self(repository);
        cx.update(move |cx| {
            cx.set_global(repo);
        });
    }
}
