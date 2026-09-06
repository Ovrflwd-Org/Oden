use std::sync::Arc;
use std::{borrow::Cow, path::PathBuf};

use anyhow::anyhow;
use gpui::{App, AppContext, AssetSource, Entity, Result, SharedString, WindowOptions};
use gpui_component::{Root, Theme, ThemeRegistry, TitleBar};
use oden_core::repository::ItemRepository;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "icons/**/*.svg"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }
        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

use crate::appstatus::AppStatus;
use crate::persistence::PersistencePerNote;
use crate::repository::AppRepository;
use crate::state::{AppMode, SelectedIdState};
use crate::{root::AppRoot, store::ItemStore};
use oden_core::db::setup_database;

mod actions;
mod appstatus;
#[cfg(debug_assertions)]
mod fixtures;
mod icons;
mod inputvaluewatcher;
mod models;
mod persistence;
mod repository;
mod root;
mod state;
mod store;
#[cfg(test)]
mod testutils;
mod views;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = setup_database().await?;
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            let _ = cx.text_system().add_fonts(vec![Cow::Borrowed(
                include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf").as_slice(),
            )]);
            gpui_component::init(cx);
            AppStatus::init(cx);
            PersistencePerNote::init(cx);
            setup_theme(cx);
            let window_options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                ..Default::default()
            };
            cx.spawn(async move |cx| {
                let repository = Arc::new(ItemRepository::new(db));
                if let Err(err) = ItemStore::init(cx, &repository).await {
                    eprintln!("failed to initialize ItemStore: {err:?}");
                    return;
                }
                AppRepository::init(cx, repository);
                cx.open_window(window_options, |window, cx| {
                    let app_mode: Entity<AppMode> = cx.new(|_| AppMode::List);
                    let selected_id_state: Entity<SelectedIdState> =
                        cx.new(|_| SelectedIdState::init());
                    let view = cx.new(|cx| AppRoot::new(app_mode, selected_id_state, window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .unwrap();
            })
            .detach();
        });
    Ok(())
}

fn setup_theme(cx: &mut App) {
    let theme_name = SharedString::from("Gruvbox Dark");
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme);
        }
    }) {
        println!("there was an error loading the theme {:?}", err)
    }
}
