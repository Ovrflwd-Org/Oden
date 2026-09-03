use std::time::Duration;

use gpui::SharedString;
use gpui::Timer;
use tokio::sync::watch::Receiver;
pub struct InputValueWatcher {}

impl InputValueWatcher {
    pub fn spawn(mut rx: Receiver<SharedString>) {
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
                // TODO: persist to DB.
            }
        });
    }
}
