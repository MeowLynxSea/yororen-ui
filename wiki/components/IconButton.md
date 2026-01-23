# IconButton

A button that displays only an icon.

## Example

```rust
use gpui::px;
use yororen_ui::component::{icon, icon_button, IconName};

let close = icon_button(icon(IconName::Close).size(px(14.)))
    .on_click(|_ev, _window, _cx| {
        // ...
    });
```
