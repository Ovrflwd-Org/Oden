#![allow(dead_code)]

use std::cell::RefCell;
use std::env;
use std::io::Read as _;
use std::rc::Rc;

use anyhow::anyhow;
use gpui::{
    App, AsyncApp, BorrowAppContext, Global, ParentElement, Styled, Subscription, WindowHandle,
    div, px,
};
use gpui_component::{Root, WindowExt as _, scroll::ScrollableElement as _};
use self_update::cargo_crate_version;

use crate::app_settings::AppSettingsState;

const REPO_OWNER: &str = "out-of-order";
const REPO_NAME: &str = "oden";
const BIN_NAME: &str = "oden";
const TAG_PREFIX: &str = "oden-v";

const FAKE_SERVER_ENV: &str = "ODEN_FAKE_UPDATE_SERVER";

fn fake_server_url() -> Option<String> {
    env::var(FAKE_SERVER_ENV).ok().filter(|s| !s.is_empty())
}

/// (filename prefix, filename suffix) identifying the self-update-eligible
/// release asset for this platform, e.g. `oden-linux-x64_0.8.0.AppImage`.
/// Must match the naming the release workflow uses
fn self_update_asset_hint() -> (&'static str, &'static str) {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return ("oden-win-x64_", ".zip");
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return ("oden-win-arm64_", ".zip");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return ("oden-macos-arm64_", ".tar.gz");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return ("oden-linux-x64_", ".AppImage");
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return ("oden-linux-arm64_", ".AppImage");
}

fn configure_builder(fake_url: Option<&str>) -> self_update::backends::github::UpdateBuilder {
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .target(self_update::get_target())
        .current_version(cargo_crate_version!());
    if let Some(url) = fake_url {
        builder.with_url(url);
    }
    builder
}

