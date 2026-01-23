# Virtualization (VirtualList / VirtualRow)

## Why

Virtualization keeps scrolling smooth by only building and laying out what is visible.

## Building blocks

- `VirtualList` (widget): a wrapper around `gpui::list(ListState, ...)`.
- `VirtualRow` (component): a virtualization-safe row shell.

## VirtualRow responsibilities

1) Stable per-row key (required).
2) Row-local element namespace, so `Location::caller()`-based ids don’t collide across recycled rows.
3) Row spacing/dividers belong to the shell; callers render only content.

## Dynamic height

If a row’s height changes (expand/collapse, async-loaded content), call controller methods such as:

- `VirtualListController.reset(...)`
- `VirtualListController.splice(...)`

## Example

```rust
use gpui::{AnyElement, ListAlignment, px};
use yororen_ui::{
    component::{list_item, virtual_row},
    widget::{virtual_list, virtual_list_state},
};

let state = virtual_list_state(1000, ListAlignment::Top, px(256.));
let list = virtual_list(state, move |ix, _window, _cx| -> AnyElement {
    virtual_row(("row", ix))
        .gap_below(px(6.))
        .child(list_item().content(format!("Row {ix}")))
        .into_any_element()
});
```
