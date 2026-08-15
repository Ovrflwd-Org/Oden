use gpui::{App, BorrowAppContext, Global};
use oden_core::settings::AppSettings;

pub struct AppSettingsState(AppSettings);

impl Global for AppSettingsState {}

impl AppSettingsState {
    pub fn init(cx: &mut App) {
        let settings = AppSettings::load().unwrap_or_else(|err| {
            tracing::warn!(error = ?err, "failed to load settings, using defaults");
            AppSettings::default()
        });
        tracing::debug!(theme = %settings.theme, "settings loaded");
        cx.set_global(AppSettingsState(settings));
    }

    #[cfg(test)]
    pub fn init_default(cx: &mut App) {
        cx.set_global(AppSettingsState(AppSettings::default()));
    }

    pub fn get(cx: &App) -> &AppSettings {
        &cx.global::<AppSettingsState>().0
    }

    pub fn update(cx: &mut App, f: impl FnOnce(&mut AppSettings)) {
        cx.update_global::<AppSettingsState, _>(|state, _| {
            f(&mut state.0);
        });
        if let Err(err) = AppSettingsState::get(cx).save() {
            tracing::error!(error = ?err, "failed to save settings");
        }
    }
}
