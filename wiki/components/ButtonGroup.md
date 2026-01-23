# ButtonGroup

A layout helper for grouping buttons.

## Example

```rust
use gpui::px;
use yororen_ui::component::{button, button_group};

let view = button_group()
    .gap(px(8.).into())
    .child(button().child("Cancel"))
    .child(button().child("Save"));
```

## Notes

- Use `.connected(true)` when you want a single connected shape.