/// Finds the newest release strictly newer than the running
/// version, returning its plain semver version and full tag name.
fn latest_oden_release(fake_url: Option<&str>) -> anyhow::Result<Option<(String, String)>> {
    tracing::debug!(
        fake_mode = fake_url.is_some(),
        "resolving latest oden release"
    );
    if let Some(url) = fake_url {
        let update = configure_builder(Some(url)).build()?;
        let latest = update.get_latest_release()?;
        let is_newer =
            self_update::version::bump_is_greater(cargo_crate_version!(), &latest.version)?;
        return Ok(is_newer.then(|| {
            let tag = format!("v{}", latest.version);
            (latest.version, tag)
        }));
    }

    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()?
        .fetch()?;
    tracing::debug!(count = releases.len(), "fetched release list from github");

    let mut best: Option<String> = None;
    for release in &releases {
        let Some(version) = release.version.strip_prefix(TAG_PREFIX) else {
            continue;
        };
        if !self_update::version::bump_is_greater(cargo_crate_version!(), version).unwrap_or(false)
        {
            continue;
        }
        let is_newer_than_best = match &best {
            None => true,
            Some(current_best) => {
                self_update::version::bump_is_greater(current_best, version).unwrap_or(false)
            }
        };
        if is_newer_than_best {
            best = Some(version.to_string());
        }
    }
    Ok(best.map(|version| {
        let tag = format!("{TAG_PREFIX}{version}");
        (version, tag)
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateChannel {
    Portable,
    Packaged(PackageKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    Flatpak,
    Snap,
    System,
}

impl UpdateChannel {
    pub fn detect() -> Self {
        if env::var_os("FLATPAK_ID").is_some() {
            return UpdateChannel::Packaged(PackageKind::Flatpak);
        }
        if env::var_os("SNAP").is_some() {
            return UpdateChannel::Packaged(PackageKind::Snap);
        }
        if env::var_os("APPIMAGE").is_some() {
            return UpdateChannel::Portable;
        }
        if cfg!(target_os = "linux") {
            return UpdateChannel::Packaged(PackageKind::System);
        }
        UpdateChannel::Portable
    }

    pub fn allows_self_update(self) -> bool {
        matches!(self, UpdateChannel::Portable)
    }

    pub fn label(self) -> &'static str {
        match self {
            UpdateChannel::Portable => "Portable build",
            UpdateChannel::Packaged(PackageKind::Flatpak) => "Flatpak",
            UpdateChannel::Packaged(PackageKind::Snap) => "Snap",
            UpdateChannel::Packaged(PackageKind::System) => "System package",
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available { version: String },
    Installing { progress: f32 },
    Installed { version: String },
    Error(String),
}

pub struct UpdateState {
    pub status: UpdateStatus,
    pub channel: UpdateChannel,
}

impl Global for UpdateState {}

impl UpdateState {
    pub fn init(cx: &mut App) {
        cx.set_global(UpdateState {
            status: UpdateStatus::Idle,
            channel: UpdateChannel::detect(),
        });
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn release_url(version: &str) -> String {
        format!("https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/tag/{TAG_PREFIX}{version}")
    }
}

pub fn check_for_updates(cx: &mut App) {
    if matches!(UpdateState::get(cx).status, UpdateStatus::Checking) {
        tracing::debug!("check_for_updates called while already checking, ignoring");
        return;
    }
    tracing::info!("checking for updates");
    cx.update_global::<UpdateState, _>(|state, _| {
        state.status = UpdateStatus::Checking;
    });
    let fake_url = fake_server_url();

    cx.spawn(async move |cx| {
        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
            let found = latest_oden_release(fake_url.as_deref())?;
            Ok(found.map(|(version, _tag)| version))
        })
        .await;

        match &result {
            Ok(Ok(Some(version))) => tracing::info!(version, "update available"),
            Ok(Ok(None)) => tracing::info!("already up to date"),
            Ok(Err(err)) => tracing::warn!(error = ?err, "update check failed"),
            Err(join_err) => tracing::error!(error = ?join_err, "update check task panicked"),
        }

        let _ = cx.update(|cx| {
            cx.update_global::<UpdateState, _>(|state, _| {
                state.status = match result {
                    Ok(Ok(Some(version))) => UpdateStatus::Available { version },
                    Ok(Ok(None)) => UpdateStatus::UpToDate,
                    Ok(Err(err)) => UpdateStatus::Error(err.to_string()),
                    Err(join_err) => UpdateStatus::Error(join_err.to_string()),
                };
            });
        });
    })
    .detach();
}

pub fn install_update(cx: &mut App) {
    tracing::info!("starting update install");
    cx.update_global::<UpdateState, _>(|state, _| {
        state.status = UpdateStatus::Installing { progress: 0.0 };
    });
    let fake_url = fake_server_url();

    cx.spawn(async move |cx| {
        let started_at = std::time::Instant::now();
        let install = run_install(cx, fake_url).await;
        match &install {
            Ok(version) => tracing::info!(
                version,
                elapsed_ms = started_at.elapsed().as_millis() as u64,
                "update installed, restart required"
            ),
            Err(err) => tracing::error!(error = ?err, "update install failed"),
        }
        let _ = cx.update_global::<UpdateState, _>(|state, _| {
            state.status = match install {
                Ok(version) => UpdateStatus::Installed { version },
                Err(err) => UpdateStatus::Error(err.to_string()),
            };
        });
    })
    .detach();
}

async fn run_install(cx: &mut AsyncApp, fake_url: Option<String>) -> anyhow::Result<String> {
    let resolve_url = fake_url.clone();
    let (version, asset) = tokio::task::spawn_blocking(
        move || -> anyhow::Result<(String, self_update::update::ReleaseAsset)> {
            let (version, release) = if let Some(url) = resolve_url.as_deref() {
                let release = configure_builder(Some(url)).build()?.get_latest_release()?;
                let version = release.version.clone();
                (version, release)
            } else {
                let (version, tag) =
                    latest_oden_release(None)?.ok_or_else(|| anyhow!("no newer release found"))?;
                let release = configure_builder(None).build()?.get_release_version(&tag)?;
                (version, release)
            };
            let asset = if release.assets.len() == 1 {
                release.assets[0].clone()
            } else {
                let (name_prefix, name_suffix) = self_update_asset_hint();
                release
                    .assets
                    .iter()
                    .find(|a| a.name.starts_with(name_prefix) && a.name.ends_with(name_suffix))
                    .cloned()
                    .ok_or_else(|| {
                        anyhow!("no release asset found matching {name_prefix}*{name_suffix}")
                    })?
            };
            Ok((version, asset))
        },
    )
    .await??;
    tracing::debug!(version, asset = %asset.name, "resolved release asset");

    let download_url = asset.download_url.clone();
    let progress_cell = std::sync::Arc::new(std::sync::Mutex::new(0.0f32));
    let progress_for_thread = progress_cell.clone();

    tracing::debug!(url = %download_url, "starting download");
    let download_started_at = std::time::Instant::now();
    let download_task = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
        let client = reqwest::blocking::Client::builder().build()?;
        let mut resp = client
            .get(&download_url)
            .header(reqwest::header::ACCEPT, "application/octet-stream")
            .send()?
            .error_for_status()?;
        let total = resp.content_length().filter(|&t| t > 0);

        let mut archive_bytes = Vec::new();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let n = resp.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            archive_bytes.extend_from_slice(&chunk[..n]);

            if let Some(total) = total {
                let progress = (archive_bytes.len() as f32 / total as f32).min(1.0);
                if let Ok(mut cell) = progress_for_thread.lock() {
                    *cell = progress;
                }
            }
        }
        Ok(archive_bytes)
    });
    tokio::pin!(download_task);

    const PROGRESS_UPDATE_MIN_INTERVAL: std::time::Duration = std::time::Duration::from_millis(80);

    let archive_bytes = loop {
        cx.background_executor()
            .timer(PROGRESS_UPDATE_MIN_INTERVAL)
            .await;
        let progress = *progress_cell.lock().unwrap();
        let _ = cx.update_global::<UpdateState, _>(|state, _| {
            if let UpdateStatus::Installing { progress: p } = &mut state.status {
                *p = progress;
            }
        });
        if download_task.is_finished() {
            break (&mut download_task).await??;
        }
    };
    tracing::debug!(
        bytes = archive_bytes.len(),
        elapsed_ms = download_started_at.elapsed().as_millis() as u64,
        "download complete"
    );

    let asset_name = asset.name.clone();
    let is_fake = fake_url.is_some();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let tmp_dir = self_update::TempDir::new()?;
        let archive_path = tmp_dir.path().join(&asset_name);
        std::fs::write(&archive_path, &archive_bytes)?;

        let bin_name = format!("oden{}", std::env::consts::EXE_SUFFIX);
        self_update::Extract::from_source(&archive_path).extract_file(tmp_dir.path(), &bin_name)?;
        let new_exe = tmp_dir.path().join(&bin_name);

        let bin_install_path = if is_fake {
            env::temp_dir().join("oden-fake-update-install.bin")
        } else {
            std::env::current_exe()?
        };

        if bin_install_path == std::env::current_exe()? {
            tracing::debug!(path = %bin_install_path.display(), "self-replacing running binary");
            self_update::self_replace::self_replace(&new_exe)?;
        } else {
            tracing::debug!(path = %bin_install_path.display(), is_fake, "moving binary into place");
            self_update::Move::from_source(&new_exe).to_dest(&bin_install_path)?;
        }
        Ok(())
    })
    .await??;

    Ok(version)
}

