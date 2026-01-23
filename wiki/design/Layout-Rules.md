# Layout rules

## `ListItem`

`ListItem` is a row content container.

Recommended default:

- Do not stretch children horizontally.
- Only opt into flexible growth when the caller explicitly wants it.

This prevents inputs/buttons/selects from unexpectedly expanding to full width when placed in a
row container.

## Example

```rust
use gpui::div;
use yororen_ui::component::{button, list_item, text_input};

let row = list_item()
    .content(
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(text_input().placeholder("Name"))
            .child(button().child("Save")),
    );
```
