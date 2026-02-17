//! yororen-ui Counter Demo
//!
//! A minimal yororen-ui application demonstrating:
//! - Simple global state management with Arc<Mutex<T>>
//! - Event handling (button on_click)
//! - UI updates via cx.notify()
//!
//! ## Run this demo
//! ```bash
//! cd demo/counter && cargo run
//! ```

mod counter_app;
mod state;

// Gpui framework imports
use gpui::{App, AppContext, Application, WindowOptions, px, size};

// yororen-ui framework imports
use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::theme::GlobalTheme;

/// Application entry point
fn main() {
    // Create application instance with UI assets
    let app = Application::new().with_assets(UiAsset);

    // Initialize application
    app.run(|cx: &mut App| {
        // Initialize yororen-ui component library
        component::init(cx);

        // Set up theming (light/dark mode based on system)
        cx.set_global(GlobalTheme::new(cx.window_appearance()));

        // Set up counter state (simple i32 wrapped in Arc<Mutex<T>>)
        cx.set_global(state::CounterState::default());

        // Create main window
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(400.0), px(300.0)),
                cx,
            ))),
            ..Default::default()
        };

        // Open window with counter component
        cx.open_window(options, |_, cx| {
            cx.new(|cx| counter_app::CounterApp::new(cx))
        }).unwrap();
    });
}
