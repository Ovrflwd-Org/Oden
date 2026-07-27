use std::sync::Arc;

use gpui::{
    AppContext, AsyncApp, BorrowAppContext, Context, CursorStyle::PointingHand, Entity,
    FocusHandle, InteractiveElement, ParentElement, Render, SharedString, Styled, Subscription,
    Window, div, px,
};
use gpui_component::{
    ActiveTheme, Icon, IndexPath, Sizable,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    label::Label,
    list::{List, ListDelegate, ListItem, ListState},
    v_flex,
};
use oden_core::repository::ItemRepositoryTrait;

use crate::{
    ItemStore,
    actions::{self, NewItem, SelectItem},
    appstatus::{AppOperation, AppStatus, Issue},
    icons::IconName,
    repository::AppRepository,
    state::SelectedIdState,
};
use crate::{models::Item, views::editor::EditorView};

pub(crate) struct ListView {
    _subscriptions: ListSubscriptions,
    pub(crate) focus_handle: FocusHandle,
    entities: ListEntities,
}

#[derive(Clone)]
struct ListEntities {
    input_state: Entity<InputState>,
    editor: Entity<EditorView>,
    list_state: Entity<ListState<ItemListDelegate>>,
    selected_id_state: Entity<SelectedIdState>,
    app_status_entity: Entity<AppStatus>,
}

impl ListView {
    pub(crate) fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        focus_handle: FocusHandle,
        selected_id_state: Entity<SelectedIdState>,
        app_status_entity: Entity<AppStatus>,
    ) -> Self {
        let entities = Self::build_entities(
            window,
            cx,
            focus_handle.clone(),
            selected_id_state,
            app_status_entity,
        );
        let subscriptions: ListSubscriptions =
            Self::wire_subscriptions(window, cx, entities.clone());
        Self {
            focus_handle,
            _subscriptions: subscriptions,
            entities,
        }
    }

    #[cfg(test)]
    pub(crate) fn editor(&self) -> Entity<EditorView> {
        self.entities.editor.clone()
    }

    fn build_entities(
        window: &mut Window,
        cx: &mut Context<Self>,
        focus_handle: FocusHandle,
        selected_id_state: Entity<SelectedIdState>,
        app_status_entity: Entity<AppStatus>,
    ) -> ListEntities {
        let input_state = Self::build_input_state(window, cx);
        let list_state = Self::build_list_state(window, cx, focus_handle.clone());
        let editor = Self::build_editor_view(window, cx, selected_id_state.clone());
        ListEntities {
            input_state,
            editor,
            list_state,
            selected_id_state,
            app_status_entity,
        }
    }

    async fn add_empty_item(
        cx: &mut AsyncApp,
        repository: Arc<dyn ItemRepositoryTrait + Send + Sync>,
        selected_id_state: Entity<SelectedIdState>,
    ) -> anyhow::Result<()> {
        let model = repository.as_ref().create_item().await?;
        let item = Item::from(model);
        let item_id = item.id;
        cx.update(|cx| {
            cx.update_global::<ItemStore, _>(|store, _app| {
                store.items.insert(item_id, item);
            });
            cx.update_entity(&selected_id_state, |state, cx| {
                state.selected_id = Some(item_id);
                cx.notify();
            });
        })?;
        Ok(())
    }

    fn build_input_state(window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx).placeholder("Search for anything..."))
    }

    fn build_editor_view(
        window: &mut Window,
        cx: &mut Context<Self>,
        selected_id_state: Entity<SelectedIdState>,
    ) -> Entity<EditorView> {
        cx.new(|cx| EditorView::new(cx, window, selected_id_state))
    }

    fn build_list_state(
        window: &mut Window,
        cx: &mut Context<Self>,
        focus_handle: FocusHandle,
    ) -> Entity<ListState<ItemListDelegate>> {
        let items: Vec<Item> = ItemStore::get(cx).items().values().cloned().collect();
        let mut delegate: ItemListDelegate = ItemListDelegate {
            items: Vec::new(),
            all_items: Vec::new(),
            selected_index: None,
            focus: focus_handle.clone(),
            query: String::new(),
        };
        delegate.set_all_items(items);
        cx.new(|cx| ListState::new(delegate, window, cx))
    }

    fn wire_subscriptions(
        window: &mut Window,
        cx: &mut Context<Self>,
        entities: ListEntities,
    ) -> ListSubscriptions {
        let list_state_clone = entities.list_state.clone();
        let input_sub = cx.subscribe_in(
            &entities.input_state,
            window,
            move |_view, state, event, _window, cx| {
                if let InputEvent::Change = event {
                    let query = state.read(cx).value().clone();
                    cx.update_entity(&list_state_clone, |state, _cx| {
                        state.delegate_mut().set_query(query);
                    });
                    cx.notify();
                }
            },
        );
        let selected_id_state_clone = entities.selected_id_state.clone();
        let list_state_clone = entities.list_state.clone();
        let store_sub = cx.observe_global::<ItemStore>(move |_store, cx| {
            let all_items: Vec<Item> = ItemStore::get(cx).items().values().cloned().collect();
            let selected_id = selected_id_state_clone.read(cx).selected_id;
            cx.update_entity(&list_state_clone, |state, _cx| {
                let delegate = state.delegate_mut();
                delegate.set_all_items(all_items);
                delegate.selected_index = selected_id
                    .and_then(|id| delegate.items.iter().position(|item| item.id == id))
                    .map(IndexPath::new);
            });
            cx.notify();
        });

        let selected_id_sub = cx.observe_in(
            &entities.selected_id_state,
            window,
            move |_this, selected_id_state, window, cx| {
                let selected_id = selected_id_state.read(cx).selected_id;
                let selected_index = {
                    let state = entities.list_state.read(cx);
                    selected_id
                        .and_then(|id| state.delegate().items.iter().position(|item| item.id == id))
                        .map(IndexPath::new)
                };
                entities.list_state.update(cx, |state, cx| {
                    state.set_selected_index(selected_index, window, cx);
                });
            },
        );

        ListSubscriptions {
            _input_sub: input_sub,
            _store_sub: store_sub,
            _selected_id_sub: selected_id_sub,
        }
    }
}

