# ClickableSurface

A generic clickable container (hover/active styles + handlers).

## Example

```rust
use yororen_ui::component::clickable_surface;

let view = clickable_surface()
    .child("Open")
    .on_click(|_ev, _window, _cx| {
        // ...
    });
```
