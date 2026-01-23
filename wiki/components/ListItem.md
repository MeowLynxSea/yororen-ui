# ListItem

A row content container for list-style UIs.

## Example

```rust
use gpui::{div, px};
use yororen_ui::component::{icon, label, list_item, IconName, LabelTone};

let row = list_item()
    .leading(icon(IconName::Search).size(px(16.)))
    .content(label("Search").tone(LabelTone::Strong))
    .secondary(label("Find in files").tone(LabelTone::Muted))
    .trailing(div().child("⌘F"));
```

## Notes

- `ListItem` is not the virtualization boundary. Use `VirtualRow` for virtualization.
