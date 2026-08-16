use gpui::{
    App, Context, Corner, FontWeight, ImageSource, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, Styled, Subscription, Window, div, prelude::FluentBuilder as _, px,
    relative,
};
use gpui_component::ThemeRegistry;
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName as UiIconName, Sizable, Size,
    avatar::Avatar,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    menu::{DropdownMenu, PopupMenuItem},
    setting::{RenderOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    v_flex,
};

use crate::{
    about::{AboutState, ContributorsStatus},
    app_settings::AppSettingsState,
    updater::{self, ChangelogState, ChangelogStatus, UpdateState, UpdateStatus},
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = "https://github.com/out-of-order/oden";

pub(crate) struct SettingsView {
    _settings_sub: Subscription,
    _update_sub: Subscription,
    _about_sub: Subscription,
    _changelog_sub: Subscription,
}

impl SettingsView {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let _settings_sub = cx.observe_global::<AppSettingsState>(|_this, cx| cx.notify());
        let _update_sub = cx.observe_global::<UpdateState>(|_this, cx| cx.notify());
        let _about_sub = cx.observe_global::<AboutState>(|_this, cx| cx.notify());
        let _changelog_sub = cx.observe_global::<ChangelogState>(|_this, cx| cx.notify());
        Self {
            _settings_sub,
            _update_sub,
            _about_sub,
            _changelog_sub,
        }
    }
}

fn theme_names(cx: &App) -> Vec<SharedString> {
    ThemeRegistry::global(cx)
        .sorted_themes()
        .into_iter()
        .map(|theme| theme.name.clone())
        .collect()
}

fn theme_dropdown_field() -> SettingField<SharedString> {
    SettingField::render(move |_options, _window, cx| {
        let names = theme_names(cx);
        let current = AppSettingsState::get(cx).theme.clone();
        let current_label = names
            .iter()
            .find(|name| name.as_ref() == current)
            .cloned()
            .unwrap_or_else(|| SharedString::from(current.clone()));

        Button::new("theme-dropdown")
            .label(current_label)
            .dropdown_caret(true)
            .outline()
            .dropdown_menu_with_anchor(Corner::TopRight, move |menu, _, _| {
                let current = current.clone();
                names
                    .iter()
                    .fold(menu.scrollable(true).max_h(px(320.)), |menu, name| {
                        let checked = name.as_ref() == current;
                        let name = name.clone();
                        menu.item(PopupMenuItem::new(name.clone()).checked(checked).on_click(
                            move |_, _, cx| {
                                AppSettingsState::update(cx, |settings| {
                                    settings.theme = name.to_string();
                                });
                                crate::apply_theme(&name, cx);
                            },
                        ))
                    })
            })
    })
}

fn general_page() -> SettingPage {
    SettingPage::new("General")
        .default_open(true)
        .group(
            SettingGroup::new().title("Appearance").item(
                SettingItem::new("Theme", theme_dropdown_field())
                    .description("Pick a color theme for the interface."),
            ),
        )
        .group(
            SettingGroup::new()
                .title("Updates")
                .item(
                    SettingItem::new(
                        "Check for updates automatically",
                        SettingField::switch(
                            |cx| AppSettingsState::get(cx).check_for_updates,
                            |value, cx| {
                                AppSettingsState::update(cx, |settings| {
                                    settings.check_for_updates = value;
                                });
                            },
                        ),
                    )
                    .description("Look for a new release on startup."),
                )
                .item(SettingItem::render(render_update_panel)),
        )
}

fn about_page() -> SettingPage {
    SettingPage::new("About")
        .resettable(false)
        .group(SettingGroup::new().item(SettingItem::render(render_about_panel)))
}

fn install_progress_button(button: Button, progress: Option<f32>, cx: &App) -> gpui::AnyElement {
    div()
        .w(px(190.))
        .relative()
        .rounded(cx.theme().radius)
        .overflow_hidden()
        .child(button)
        .when_some(progress, |this, progress| {
            this.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .bottom_0()
                    .w(relative(progress.clamp(0., 1.)))
                    .bg(cx.theme().secondary.opacity(0.35)),
            )
        })
        .into_any_element()
}

