# Keyed state (`key(...)` / `id(...)`)

`gpui` stores many pieces of UI state (cursor, selection, open menus, toggles, etc.) against an
`ElementId`.

Yororen UI uses the convention:

- Any Component that owns internal state should expose `key(...)`.
- `key(...)` is an alias of `id(...)`.

When to use:

- Any repeated UI (lists/grids).
- Virtualized rows.
- Anything reorderable.

Guideline:

- Keys should come from your data model (id/uuid/path).
- Do not use call-site location as identity when virtualization can recycle rows.

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::{text_input, virtual_row};

let row = virtual_row(("settings", "username-row"))
    .child(
        text_input()
            .key(ElementId::from((ElementId::from("settings"), "username")))
            .placeholder("Username"),
    );
```
