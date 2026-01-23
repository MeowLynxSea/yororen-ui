# Quick start

## 1) Register components

Some components need one-time registration.

```rust
use gpui::App;
use yororen_ui::component;

fn init_ui(cx: &mut App) {
    component::init(cx);
}
```

## 2) Install the global theme

```rust
use gpui::App;
use yororen_ui::theme::GlobalTheme;

fn init_theme(cx: &mut App) {
    cx.set_global(GlobalTheme::new(cx.window_appearance()));
}
```

Inside render functions you can access theme colors via `ActiveTheme`.

## 3) Provide assets (icons)

```rust
use gpui::Application;
use yororen_ui::assets::UiAsset;

let app = Application::new().with_assets(UiAsset);
```

If your app has its own assets, use `CompositeAssetSource`.
