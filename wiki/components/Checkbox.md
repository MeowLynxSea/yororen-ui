# Checkbox

A boolean input.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::checkbox;

let view = checkbox()
    .key(("settings", "auto-save"))
    .checked(true)
    .on_toggle(|checked, _ev, _window, _cx| {
        // ...
    });
```
