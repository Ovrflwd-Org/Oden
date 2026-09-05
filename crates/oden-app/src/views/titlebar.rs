use gpui::{
    AnyElement, Context, CursorStyle, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme, Icon, TitleBar,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    spinner::Spinner,
};

use crate::{
    appstatus::{AppStatus, IssueStatus},
    icons::IconName,
    persistence::PersistenceStatus::{self, Failed, Idle, Saving},
};

pub(crate) struct Titlebar {
    _status_entity_sub: Subscription,
}

impl Titlebar {
    pub(crate) fn new(cx: &mut Context<Self>, window: &mut Window) -> Self {
        let _status_entity_sub =
            cx.observe_global_in::<AppStatus>(window, |_status_entity, _window, cx| {
                cx.notify();
            });
        Self { _status_entity_sub }
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
        let status_entity = cx.global::<AppStatus>();
        let total_issues_found: usize = status_entity
            .issues
            .iter()
            .filter(|issue| issue.1.issue_status == IssueStatus::Open)
            .count();
        let issue_label = if total_issues_found == 1 {
            "issue"
        } else {
            "issues"
        };
        let status_message = format!("{} {}", total_issues_found, issue_label);
        let status_message = SharedString::from(status_message);
        let (icon_name, color) = if total_issues_found == 0 {
            (IconName::Check, green)
        } else {
            (IconName::Close, red)
        };
        TitleBar::new()
            // render the status of the app.
            .pr_4()
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
            .child(render_persistence(cx))
    }
}

fn render_persistence(cx: &gpui::prelude::Context<Titlebar>) -> AnyElement {
    let persistence = cx.global::<PersistenceStatus>();
    let red = cx.theme().red_light;
    let muted = cx.theme().muted_foreground;
    match persistence {
        Idle => Label::new("synced").text_color(muted).into_any_element(),
        Saving => h_flex()
            .child(Spinner::new().color(muted))
            .child(Label::new("syncing").text_color(muted))
            .text_color(muted)
            .gap_1()
            .items_center()
            .into_any_element(),
        Failed => h_flex()
            .child(Icon::new(IconName::Close).text_color(red))
            .child(
                div()
                    .child(Label::new("out of sync").text_color(muted))
                    .text_color(muted),
            )
            .gap_1()
            .items_center()
            .into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui::TestAppContext;
    use oden_core::repository::ItemRepositoryTrait;
    use oden_core::{entities::item, errors::UpdateItemError};
    use sea_orm::DbErr;
    use uuid::Uuid;

    use crate::appstatus::AppStatus;
    use crate::{actions::NewItem, repository::AppRepository, testutils::setup};

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

        async fn update_item(&self, _id: Uuid, _content: String) -> Result<(), UpdateItemError> {
            Ok(())
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
            .update(cx, |_root, _window, cx| {
                let status_entity = cx.global::<AppStatus>();
                let issue = status_entity
                    .issues
                    .values()
                    .next()
                    .expect("one issue should have been created");
                assert_eq!(
                    issue.message,
                    "Custom Error: an error occurred when inserting an item"
                );
            })
            .unwrap();
    }
}
