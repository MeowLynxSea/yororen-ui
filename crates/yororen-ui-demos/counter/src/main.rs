//! yororen-ui Counter Demo
//!
//! A minimal yororen-ui application demonstrating:
//! - Global state via `gpui::Entity<T>`
//! - Event handling (button on_click)
//! - Reactive UI updates (`cx.notify()`)
//!
//! ## Run this demo
//! ```bash
//! cargo run -p counter-demo
//! ```

mod counter_app;
mod state;

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::locale_en;
use yororen_ui::renderer;

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Install the default theme + 54 default renderers in
        // one call.
        renderer::install(cx, cx.window_appearance());

        // Set up i18n with English translations.
        locale_en::install(cx);

        // Set up counter state.
        let counter_state = state::CounterState::new(cx);
        cx.set_global(counter_state);

        // Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(400.0), px(300.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| counter_app::CounterApp));
    });
}
