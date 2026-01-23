# Recommended practices

## 1) Prefer primitives + composition

- Use Components for small reusable pieces (inputs, buttons, list rows).
- Use Widgets for container behavior (virtualized scrolling, title bars, overlays).

## 2) Keyed state: always provide stable identity when needed

If a component owns internal UI state, use a stable `key(...)` (alias of `id(...)`) when:

- Rendering a list of many stateful components.
- Using virtualization.
- The same UI can be reordered or inserted/removed.

Derive keys from your data model (id/uuid/path), not from call sites.

## 3) Virtualize large scrollable content

For long scrollable content, use `VirtualList` + `VirtualRow`.

Avoid rendering a single “giant item” that contains a huge subtree if you can split it into
rows — virtualization clips what’s offscreen, but it can’t make a single visible item cheap.

## Example

```rust
use gpui::{AnyElement, ListAlignment, px};
use yororen_ui::{
    component::{select, virtual_row, SelectOption},
    widget::{virtual_list, virtual_list_state},
};

let state = virtual_list_state(2, ListAlignment::Top, px(128.));
let list = virtual_list(state, move |ix, _window, _cx| -> AnyElement {
    virtual_row(("row", ix))
        .child(
            select()
                .key(("row", ix))
                .options([
                    SelectOption::new("a", "Option A"),
                    SelectOption::new("b", "Option B"),
                ]),
        )
        .into_any_element()
});
```
