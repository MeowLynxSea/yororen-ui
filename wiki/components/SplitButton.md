# SplitButton

A primary action + dropdown options.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::{split_button, SplitButtonAction};

let view = split_button()
    .key(("actions", "export"))
    .label("Export")
    .option("png", "Export PNG")
    .option("pdf", "Export PDF")
    .on_primary(|_ev, _window, _cx| {
        // ...
    })
    .on_action(|action, _ev, _window, _cx| {
        match action {
            SplitButtonAction::Primary => {}
            SplitButtonAction::Option(id) => {
                // ...
            }
        }
    });
```
