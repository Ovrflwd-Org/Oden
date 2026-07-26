use gpui::{
    Context, CursorStyle, Entity, ParentElement, Render, SharedString, Styled, Subscription, Window,
};
use gpui_component::{
    ActiveTheme, Icon, TitleBar,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
};

use crate::{
    appstatus::{AppStatus, IssueStatus},
    icons::IconName,
};

pub(crate) struct Titlebar {
    pub(crate) status_entity: Entity<AppStatus>,
    _status_entity_sub: Subscription,
}

impl Titlebar {
    pub(crate) fn new(
        cx: &mut Context<Self>,
        window: &mut Window,
        status_entity: Entity<AppStatus>,
    ) -> Self {
        let _status_entity_sub = cx.observe_in(
            &status_entity,
            window,
            |_this, _status_entity, _window, cx| {
                cx.notify();
            },
        );
        Self {
            status_entity,
            _status_entity_sub,
        }
    }
}

impl Render for Titlebar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let muted = cx.theme().muted_foreground;
        let green = cx.theme().green_light;
        let red = cx.theme().red_light;
        let total_issues_found: usize = self
            .status_entity
            .read(cx)
            .issues
            .iter()
            .filter(|issue| issue.issue_status == IssueStatus::Open)
            .count();
        let message = if let Some(issue) = self.status_entity.read(cx).issues.first() {
            issue.message.chars().take(50).collect::<String>()
        } else {
            String::new()
        };
        let issue_label = if total_issues_found == 1 {
            "issue"
        } else {
            "issues"
        };
        let status_message = format!("{} {} {}", total_issues_found, issue_label, message);
        let status_message = SharedString::from(status_message);
        let (icon_name, color) = if total_issues_found == 0 {
            (IconName::Check, green)
        } else {
            (IconName::Close, red)
        };
        TitleBar::new()
            // render the status of the app.
            .child(
                h_flex().gap_2().child(SharedString::from("Oden")).child(
                    Button::new("issues")
                        .cursor(CursorStyle::PointingHand)
                        // TODO: Navigate to a file showing all the issues.
                        .tooltip("check issue logs")
                        .h_3_4()
                        .ghost()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .items_center()
                        .child(Icon::new(icon_name).text_color(color))
                        .child(Label::new(status_message).text_color(muted)),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui::TestAppContext;
    use oden_core::entities::item;
    use oden_core::repository::ItemRepositoryTrait;
    use sea_orm::DbErr;

    use crate::{
        actions::NewItem, appstatus::AppOperation, repository::AppRepository, testutils::setup,
    };

    use async_trait::async_trait;

    pub struct FailingItemRepository {}

    #[async_trait]
    impl ItemRepositoryTrait for FailingItemRepository {
        async fn find_all(&self) -> Result<Vec<item::Model>, DbErr> {
            Ok(vec![])
        }

        async fn create_item(&self) -> Result<item::Model, DbErr> {
            Err(DbErr::Custom(
                "an error occurred when inserting an item".into(),
            ))
        }
    }

    #[gpui::test]
    fn test_titlebar_status_change_on_issues(cx: &mut TestAppContext) {
        let (window, _app_mode_state, _selected_id_state) = setup(cx);
        cx.update(|cx| {
            let failing_repository: Arc<dyn ItemRepositoryTrait + Send + Sync> =
                Arc::new(FailingItemRepository {});
            cx.set_global(AppRepository(failing_repository));
        });
        window
            .update(cx, |root, window, cx| {
                root.list_view.read(cx).focus_handle.focus(window);
                window.dispatch_action(Box::new(NewItem), cx);
            })
            .unwrap();
        cx.run_until_parked();
        window
            .update(cx, |root, _window, cx| {
                let status_entity = root.titlebar.read(cx).status_entity.clone();
                let issue = status_entity
                    .read(cx)
                    .issues
                    .first()
                    .expect("one issue should have been created");
                assert_eq!(issue.operation, AppOperation::CreateNewItem);
                assert_eq!(
                    issue.message,
                    "Custom Error: an error occurred when inserting an item"
                );
            })
            .unwrap();
    }
}
