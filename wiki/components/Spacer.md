# Spacer

A flexible spacer that grows to fill remaining space.

## Example

```rust
use gpui::div;
use yororen_ui::component::{button, spacer};

let view = div()
    .flex()
    .items_center()
    .child(button().child("Left"))
    .child(spacer())
    .child(button().child("Right"));
```
