# VirtualRow

A virtualization-safe row shell.

## Example

```rust
use gpui::px;
use yororen_ui::component::{list_item, virtual_row};

let row = virtual_row(("item", 42))
    .gap_below(px(6.))
    .child(list_item().content("Row content"));
```

## Contract

- Stable key is required.
- Spacing/dividers should be attached to the row shell.
- If row height changes after render, notify the list via `VirtualListController`.
