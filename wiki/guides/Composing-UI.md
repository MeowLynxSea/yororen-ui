# Composing UI (how to build real screens)

This page shows how to combine Yororen UI primitives into common application layouts.

## 1) List rows: `VirtualList` + `VirtualRow` + `ListItem`

Use this pattern for settings screens, search results, file trees, etc.

```rust
use gpui::{div, px, AnyElement, ListAlignment};
use yororen_ui::{
    component::{label, list_item, virtual_row, LabelTone},
    widget::{virtual_list, virtual_list_state, VirtualListController},
};

struct MyView {
    list_state: gpui::ListState,
    list_controller: VirtualListController,
    items: Vec<String>,
}

impl MyView {
    fn new(cx: &mut gpui::App) -> Self {
        let list_state = virtual_list_state(0, ListAlignment::Top, px(256.));
        let list_controller = VirtualListController::new(list_state.clone());
        Self { list_state, list_controller, items: Vec::new() }
    }

    fn set_items(&mut self, items: Vec<String>) {
        let old_len = self.items.len();
        let new_len = items.len();
        self.items = items;
        // simplest: reset, if you don't have fine-grained splices
        self.list_controller.reset(new_len);
        // or: self.list_controller.splice(0..old_len, new_len);
    }

    fn render_list(&mut self) -> impl gpui::IntoElement {
        let items = self.items.clone();
        virtual_list(self.list_state.clone(), move |ix, _window, cx| -> AnyElement {
            let text = items.get(ix).cloned().unwrap_or_default();

            virtual_row(("row", ix))
                .gap_below(px(6.))
                .child(
                    list_item().content(label(text).tone(LabelTone::Strong))
                )
                .into_any_element()
        })
    }
}
```

Notes:

- `VirtualRow(key)` must use a stable key derived from your model.
- Keep spacing/dividers in the row shell (`gap_below` / `divider`).

## 2) “Toolbar + content” layout

```rust
use gpui::{div, px};
use yororen_ui::component::{button, heading};

let view = div()
    .flex()
    .flex_col()
    .w_full()
    .h_full()
    .child(
        // toolbar
        div()
            .h(px(44.))
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .child(heading("Settings").h2())
            .child(button().child("Save")),
    )
    .child(
        // content
        div().flex_grow().p_3().child("..."),
    );
```

