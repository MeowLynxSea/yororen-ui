# SearchInput

A standardized search input (magnifier + clear button + submit).

## Example

```rust
use gpui::ElementId;
use yororen_ui::component::search_input;

let view = search_input()
    .key(("search", "query"))
    .placeholder("Search…")
    .on_change(|value, _window, _cx| {
        // ...
    })
    .on_submit(|value, _window, _cx| {
        // ...
    });
```
