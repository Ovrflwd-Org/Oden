use gpui::{AppContext, Entity, TestAppContext, WindowHandle, WindowOptions};
use std::sync::OnceLock;

use crate::persistence::PersistencePerNote;
use crate::root::AppRoot;
use crate::state::{AppMode, SelectedIdState};
use crate::store::ItemStore;

fn enter_tokio_runtime() -> tokio::runtime::EnterGuard<'static> {
    static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    TOKIO_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime should initialize in tests")
        })
        .enter()
}

pub fn setup(
    cx: &mut TestAppContext,
) -> (
    WindowHandle<AppRoot>,
    gpui::Entity<AppMode>,
    gpui::Entity<SelectedIdState>,
    tokio::runtime::EnterGuard<'static>,
) {
    let tokio_guard = enter_tokio_runtime();
    let (window, app_mode_state, selected_id_state) = cx.update(|cx| {
        gpui_component::init(cx);
        ItemStore::mock_store(cx);
        PersistencePerNote::init(cx);
        crate::appstatus::AppStatus::init(cx);
        let window = cx
            .open_window(WindowOptions::default(), |window, cx| {
                let selected_id_state: Entity<SelectedIdState> =
                    cx.new(|_| SelectedIdState::init());
                let app_mode_state: Entity<AppMode> = cx.new(|_| AppMode::List);
                cx.new(|cx| AppRoot::new(app_mode_state, selected_id_state, window, cx))
            })
            .unwrap();
        let app_mode_state = window.root(cx).unwrap().read(cx).app_mode.clone();
        let selected_id_state = window.root(cx).unwrap().read(cx).selected_id_state.clone();
        (window, app_mode_state, selected_id_state)
    });
    (window, app_mode_state, selected_id_state, tokio_guard)
}