struct ListSubscriptions {
    _input_sub: Subscription,
    _store_sub: Subscription,
    _selected_id_sub: Subscription,
}

struct ItemListDelegate {
    items: Vec<Item>,
    all_items: Vec<Item>,
    selected_index: Option<IndexPath>,
    focus: FocusHandle,
    query: String,
}

fn preview_content(s: SharedString) -> SharedString {
    let truncated = &s[..s.floor_char_boundary(50.min(s.len()))];
    let truncated: Vec<_> = truncated.lines().filter(|line| !line.is_empty()).collect();
    truncated.join(" ").into()
}

impl ItemListDelegate {
    fn set_all_items(&mut self, items: Vec<Item>) {
        self.all_items = items;
        self.recompute_items();
    }

    fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into().to_lowercase();
        self.recompute_items();
    }

    fn recompute_items(&mut self) {
        let query = self.query.as_str();
        let mut visible: Vec<Item> = self
            .all_items
            .iter()
            .filter(|item| item.name.to_string().to_lowercase().contains(query))
            .cloned()
            .collect();
        visible.sort_by_key(|i| i.name.to_lowercase());
        self.items = visible;
    }
}

impl ListDelegate for ItemListDelegate {
    type Item = ListItem;
    fn items_count(&self, _section: usize, _cx: &gpui::App) -> usize {
        self.items.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<gpui_component::list::ListState<Self>>,
    ) -> Option<Self::Item> {
        let theme = cx.theme();
        let muted_color = theme.muted_foreground;
        let date_color = theme.blue;
        let border_color = theme.border;
        let size = cx.theme().mono_font_size;
        let focus = self.focus.clone();
        self.items.get(ix.row).map(|item| {
            let selected_id = item.id;
            ListItem::new(ix)
                .h_32()
                .overflow_hidden()
                .flex()
                .flex_col()
                .justify_between()
                .p_2()
                .border_b(px(1.0))
                .border_color(border_color)
                .child(Label::new(item.name.clone()))
                .child(v_flex().children(vec![
                    Label::new(item.created_at.date_naive().to_string())
                        .text_color(date_color)
                        .text_size(size),
                    Label::new(preview_content(item.content.clone()))
                        .text_color(muted_color)]))
                .on_click(move |_event, window, cx| {
                    focus.focus(window);
                    let select_item_action = SelectItem { selected_id };
                    window.dispatch_action(Box::new(select_item_action), cx);
                })
                .selected(Some(ix) == self.selected_index)
        })
    }
    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<gpui_component::list::ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }
}

