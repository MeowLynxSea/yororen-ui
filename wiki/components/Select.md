# Select

A non-editable dropdown.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::{select, SelectOption};

let region = select()
    .key(ElementId::from((ElementId::from("settings"), "region")))
    .placeholder("Region")
    .options([
        SelectOption::new("us", "US"),
        SelectOption::new("eu", "EU"),
    ])
    .on_change(|value, _ev, _window, _cx| {
        // ...
    });
```
