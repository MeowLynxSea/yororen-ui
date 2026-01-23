# Tooltip

A tooltip view builder.

## Example

```rust
use yororen_ui::component::{clickable_surface, tooltip};

let view = clickable_surface()
    .child("Hover me")
    .tooltip(tooltip("Hello").build());
```
