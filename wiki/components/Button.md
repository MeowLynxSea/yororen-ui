# Button

A clickable action.

## Example

```rust
use yororen_ui::component::button;

let save = button()
    .child("Save")
    .on_click(|_ev, _window, _cx| {
        // ...
    });
```

## Notes

- Prefer `key(...)` only when you render many stateful buttons in repeated/virtualized UIs.
