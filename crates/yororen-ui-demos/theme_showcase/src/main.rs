//! yororen-ui Theme Showcase Demo
//!
//! A single window that demonstrates live theme switching: a
//! `headless::button` is rendered by the same `TokenButtonRenderer`
//! but the JSON the renderer reads is swapped on every "Next"
//! click. The four bundled themes are:
//!
//! - `themes/system-light.json` (default light — neutral palette)
//! - `themes/system-dark.json`  (default dark — neutral palette)
//! - `CATPPUCCIN` (inline — user-defined catppuccin mocha)
//! - `MATERIAL`   (inline — user-defined material rose)
//!
//! The point of the demo: themes are **just JSON**. The
//! renderer doesn't care which JSON you feed it; the same
//! `headless::button` + the same `TokenButtonRenderer` is
//! reused for all four.

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui_default_renderer as default_renderer;

mod theme_app;

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Register the 54 default `TokenXxxRenderer` impls. We
        // do *not* install a theme here — the demo installs a
        // theme on every `Render::render` from `theme_app`, so
        // that clicking "Next theme" can swap the active JSON
        // and the whole window re-themes instantly.
        default_renderer::install(cx, cx.window_appearance());

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(700.0), px(500.0)),
                cx,
            ))),
            ..Default::default()
        };
        let app_entity = cx.new(|_cx| theme_app::ThemeApp::new());
        let _ = cx.open_window(options, |_, _cx| app_entity);
    });
}
