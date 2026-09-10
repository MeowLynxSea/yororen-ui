//! yororen-ui Variant Showcase Demo
//!
//! Three side-by-side buttons, each demonstrating how the
//! same `headless::button` + `default_render` can pull a
//! different `action.<variant>.*` slot from the theme JSON.
//!
//! The headless API exposes the variant through the renderer's
//! `ButtonRenderState` (built per-render inside `default_render`).
//! To exercise a non-default variant the demo calls the
//! renderer directly via the core registry:
//!
//! 1. `cx.renderer_arc::<markers::Button, dyn ButtonRenderer>()`
//! 2. Build a `ButtonRenderState` with the desired variant
//! 3. Call the renderer's `bg` / `fg` / `padding` / etc.

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::renderer;

mod variant_app;

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        renderer::install(cx, cx.window_appearance());

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(700.0), px(420.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| {
            cx.new(|_cx| variant_app::VariantApp::new())
        });
    });
}
