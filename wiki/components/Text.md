# Text

A simple text element that can optionally include an icon.

## Example

```rust
use gpui::px;
use yororen_ui::component::{icon, text, IconName};

let view = text("Info").with_icon(icon(IconName::Info).size(px(14.)));
```
