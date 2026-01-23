# VirtualList

A virtualized list widget based on `gpui::list(ListState, ...)`.

## Example

```rust
use gpui::{AnyElement, ListAlignment, px};
use yororen_ui::{
    component::{list_item, virtual_row},
    widget::{virtual_list, virtual_list_state, VirtualListController},
};

struct MyView {
    state: gpui::ListState,
    controller: VirtualListController,
    items: Vec<String>,
}

impl MyView {
    fn new(cx: &mut gpui::App) -> Self {
        let state = virtual_list_state(0, ListAlignment::Top, px(256.));
        let controller = VirtualListController::new(state.clone());
        Self { state, controller, items: Vec::new() }
    }

    fn render(&mut self) -> impl gpui::IntoElement {
        let items = self.items.clone();
        virtual_list(self.state.clone(), move |ix, _window, _cx| -> AnyElement {
            virtual_row(("row", ix))
                .child(list_item().content(items.get(ix).cloned().unwrap_or_default()))
                .into_any_element()
        })
    }
}
```

## Notes

- Hold `ListState` at the view level.
- When item count or row height changes, notify via `VirtualListController.reset/splice`.
