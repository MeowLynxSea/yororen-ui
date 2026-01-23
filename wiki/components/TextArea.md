# TextArea

A multi-line text input.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::{text_area, EnterBehavior, WrapMode};

let view = text_area()
    .key(("settings", "notes"))
    .placeholder("Notes")
    .wrap(WrapMode::Soft)
    .enter_behavior(EnterBehavior::Newline)
    .on_change(|value, _window, _cx| {
        // ...
    });
```
