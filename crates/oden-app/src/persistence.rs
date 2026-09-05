use gpui::{App, Global};

use crate::persistence::PersistenceStatus::Idle;

pub enum PersistenceStatus {
    Saving,
    Failed,
    Idle,
}

impl PersistenceStatus {
    pub fn init(cx: &mut App) {
        cx.set_global::<PersistenceStatus>(Idle);
    }
}

impl Global for PersistenceStatus {}
