use comrak_gpui::render_document;
use gpui::{
    AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window, div, px,
};
use gpui_component::ActiveTheme;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use uuid::Uuid;

use crate::models::Item;
use crate::state::SelectedIdState;
use crate::store::ItemStore;

pub struct EditorView {
    input_state: Entity<InputState>,
    _selected_id_state_sub: Subscription,
}

impl EditorView {
    pub fn new(
        cx: &mut Context<Self>,
        window: &mut Window,
        selected_id_state: Entity<SelectedIdState>,
    ) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .code_editor("markdown")
                .searchable(true)
                // this is essential so the layout does not break
                // if the text is too long.
                .soft_wrap(true)
                .line_number(true)
        });
        let _selected_id_state_sub = cx.observe_in(
            &selected_id_state,
            window,
            move |this, selected_id_state, window, cx| {
                let selected_id_maybe = selected_id_state.read(cx).selected_id;
                let content = selected_id_maybe
                    .and_then(|selected_id| Self::get_item_for_selected_id(cx, selected_id))
                    .map(|item| item.content)
                    .unwrap_or_else(|| "".into());
                this.input_state.update(cx, |input_state, cx| {
                    input_state.set_value(content, window, cx);
                });
            },
        );
        EditorView {
            input_state,
            _selected_id_state_sub,
        }
    }

    fn get_item_for_selected_id(cx: &mut Context<Self>, selected_id: Uuid) -> Option<Item> {
        ItemStore::get(cx).items().get(&selected_id).cloned()
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let content = self.input_state.read(cx).value();
        let theme = cx.theme();
        div()
            .flex()
            .flex_row()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    // 50% split when we set flex_1 with two children.
                    .flex_1()
                    // min_w_0 so width does not interfere with the split.
                    .w_0()
                    .min_w_0()
                    .overflow_hidden()
                    .border_r(px(1.))
                    .border_color(theme.border)
                    .child(
                        Input::new(&self.input_state)
                            .h_full()
                            .border(px(0.))
                            .font_family("JetBrainsMono Nerd Font"),
                    ),
            )
            .child(
                div().flex_1().w_0().min_w_0().overflow_hidden().child(
                    div()
                        .size_full()
                        .overflow_y_scrollbar()
                        .child(render_document(content.as_ref(), cx)),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::SelectItem;
    use crate::store::ItemStore;
    use crate::testutils::setup;

    #[gpui::test]
    fn test_editor_updates_on_select(cx: &mut gpui::TestAppContext) {
        let (window, _app_mode_state, _selected_id_state) = setup(cx);
        let selected_id = cx.update(|cx| {
            ItemStore::get(cx)
                .items()
                .keys()
                .next()
                .copied()
                .expect("store should have at least one item in this test")
        });
        window
            .update(cx, |root, window, cx| {
                root.focus.focus(window);
                window.dispatch_action(Box::new(SelectItem { selected_id }), cx);
            })
            .unwrap();
        let editor_text = window
            .update(cx, |root, _window, cx| {
                root.list_view
                    .read(cx)
                    .editor()
                    .read(cx)
                    .input_state
                    .read(cx)
                    .value()
            })
            .unwrap();
        let expected_content = cx.update(|cx| {
            ItemStore::get(cx)
                .items()
                .get(&selected_id)
                .unwrap()
                .content
                .clone()
        });
        assert!(editor_text == expected_content)
    }
}
