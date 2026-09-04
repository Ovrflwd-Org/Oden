use std::sync::Arc;
use std::time::Duration;

use gpui::SharedString;
use gpui::Timer;
use oden_core::repository::ItemRepositoryTrait;
use tokio::sync::watch::Receiver;
use uuid::Uuid;

pub struct InputValueWatcher;
impl InputValueWatcher {
    pub fn spawn(
        mut rx: Receiver<SharedString>,
        id: Uuid,
        repository: Arc<dyn ItemRepositoryTrait + Send + Sync>,
    ) {
        tokio::spawn(async move {
            loop {
                if rx.changed().await.is_err() {
                    return;
                }

                loop {
                    tokio::select! {
                        _ = Timer::after(Duration::from_millis(4000)) => break,
                        changed = rx.changed() => {
                           if changed.is_err() {
                               return;
                           }
                           continue;
                        }
                    }
                }

                let content = rx.borrow_and_update().clone();
                if let Err(_e) = repository.update_item(id, content.to_string()).await {};
            }
        });
    }
}
