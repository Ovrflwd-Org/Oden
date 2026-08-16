use gpui::{App, Global, Pixels, px};

pub struct Layout {
    pub titlebar_height: Pixels,
    pub list_menu_width: Pixels,
    pub icon_rail_width: Pixels,
}

impl Layout {
    pub fn init(cx: &mut App) {
        let layout = Self {
            titlebar_height: px(32.0),
            icon_rail_width: px(56.0),
            list_menu_width: px(240.0),
        };
        cx.set_global(layout);
    }
    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

impl Global for Layout {}