fn render_update_panel(
    _options: &RenderOptions,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let update_state = UpdateState::get(cx);
    let channel = update_state.channel;
    let status = update_state.status.clone();
    let muted = cx.theme().muted_foreground;
    let green = cx.theme().green_light;
    let red = cx.theme().red_light;

    let is_checking = matches!(status, UpdateStatus::Checking);

    let status_row: Option<gpui::AnyElement> = match status.clone() {
        UpdateStatus::Idle => Some(
            Label::new("Not checked yet.")
                .text_color(muted)
                .into_any_element(),
        ),
        UpdateStatus::Checking => Some(
            Label::new("Checking for updates...")
                .text_color(muted)
                .into_any_element(),
        ),
        UpdateStatus::UpToDate => Some(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(UiIconName::CircleCheck).text_color(green))
                .child(Label::new("You're up to date."))
                .into_any_element(),
        ),
        UpdateStatus::Available { version } => Some(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Icon::new(UiIconName::Bell).text_color(green))
                        .child(Label::new(format!("Oden v{version} is available."))),
                )
                .when(!channel.allows_self_update(), |this| {
                    this.child(
                        Label::new(format!(
                            "Update through {} to get this release.",
                            channel.label()
                        ))
                        .text_color(muted),
                    )
                })
                .into_any_element(),
        ),
        UpdateStatus::Installing { .. } => Some(
            Label::new("Downloading the update...")
                .text_color(muted)
                .into_any_element(),
        ),
        UpdateStatus::Installed { version } => Some(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(UiIconName::CircleCheck).text_color(green))
                .child(Label::new(format!(
                    "Updated to v{version}. Restart to finish."
                )))
                .into_any_element(),
        ),
        UpdateStatus::Error(message) => Some(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(UiIconName::TriangleAlert).text_color(red))
                .child(Label::new(message).text_color(red))
                .into_any_element(),
        ),
    };

    let primary_button = match status {
        UpdateStatus::Available { version } if channel.allows_self_update() => {
            let button = Button::new("install-update")
                .primary()
                .tooltip(format!("Download and install v{version}"))
                .label("Install & Restart")
                .w_full()
                .on_click(move |_, _, cx| updater::install_update(cx));

            install_progress_button(button, None, cx)
        }
        UpdateStatus::Installing { progress } => {
            let button = Button::new("install-update")
                .primary()
                .loading(true)
                .tooltip("Downloading the update...")
                .label(format!(
                    "Installing... {}%",
                    (progress * 100.0).round() as i32
                ))
                .w_full();

            install_progress_button(button, Some(progress), cx)
        }
        UpdateStatus::Available { version } => Button::new("view-release")
            .child(Label::new("View Release"))
            .icon(Icon::new(UiIconName::ExternalLink).small())
            .on_click(move |_, _, cx| {
                cx.open_url(&UpdateState::release_url(&version));
            })
            .into_any_element(),
        UpdateStatus::Installed { .. } => Button::new("restart-now")
            .primary()
            .child(Label::new("Restart Now"))
            .on_click(|_, _, cx| updater::restart_app(cx))
            .into_any_element(),
        _ => Button::new("check-updates")
            .loading(is_checking)
            .label(if is_checking {
                "Checking..."
            } else {
                "Check for Updates"
            })
            .on_click(move |_, _, cx| {
                if !is_checking {
                    updater::check_for_updates(cx);
                }
            })
            .into_any_element(),
    };

    let preview_button = {
        let window_handle = window.window_handle().downcast::<gpui_component::Root>();
        let is_loading_changelog =
            matches!(ChangelogState::get(cx).status, ChangelogStatus::Loading);
        Button::new("preview-whats-new")
            .ghost()
            .small()
            .loading(is_loading_changelog)
            .disabled(is_loading_changelog)
            .label(if is_loading_changelog {
                "Loading changelog..."
            } else {
                "Preview \"What's New\""
            })
            .tooltip("Show the post-update changelog popup now, without waiting for an update.")
            .on_click(move |_, _, cx| {
                if let Some(window_handle) = window_handle {
                    updater::preview_whats_new(window_handle, cx);
                }
            })
    };

    v_flex()
        .gap_4()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(Label::new(format!("Oden v{APP_VERSION}")))
                        .child(
                            Label::new(format!("Update channel: {}", channel.label()))
                                .text_color(muted)
                                .text_sm(),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(preview_button)
                        .child(primary_button),
                ),
        )
        .when_some(status_row, |this, status_row| this.child(status_row))
        .into_any_element()
}

