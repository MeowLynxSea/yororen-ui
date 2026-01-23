# TextInput

Single-line text input.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::text_input;

let input = text_input()
    .key(ElementId::from((ElementId::from("settings"), "username")))
    .placeholder("Username")
    .on_change(|value, _window, _cx| {
        // value: SharedString
        // ...
    });
```

## Notes

- TextInput owns internal cursor/selection state, so stable `key(...)` is recommended in lists.
