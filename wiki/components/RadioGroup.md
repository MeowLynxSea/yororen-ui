# RadioGroup

A group of radio options.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::{radio_group, RadioOption};

let view = radio_group()
    .key(("settings", "theme"))
    .options([
        RadioOption::new("light", "Light"),
        RadioOption::new("dark", "Dark"),
    ])
    .on_change(|value, _ev, _window, _cx| {
        // ...
    });
```
