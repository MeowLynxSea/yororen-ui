//! yororen-ui File Browser Demo
//!
//! Demonstrates rendering and interacting with a complex hierarchical data structure:
//! - Directory tree (Tree + TreeItem)
//! - Icons
//! - Empty state
//! - Right-click popover menu (copy/paste)
//!
//! ## Run
//! ```bash
//! cd demo/file_browser && cargo run
//! ```

mod file_browser_app;
mod actions;
mod clipboard;
mod components;
mod format;
mod fs_ops;
mod scan;
mod state;

use gpui::{App, AppContext, Application, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::theme::GlobalTheme;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        component::init(cx);
        cx.set_global(GlobalTheme::new(cx.window_appearance()));
        cx.set_global(state::FileBrowserState::default());

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(980.0), px(680.0)),
                cx,
            ))),
            ..Default::default()
        };

        cx.open_window(options, |_, cx| {
            cx.new(|cx| file_browser_app::FileBrowserApp::new(cx))
        })
        .unwrap();
    });
}
