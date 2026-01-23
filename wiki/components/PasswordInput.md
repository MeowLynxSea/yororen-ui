# PasswordInput

A password input field.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::password_input;

let view = password_input()
    .key(("settings", "password"))
    .placeholder("Password")
    .on_change(|value, _window, _cx| {
        // ...
    });
```
