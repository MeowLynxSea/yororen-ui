# TextEditState

A helper type for text editing (selection, cursor, marked text) used by text input components.

This is not typically used directly in app UI layouts, but it can be useful when you need to
inspect or manipulate selections/cursor behavior.

## Example

```rust
use yororen_ui::component::TextEditState;

let mut state = TextEditState::new();
state.set_content("Hello");
state.move_to(5);
```
