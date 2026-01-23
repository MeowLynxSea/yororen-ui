# Yororen UI

Yororen UI is a reusable UI components + widgets library built on top of
[`gpui`](https://github.com/zed-industries/zed) (Zed).

It’s designed to be consumed by a `gpui` application crate, while keeping the UI layer
self-contained (theme, components, widgets, and embedded icon assets).

## Features

- Components: buttons, inputs, badges, tooltips, icons, headings, etc.
- Widgets: currently includes a `TitleBar` widget.
- Theme system: `GlobalTheme` + `ActiveTheme` helper to access colors.
- Embedded icon assets via `rust-embed` (`assets/icons/**`).

## Requirements

- Rust edition: 2024 (works with the toolchain used by your `gpui` app).
- `gpui` is a git dependency and should be pinned to a specific commit.

## Installation

### Use from GitHub (recommended)

Add this crate as a dependency via git, and pin it with a release tag (recommended):

```toml
[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.1.0" }
```

### Use from a local path (development)

```toml
[dependencies]
yororen_ui = { path = "../yororen-ui" }
```

## Pinning `gpui`

`gpui` evolves quickly and is consumed via a git dependency. If your app and `yororen_ui`
end up using *different* `gpui` revisions, you will see errors like “multiple different
versions of crate `gpui` in the dependency graph” and many trait/type mismatches.

Recommended approach:

- Pin `gpui` to the same `rev` in your application workspace.
- Pin `yororen_ui` and your application to the same `gpui` revision.

In this repository, `gpui` is pinned in `Cargo.toml`.

## Quick start

### 1) Register components

Some components need one-time registration/initialization.
Call `component::init` during app startup:

```rust
use gpui::App;
use yororen_ui::component;

fn init_ui(cx: &mut App) {
    component::init(cx);
}
```

### 2) Install the global theme

Yororen UI provides a `GlobalTheme` that selects light/dark palettes based on
`WindowAppearance`.

```rust
use gpui::App;
use yororen_ui::theme::GlobalTheme;

fn init_theme(cx: &mut App) {
    cx.set_global(GlobalTheme::new(cx.window_appearance()));
}
```

Inside render functions you can access theme colors via `ActiveTheme`:

```rust
use gpui::{Render, div};
use yororen_ui::theme::ActiveTheme;

// in render(..., cx: &mut gpui::Context<Self>)
let theme = cx.theme();
let _ = div().bg(theme.surface.base).text_color(theme.content.primary);
```

### 3) Provide assets (icons)

This crate embeds its icons under `assets/icons/**` and exposes them as a `gpui::AssetSource`
(`yororen_ui::assets::UiAsset`).

If your app only needs Yororen UI’s icons, you can install them directly:

```rust
use gpui::Application;
use yororen_ui::assets::UiAsset;

let app = Application::new().with_assets(UiAsset);
```

If your app has its own assets too, compose asset sources so both sets are available.
Yororen UI includes a small helper `CompositeAssetSource`:

```rust
use gpui::Application;
use yororen_ui::assets::{CompositeAssetSource, UiAsset};

// `MyAsset` is your own AssetSource implementation
let app = Application::new().with_assets(CompositeAssetSource::new(MyAsset, UiAsset));
```

Important: your primary `AssetSource` should return `Ok(None)` when a path doesn’t exist.
If it returns an error on missing paths, it can prevent fallback to `UiAsset`.

## What’s inside

### Modules

- `yororen_ui::theme`
  - `Theme` (palettes)
  - `GlobalTheme` (`gpui::Global`)
  - `ActiveTheme` trait (gives `theme()` on `App` and render contexts)

- `yororen_ui::assets`
  - `UiAsset` (`gpui::AssetSource` for embedded icons)
  - `CompositeAssetSource` (compose two asset sources with fallback)

- `yororen_ui::component`
  - Common building blocks used across pages:
    - `button`, `icon_button`, `text_input`, `password_input`, `tooltip`, `badge`, `divider`, etc.
  - `component::init(cx)` for any registrations.

- `yororen_ui::widget`
  - Higher-level widgets composed from components.
  - Currently: `TitleBar` and helper constructors.

### Icons

The component icon API uses strongly-typed names:

```rust
use yororen_ui::component::{icon, IconName};

let _ = icon(IconName::Minecraft);
```

Icon paths map to embedded SVG assets like `icons/minecraft.svg`.

## License

- Yororen UI is licensed under the Apache License, Version 2.0.
  See `LICENSE`.
- This project is built on top of `gpui` (Zed Industries), also Apache-2.0.

See `NOTICE` for attribution details.

## Contributing

Issues and PRs are welcome.

When changing visuals:
- Include screenshots or a short recording.
- Keep changes `rustfmt` clean.


## UI guidelines

### Keyed state (`key(...)`)

GPUI stores many pieces of UI state (cursor, selection, open menus, toggles, etc.) against an
`ElementId`. Any component that owns internal UI state should expose a `key(...)` setter as an
alias for `id(...)`.

- Use `key(...)` when you want to emphasize **state identity** (it reads better at call sites).
- Always derive keys from your data model (id/uuid/path), not from call sites.
- In virtualized lists, stable keys are mandatory; otherwise state can “bleed” between recycled
  rows.

### Virtualization (`VirtualList` / `VirtualRow`)

For long, scrollable content use Yororen UI’s virtualization primitives:

- `VirtualList` (widget): a wrapper around `gpui::list(ListState, ...)`.
- `VirtualRow` (component): a virtualization-safe row shell.

`VirtualRow` responsibilities:

1) Stable row key (required).
2) Row-local element namespace, so `Location::caller()`-based ids don’t collide across recycled rows.
3) Row spacing/dividers belong to the shell; callers should render only content.

If row height can change (expand/collapse, async-loaded content), notify the list via
`VirtualListController.reset(...)` or `VirtualListController.splice(...)`.

### `ListItem` layout

`ListItem` is a row content container. By default it **does not stretch** child components
horizontally; children keep their intrinsic widths unless you explicitly opt into flex growth.
