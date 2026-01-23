# DragHandle

A drag handle primitive (mouse down/move hooks + cursor).

## Example

```rust
use yororen_ui::component::drag_handle;

let view = drag_handle()
    .on_drag_start(|_ev, _window, _cx| {
        // ...
    })
    .on_drag_move(|_ev, _window, _cx| {
        // ...
    });
```
