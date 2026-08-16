use gpui::{
    Context, CursorStyle, Entity, InteractiveElement, MouseButton, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme, Icon, IconName as UiIconName, InteractiveElementExt, Sizable, TITLE_BAR_HEIGHT,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
};

use crate::{
    appstatus::{AppStatus, IssueStatus},
    icons::IconName,
};

#[cfg(target_os = "macos")]
const TITLE_BAR_LEFT_PADDING: gpui::Pixels = gpui::px(80.);
#[cfg(not(target_os = "macos"))]
const TITLE_BAR_LEFT_PADDING: gpui::Pixels = gpui::px(12.);

#[cfg(target_os = "windows")]
mod win32 {
    use gpui::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows::Win32::UI::WindowsAndMessaging::{
        HTCAPTION, PostMessageW, SHOW_WINDOW_CMD, ShowWindow, WM_NCLBUTTONDOWN,
    };

    fn hwnd(window: &Window) -> Option<HWND> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as *mut _)),
            _ => None,
        }
    }

    pub(super) fn start_drag(window: &Window) {
        let Some(hwnd) = hwnd(window) else { return };
        unsafe {
            let _ = ReleaseCapture();
            let _ = PostMessageW(
                Some(hwnd),
                WM_NCLBUTTONDOWN,
                WPARAM(HTCAPTION as usize),
                LPARAM(0),
            );
        }
    }

    pub(super) fn restore(window: &Window) {
        const SW_RESTORE: SHOW_WINDOW_CMD = SHOW_WINDOW_CMD(9);
        let Some(hwnd) = hwnd(window) else { return };
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
    }
}

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
        window: &mut gpui::Window,
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

        let is_maximized = window.is_maximized();

        h_flex()
            .id("title-bar")
            .flex_shrink_0()
            .w_full()
            .h(TITLE_BAR_HEIGHT)
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(cx.theme().title_bar_border)
            .bg(cx.theme().title_bar)
            .pl(TITLE_BAR_LEFT_PADDING)
            // The status/issues indicator on the left
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
            .child(
                div()
                    .id("titlebar-drag-region")
                    .flex_1()
                    .h_full()
                    .on_mouse_down(MouseButton::Left, {
                        move |_, window, _cx| {
                            #[cfg(target_os = "windows")]
                            win32::start_drag(window);
                            #[cfg(not(target_os = "windows"))]
                            window.start_window_move();
                        }
                    })
                    .on_double_click(move |_, window, _cx| {
                        if is_maximized {
                            #[cfg(target_os = "windows")]
                            win32::restore(window);
                            #[cfg(not(target_os = "windows"))]
                            window.zoom_window();
                        } else {
                            window.zoom_window();
                        }
                    }),
            )
            .child(window_controls(is_maximized, cx))
    }
}

fn window_control_button(
    id: &'static str,
    icon: UiIconName,
    danger: bool,
    cx: &gpui::App,
    on_click: impl Fn(&mut Window, &mut gpui::App) + 'static,
) -> impl gpui::IntoElement {
    let hover_bg = if danger {
        cx.theme().danger
    } else {
        cx.theme().secondary_hover
    };
    let active_bg = if danger {
        cx.theme().danger_active
    } else {
        cx.theme().secondary_active
    };
    let hover_fg = if danger {
        cx.theme().danger_foreground
    } else {
        cx.theme().secondary_foreground
    };

    div()
        .id(id)
        .flex()
        .w(TITLE_BAR_HEIGHT)
        .h_full()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .text_color(cx.theme().foreground)
        .cursor(CursorStyle::PointingHand)
        .hover(|style| style.bg(hover_bg).text_color(hover_fg))
        .active(|style| style.bg(active_bg).text_color(hover_fg))
        .on_mouse_down(MouseButton::Left, |_, _window, cx| cx.stop_propagation())
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(Icon::new(icon).small())
}

fn window_controls(is_maximized: bool, cx: &gpui::App) -> impl gpui::IntoElement {
    if cfg!(target_os = "macos") {
        return h_flex().id("window-controls");
    }

    h_flex()
        .id("window-controls")
        .items_center()
        .flex_shrink_0()
        .h(TITLE_BAR_HEIGHT)
        .child(window_control_button(
            "win-minimize",
            UiIconName::WindowMinimize,
            false,
            cx,
            |window, _cx| window.minimize_window(),
        ))
        .child(window_control_button(
            "win-maximize",
            if is_maximized {
                UiIconName::WindowRestore
            } else {
                UiIconName::WindowMaximize
            },
            false,
            cx,
            move |window, _cx| {
                if is_maximized {
                    #[cfg(target_os = "windows")]
                    win32::restore(window);
                    #[cfg(not(target_os = "windows"))]
                    window.zoom_window();
                } else {
                    window.zoom_window();
                }
            },
        ))
        .child(window_control_button(
            "win-close",
            UiIconName::WindowClose,
            true,
            cx,
            |window, _cx| window.remove_window(),
        ))
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
