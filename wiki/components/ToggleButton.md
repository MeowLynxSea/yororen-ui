# ToggleButton

A selectable button (useful for segmented controls or filters).

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::toggle_button;

let view = toggle_button("Bold")
    .key(("toolbar", "bold"))
    .default_selected(false)
    .on_toggle(|selected, _ev, _window, _cx| {
        // ...
    });
```

## Notes

- Use `.group("...")` to create a mutually exclusive group.
