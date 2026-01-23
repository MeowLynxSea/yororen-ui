# Switch

A toggle switch.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::switch;

let view = switch()
    .key(("settings", "notifications"))
    .checked(true)
    .on_toggle(|checked, _ev, _window, _cx| {
        // ...
    });
```
