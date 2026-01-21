# Yororen UI

Reusable UI components and widgets built on top of `gpui`.

## Usage

Add as a dependency (git):

```toml
yororen_ui = { git = "https://github.com/<you>/yororen-ui", rev = "<rev>" }
```

In your app:

```rust
use yororen_ui::{component, theme::GlobalTheme};

// Initialize component registries.
component::init(cx);

// Install global theme.
cx.set_global(GlobalTheme::new(cx.window_appearance()));
```
