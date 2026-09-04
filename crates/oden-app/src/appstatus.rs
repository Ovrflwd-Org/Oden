use chrono::{DateTime, Utc};
use gpui::{App, Global};
use uuid::Uuid;

pub(crate) struct AppStatus {
    pub(crate) issues: Vec<Issue>,
}

impl Global for AppStatus {}

#[allow(dead_code)]
pub(crate) struct Issue {
    pub(crate) id: Uuid,
    pub(crate) operation: AppOperation,
    pub(crate) message: String,
    pub(crate) issue_status: IssueStatus,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum AppOperation {
    CreateNewItem,
}

impl Issue {
    pub fn new(operation: AppOperation, message: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            operation,
            message,
            created_at: now,
            issue_status: IssueStatus::Open,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub(crate) enum IssueStatus {
    Open,
    Dismissed,
}

impl AppStatus {
    pub(crate) fn init(cx: &mut App) {
        cx.set_global(Self { issues: Vec::new() })
    }
}