impl Render for ListView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let border_color = theme.border;
        let muted_color = theme.muted_foreground;
        let focus = self.focus_handle.clone();
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(move |this, _action: &NewItem, _window, cx| {
                let selected_id_state = this.entities.selected_id_state.clone();
                let app_status_entity = this.entities.app_status_entity.clone();
                let repository = cx.global::<AppRepository>().0.clone();
                cx.spawn(async move |_this, cx| {
                    if let Err(err) = Self::add_empty_item(cx, repository, selected_id_state).await
                    {
                        let _ = app_status_entity.update(cx, |state, cx| {
                            let new_issue =
                                Issue::new(AppOperation::CreateNewItem, err.to_string());
                            state.issues.push(new_issue);
                            cx.notify();
                        });
                        eprintln!("failed to add item: {err}");
                    }
                })
                .detach();
            }))
            .w_full()
            .h_full()
            .flex_row()
            .flex()
            .child(
                div()
                    .w_1_4()
                    .border_r(px(1.0))
                    .border_color(border_color)
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .justify_center()
                            .items_center()
                            .border_b(px(1.0))
                            .border_color(border_color)
                            .p_2()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap(px(5.0))
                                    .w_full()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .gap_1()
                                            .w_full()
                                            .child(div())
                                            .child(Label::new(SharedString::from("All Items")))
                                            .child(
                                                Button::new("new-item")
                                                    .icon(
                                                        Icon::new(IconName::Pencil)
                                                            .small()
                                                            .text_color(muted_color),
                                                    )
                                                    .cursor(PointingHand)
                                                    .tooltip("New Item")
                                                    .on_click(move |_event, window, cx| {
                                                        focus.focus(window);
                                                        window.dispatch_action(
                                                            Box::new(actions::NewItem),
                                                            cx,
                                                        );
                                                    })
                                                    .ghost(),
                                            ),
                                    )
                                    .child(Input::new(&self.entities.input_state).prefix(
                                        Icon::new(IconName::Search).small().text_color(muted_color),
                                    )),
                            ),
                    )
                    .child(
                        List::new(&self.entities.list_state)
                            .flex()
                            .flex_col()
                            .gap_5(),
                    ),
            )
            .child(self.entities.editor.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::actions::{NewItem, SelectItem};
    use crate::repository::AppRepository;
    use crate::store::ItemStore;
    use crate::testutils::setup;
    use async_trait::async_trait;
    use chrono::Utc;
    use gpui::TestAppContext;
    use oden_core::entities::item;
    use oden_core::repository::ItemRepositoryTrait;
    use sea_orm::DbErr;
    use serde_json::json;
    use uuid::Uuid;

    struct MockItemRepository;

    #[async_trait]
    impl ItemRepositoryTrait for MockItemRepository {
        async fn find_all(&self) -> Result<Vec<item::Model>, DbErr> {
            Ok(vec![])
        }

        async fn create_item(&self) -> Result<item::Model, DbErr> {
            let now = Utc::now();
            Ok(item::Model {
                id: Uuid::from_u128(1),
                name: "Untitled".to_string(),
                content: "# Untitled".to_string(),
                kind: item::ItemKind::Note,
                tags: json!([]),
                language: None,
                created_at: now,
                modified_at: now,
            })
        }
    }

    #[gpui::test]
    fn test_list_items_navigation(cx: &mut TestAppContext) {
        let (window, _app_mode_state, selected_id_state) = setup(cx);
        let uuid = Uuid::new_v4();
        window
            .update(cx, |root, window, cx| {
                root.focus.focus(window);
                window.dispatch_action(Box::new(SelectItem { selected_id: uuid }), cx);
            })
            .unwrap();
        cx.update(|cx| {
            assert_eq!(
                selected_id_state.read(cx).selected_id.unwrap(),
                uuid,
                "id selection failed"
            )
        })
    }

    #[gpui::test]
    fn test_new_item_creation(cx: &mut TestAppContext) {
        let (window, _app_mode_state, selected_id_state) = setup(cx);
        cx.update(|cx| {
            let repository: Arc<dyn ItemRepositoryTrait + Send + Sync> =
                Arc::new(MockItemRepository);
            cx.set_global(AppRepository(repository));
        });
        window
            .update(cx, |root, window, cx| {
                root.list_view.read(cx).focus_handle.focus(window);
                window.dispatch_action(Box::new(NewItem), cx);
            })
            .unwrap();

        // wait for async tasks to finish.
        cx.run_until_parked();

        cx.update(|cx| {
            let id = selected_id_state
                .read(cx)
                .selected_id
                .expect("expected selected_id to be set after NewItem");
            assert_eq!(id, Uuid::from_u128(1));
        })
    }

    #[gpui::test]
    fn test_selected_id_subscription(cx: &mut TestAppContext) {
        let (window, _app_mode_state, selected_id_state) = setup(cx);
        let target_id = cx.update(|cx| {
            ItemStore::get(cx)
                .items()
                .keys()
                .next()
                .copied()
                .expect("item store should contain at least one item")
        });
        window
            .update(cx, |_root, _window, cx| {
                selected_id_state.update(cx, |state, cx| {
                    state.selected_id = Some(target_id);
                    cx.notify();
                })
            })
            .unwrap();
        cx.run_until_parked();
        window
            .update(cx, |root, _window, cx| {
                let list_state = root.list_view.read(cx).entities.list_state.read(cx);
                let index_path = list_state
                    .selected_index()
                    .expect("index path should be present");
                let selected_id = list_state.delegate().items[index_path.row].id;
                // assert that the selected_id in the list (that actually shows up as selected in
                // the UI) is the id that we updated the selected_id_state with.
                assert_eq!(selected_id, target_id);
            })
            .unwrap();
    }
}
