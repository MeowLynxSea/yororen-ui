//! Toast Notification Demo
//!
//! This demo showcases the Toast notification component with various styles.
//!
//! ## Usage
//! ```bash
//! cd demo/toast_notification && cargo run
//! ```

mod toast_demo_app;

// Gpui framework imports
use gpui::{App, AppContext, Application, WindowOptions, px, size};

// yororen-ui framework imports
use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::theme::GlobalTheme;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        component::init(cx);

        cx.set_global(GlobalTheme::new(cx.window_appearance()));

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(500.0), px(400.0)),
                cx,
            ))),
            ..Default::default()
        };

        cx.open_window(options, |_, cx| {
            cx.new(|cx| toast_demo_app::ToastDemoApp::new(cx))
        }).unwrap();
    });
}