const APP_TAGLINE: &str = "Your commands, snippets, and notes live in your head. Oden puts \
them somewhere better. Store everything in one place, link related pieces together, and \
navigate your own knowledge graph, so you stop Googling the same thing twice.";

fn render_about_panel(
    _options: &RenderOptions,
    _window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;

    v_flex()
        .gap_5()
        .child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_2()
                        .items_baseline()
                        .child(Label::new("Oden").text_xl().font_weight(FontWeight::BOLD))
                        .child(Label::new(format!("v{APP_VERSION}")).text_color(muted)),
                )
                .child(Label::new(APP_TAGLINE).text_color(muted).max_w(px(520.))),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("open-repo")
                        .ghost()
                        .icon(Icon::new(UiIconName::GitHub).small())
                        .child(Label::new("Repository"))
                        .on_click(|_, _, cx| cx.open_url(REPO_URL)),
                )
                .child(
                    Button::new("report-issue")
                        .ghost()
                        .icon(Icon::new(UiIconName::TriangleAlert).small())
                        .child(Label::new("Report an Issue"))
                        .on_click(|_, _, cx| cx.open_url(&format!("{REPO_URL}/issues/new"))),
                )
                .child(
                    Button::new("view-releases")
                        .ghost()
                        .icon(Icon::new(UiIconName::ExternalLink).small())
                        .child(Label::new("Releases"))
                        .on_click(|_, _, cx| cx.open_url(&format!("{REPO_URL}/releases"))),
                ),
        )
        .child(
            div().border_t_1().border_color(border).pt_4().child(
                v_flex()
                    .gap_3()
                    .child(Label::new("Contributors").font_weight(FontWeight::BOLD))
                    .child(contributors_section(cx)),
            ),
        )
        .into_any_element()
}

fn contributors_section(cx: &App) -> gpui::AnyElement {
    let muted = cx.theme().muted_foreground;

    match &AboutState::get(cx).contributors {
        ContributorsStatus::Loading => Label::new("Loading contributors from GitHub...")
            .text_color(muted)
            .into_any_element(),
        ContributorsStatus::Error(message) => {
            Label::new(format!("Couldn't load contributors from GitHub: {message}"))
                .text_color(muted)
                .into_any_element()
        }
        ContributorsStatus::Loaded(contributors) if contributors.is_empty() => {
            Label::new("No contributors found.")
                .text_color(muted)
                .into_any_element()
        }
        ContributorsStatus::Loaded(contributors) => h_flex()
            .flex_wrap()
            .gap_2()
            .children(contributors.iter().map(|contributor| {
                let profile_url = contributor.html_url.clone();
                Button::new(SharedString::from(format!(
                    "contributor-{}",
                    contributor.login
                )))
                .ghost()
                .compact()
                .tooltip(format!(
                    "{} · {} contribution{}",
                    contributor.login,
                    contributor.contributions,
                    if contributor.contributions == 1 {
                        ""
                    } else {
                        "s"
                    }
                ))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child({
                            let avatar = Avatar::new()
                                .name(contributor.login.clone())
                                .with_size(Size::Small);
                            match contributor.avatar.clone() {
                                Some(image) => avatar.src(ImageSource::Image(image)),
                                None => avatar,
                            }
                        })
                        .child(Label::new(contributor.login.clone())),
                )
                .on_click(move |_, _, cx| cx.open_url(&profile_url))
            }))
            .into_any_element(),
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().id("settings-view").size_full().child(
            Settings::new("app-settings")
                .page(general_page())
                .page(about_page()),
        )
    }
}
