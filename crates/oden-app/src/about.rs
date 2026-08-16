//! Fetches the repo's top contributors from the GitHub API

use std::sync::Arc;

use gpui::{App, Global, Image, ImageFormat};
use serde::Deserialize;

pub const REPO_OWNER: &str = "out-of-order";
pub const REPO_NAME: &str = "oden";

#[derive(Debug, Clone, Deserialize)]
pub struct Contributor {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
    pub contributions: u32,
    #[serde(skip)]
    pub avatar: Option<Arc<Image>>,
}

#[derive(Debug, Clone, Default)]
pub enum ContributorsStatus {
    #[default]
    Loading,
    Loaded(Vec<Contributor>),
    Error(String),
}

pub struct AboutState {
    pub contributors: ContributorsStatus,
}

impl Global for AboutState {}

impl AboutState {
    pub fn init(cx: &mut App) {
        cx.set_global(AboutState {
            contributors: ContributorsStatus::Loading,
        });
        load_contributors(cx);
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

async fn fetch_contributors() -> anyhow::Result<Vec<Contributor>> {
    let url =
        format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/contributors?per_page=12");
    let client = reqwest::Client::builder()
        .user_agent(format!("oden/{}", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut contributors = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Contributor>>()
        .await?;

    for contributor in &mut contributors {
        contributor.avatar = fetch_avatar(&client, &contributor.avatar_url).await;
    }

    Ok(contributors)
}

fn load_contributors(cx: &mut App) {
    tracing::debug!("fetching repo contributors");

    cx.spawn(async move |cx| {
        let result = tokio::spawn(fetch_contributors()).await;

        let status = match result {
            Ok(Ok(contributors)) => {
                tracing::info!(count = contributors.len(), "loaded contributors");
                ContributorsStatus::Loaded(contributors)
            }
            Ok(Err(err)) => {
                tracing::warn!(error = ?err, "failed to load contributors");
                ContributorsStatus::Error(err.to_string())
            }
            Err(join_err) => {
                tracing::error!(error = ?join_err, "contributors fetch task panicked");
                ContributorsStatus::Error(join_err.to_string())
            }
        };

        let _ = cx.update_global::<AboutState, _>(|state, _| {
            state.contributors = status;
        });
    })
    .detach();
}

async fn fetch_avatar(client: &reqwest::Client, avatar_url: &str) -> Option<Arc<Image>> {
    let response = client
        .get(avatar_url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let format = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(image_format_from_mime_type)
        .unwrap_or(ImageFormat::Jpeg);
    let bytes = response.bytes().await.ok()?.to_vec();
    Some(Arc::new(Image::from_bytes(format, bytes)))
}

fn image_format_from_mime_type(mime_type: &str) -> Option<ImageFormat> {
    match mime_type.split(';').next()?.trim() {
        "image/png" => Some(ImageFormat::Png),
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/webp" => Some(ImageFormat::Webp),
        "image/gif" => Some(ImageFormat::Gif),
        "image/bmp" => Some(ImageFormat::Bmp),
        _ => None,
    }
}
