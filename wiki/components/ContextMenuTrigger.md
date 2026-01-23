# ContextMenuTrigger

A right-click trigger primitive.

## Example

```rust
use yororen_ui::component::context_menu_trigger;

let view = context_menu_trigger().on_open(|_ev, _window, _cx| {
    // show a context menu / popover in your app
});
```
