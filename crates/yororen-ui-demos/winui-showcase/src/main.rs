//! WinUI Showcase — a small, fully WinUI-styled demo app.
//!
//! Boots a single window with a WinUI "app frame":
//!
//! - a left **navigation sidebar** (icon + label per page,
//!   selected highlight, hover, click switches pages),
//! - a content area that swaps between a Home dashboard and
//!   functional Controls / Inputs / Toggles / Lists pages,
//! - all panels drawn with the WinUI renderer's tokens
//!   (`winui.app_bg` / `winui.card_bg` / `winui.card_stroke`),
//!   corners, and Segoe UI font.
//!
//! Every control on the pages is wired to real state, so the
//! demo has actual behaviour: navigate with the sidebar, toggle
//! the switch/checkbox/radio, move the slider, type into the
//! inputs, pick items from select/combo/listbox, and click the
//! split button.

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui_winui_renderer::{install_with, winui_dark};

mod app;
mod pages;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1. Bind the text-input keymap once (text_input /
        //    password / number / search / file_path / text_area).
        yororen_ui::headless::text_input::init(cx);

        // 2. Install the WinUI renderer + dark theme. The WinUI
        //    Gallery you're matching is a dark, Mica-style app, so
        //    the showcase boots dark by default.
        install_with(cx, winui_dark());

        // 3. Open a single window with the WinUI app attached.
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1120.0), px(760.0)),
                cx,
            ))),
            ..Default::default()
        };
        let app_entity = cx.new(app::WinuiApp::new);
        let _ = cx.open_window(window_options, |_, _cx| app_entity);
    });
}
