# Design logic

## Components vs Widgets

Yororen UI splits UI building blocks into two layers:

- Components: primitives (visual building blocks). They should be composable and predictable.
- Widgets: containers/behaviors (scrolling, virtualization control, higher-level layout).

This separation keeps Components small and reusable, while Widgets can focus on behavior and
performance.

## Performance philosophy

- Make the fast path the default.
- Use virtualization for long scrolling content.
- Ensure stable identity (`key(...)`) for stateful elements, especially under virtualization.

## Example: Components + Widgets

```rust
use gpui::{AnyElement, ListAlignment, div, px};
use yororen_ui::{
    component::{heading, list_item, virtual_row, HeadingLevel},
    widget::{virtual_list, virtual_list_state},
};

let state = virtual_list_state(3, ListAlignment::Top, px(128.));
let view = div().flex().flex_col().child(heading("Demo").level(HeadingLevel::H2)).child(
    virtual_list(state, move |ix, _window, _cx| -> AnyElement {
        virtual_row(("demo", ix))
            .child(list_item().content(format!("Item {ix}")))
            .into_any_element()
    }),
);
```
