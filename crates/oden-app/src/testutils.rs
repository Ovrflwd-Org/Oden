use gpui::{AppContext, Entity, TestAppContext, WindowHandle, WindowOptions};

use crate::app_settings::AppSettingsState;
use crate::root::AppRoot;
use crate::state::{AppMode, SelectedIdState};
use crate::store::ItemStore;
use crate::updater::UpdateState;

pub fn setup(
    cx: &mut TestAppContext,
) -> (
    WindowHandle<AppRoot>,
    gpui::Entity<AppMode>,
    gpui::Entity<SelectedIdState>,
) {
    cx.update(|cx| {
        gpui_component::init(cx);
        AppSettingsState::init_default(cx);
        UpdateState::init(cx);
        ItemStore::mock_store(cx);
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
    })
}
