# Tag

A small tag/chip. Can be selectable and/or closable.

## Example

```rust
use yororen_ui::component::tag;

let view = tag("Filter")
    .selected(true)
    .closable(true)
    .on_close(|_ev, _window, _cx| {
        // ...
    });
```
