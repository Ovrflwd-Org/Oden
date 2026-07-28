use comrak_gpui::render_document;
use gpui::{AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window, px};
use gpui_component::input::{Input, InputState};
use gpui_component::resizable::{h_resizable, resizable_panel};
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
        let content = &self.input_state.read(cx).text().as_str().unwrap_or("");
        h_resizable("split pane editor")
            .child(
                resizable_panel().child(
                    Input::new(&self.input_state)
                        .border(px(0.))
                        .font_family("JetBrainsMono Nerd Font"),
                ),
            )
            .child(resizable_panel().child(render_document(content)))
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
