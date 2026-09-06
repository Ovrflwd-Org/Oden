use std::sync::Arc;
use std::time::Duration;

use gpui::SharedString;
use oden_core::errors::UpdateItemError;
use oden_core::repository::ItemRepositoryTrait;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;
use uuid::Uuid;

use crate::persistence::PersistenceStatus;

pub struct InputValueWatcher {}

impl InputValueWatcher {
    pub fn spawn(
        mut rx: Receiver<SharedString>,
        error_tx: UnboundedSender<UpdateItemError>,
        persistence_state_tx: UnboundedSender<PersistenceStatus>,
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
                        _ = tokio::time::sleep(Duration::from_millis(1000)) => break,
                        changed = rx.changed() => {
                           if changed.is_err() {
                               return;
                           }
                           continue;
                        }
                    }
                }
                if persistence_state_tx
                    .send(PersistenceStatus::Saving)
                    .is_err()
                {
                    return;
                };
                let content = rx.borrow_and_update().clone();
                if let Err(e) = repository.update_item(id, content.to_string()).await {
                    if persistence_state_tx
                        .send(PersistenceStatus::Failed)
                        .is_err()
                        || error_tx.send(e).is_err()
                    {
                        return;
                    }
                } else {
                    if persistence_state_tx.send(PersistenceStatus::Idle).is_err() {
                        return;
                    };
                };
            }
        });
    }
}