pub fn restart_app(cx: &mut App) {
    cx.restart();
}

#[derive(Debug, Clone)]
pub enum ChangelogStatus {
    Loading,
    Loaded {
        version: String,
        body: Option<String>,
    },
    Error(String),
}

pub struct ChangelogState {
    pub status: ChangelogStatus,
}

impl Global for ChangelogState {}

impl ChangelogState {
    pub fn init(cx: &mut App) {
        cx.set_global(ChangelogState {
            status: ChangelogStatus::Loading,
        });
        load_changelog(cx);
    }

    pub fn init_default(cx: &mut App) {
        cx.set_global(ChangelogState {
            status: ChangelogStatus::Loading,
        });
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

fn load_changelog(cx: &mut App) {
    let fake_url = fake_server_url();
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    tracing::debug!(version = %current_version, "fetching changelog for the running version");

    cx.spawn(async move |cx| {
        let result =
            tokio::task::spawn_blocking(move || -> anyhow::Result<(String, Option<String>)> {
                let update = configure_builder(fake_url.as_deref()).build()?;
                let release = if fake_url.is_some() {
                    update.get_latest_release()?
                } else {
                    update.get_release_version(&format!("{TAG_PREFIX}{current_version}"))?
                };
                let version = release
                    .version
                    .strip_prefix(TAG_PREFIX)
                    .unwrap_or(&release.version)
                    .to_string();
                Ok((version, release.body))
            })
            .await;

        let status = match result {
            Ok(Ok((version, body))) => {
                tracing::info!(version, "changelog: loaded release notes");
                ChangelogStatus::Loaded { version, body }
            }
            Ok(Err(err)) => {
                tracing::warn!(error = ?err, "changelog: failed to fetch release notes");
                ChangelogStatus::Error(err.to_string())
            }
            Err(join_err) => {
                tracing::error!(error = ?join_err, "changelog: fetch task panicked");
                ChangelogStatus::Error(join_err.to_string())
            }
        };

        let _ = cx.update_global::<ChangelogState, _>(|state, _| {
            state.status = status;
        });
    })
    .detach();
}

pub fn show_whats_new_if_updated(window: WindowHandle<Root>, cx: &mut App) {
    let fake_url = fake_server_url();
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    if fake_url.is_none() {
        let last_seen = AppSettingsState::get(cx).last_seen_version.clone();
        if last_seen == current_version {
            return;
        }
        let is_first_run = last_seen.is_empty();
        AppSettingsState::update(cx, |settings| {
            settings.last_seen_version = current_version.clone();
        });
        if is_first_run {
            tracing::debug!("first run, skipping what's-new popup");
            return;
        }
    }

    tracing::info!(version = %current_version, "showing what's-new for updated version");
    show_whats_new(window, cx);
}

pub fn preview_whats_new(window: WindowHandle<Root>, cx: &mut App) {
    show_whats_new(window, cx);
}

fn show_whats_new(window: WindowHandle<Root>, cx: &mut App) {
    match ChangelogState::get(cx).status.clone() {
        ChangelogStatus::Loaded { version, body } => {
            open_whats_new_dialog(window, cx, version, body);
        }
        ChangelogStatus::Error(_) => {
            load_changelog(cx);
            show_whats_new_once_loaded(window, cx);
        }
        ChangelogStatus::Loading => {
            show_whats_new_once_loaded(window, cx);
        }
    }
}

fn show_whats_new_once_loaded(window: WindowHandle<Root>, cx: &mut App) {
    let subscription_slot: Rc<RefCell<Option<Subscription>>> = Rc::new(RefCell::new(None));
    let slot_for_callback = subscription_slot.clone();
    let subscription = cx.observe_global::<ChangelogState>(move |cx| {
        if let ChangelogStatus::Loaded { version, body } = ChangelogState::get(cx).status.clone() {
            open_whats_new_dialog(window, cx, version, body);
            slot_for_callback.borrow_mut().take();
        }
    });
    *subscription_slot.borrow_mut() = Some(subscription);
}

fn open_whats_new_dialog(
    window: WindowHandle<Root>,
    cx: &mut App,
    version: String,
    body: Option<String>,
) {
    cx.defer(move |cx| {
        let update_result = window.update(cx, move |_root, window, cx| {
            window.defer(cx, move |window, cx| {
                window.open_dialog(cx, move |dialog, window, cx| {
                    let content = body.clone().unwrap_or_else(|| {
                        "No release notes were found for this version.".to_string()
                    });
                    let max_content_h = window.viewport_size().height * 0.6;
                    dialog
                        .title(format!("What's new in {version}"))
                        .alert()
                        .w(px(560.))
                        .child(
                            div()
                                .w_full()
                                .max_h(max_content_h)
                                .overflow_y_scrollbar()
                                .child(comrak_gpui::render_document(&content, cx)),
                        )
                });
            });
        });
        if let Err(err) = update_result {
            tracing::error!(error = ?err, "what's-new: window.update failed");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_channels_never_allow_self_update() {
        assert!(!UpdateChannel::Packaged(PackageKind::Flatpak).allows_self_update());
        assert!(!UpdateChannel::Packaged(PackageKind::Snap).allows_self_update());
        assert!(!UpdateChannel::Packaged(PackageKind::System).allows_self_update());
    }

    #[test]
    fn portable_channel_allows_self_update() {
        assert!(UpdateChannel::Portable.allows_self_update());
    }

    #[test]
    fn release_url_points_at_tagged_release() {
        assert_eq!(
            UpdateState::release_url("1.2.3"),
            "https://github.com/out-of-order/oden/releases/tag/oden-v1.2.3"
        );
    }
}
