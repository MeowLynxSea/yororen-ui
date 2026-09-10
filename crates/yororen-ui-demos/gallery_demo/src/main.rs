//! yororen-ui Gallery Demo — a one-stop demo that exercises every
//! component shipped by `yororen-ui` and lets the user hot-swap
//! between the default and the brutalism renderer at runtime.
//!
//! ## Boot sequence
//!
//! 1. `headless::text_input::init(cx)` — bind the 14-action
//!    keymap for the 7 input components (idempotent via `OnceLock`).
//! 2. `cx.set_global(NotificationCenter::new())` — toast /
//!    notification global state.
//! 3. `yororen_ui::locale::install_with_translations(...)` — install
//!    English as the active locale and layer the gallery's own
//!    translations on top (the toolbar can swap to zh-CN / ar).
//! 4. `theme_switcher::install_renderer` — install the default
//!    renderer + light theme (the toolbar re-installs per render).
//! 5. Open a single 1280×900 window and attach a `GalleryApp`.
//!
//! See `theme_switcher.rs` for the runtime swap mechanism and
//! `gallery_app.rs` for the per-render `install_renderer` call.

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::notification::center::NotificationCenter;

mod gallery_app;
mod i18n;
mod notifications_host;
mod sections;
mod state;
mod theme_switcher;

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1. Bind the text-input keymap once.
        yororen_ui::headless::text_input::init(cx);

        // 2. Install the notification center (toast / notification
        //    trigger from the toolbar).
        cx.set_global(NotificationCenter::new());

        // 3. Install English as the default locale. The
        //    `gallery_demo::i18n::install_for_locale` helper uses
        //    `yororen_ui::locale::install_with_translations` to layer
        //    this crate's own demo translations on top of the framework
        //    defaults from `yororen-ui-locale-en`.
        //    The toolbar locale toggle calls it again to hot-swap.
        crate::i18n::install_for_locale(cx, crate::state::LocaleChoice::En);

        // 4. Install the default renderer + light theme. The
        //    toolbar will hot-swap at runtime via
        //    `install_renderer` (called per render in
        //    `gallery_app`).
        theme_switcher::install_renderer(
            cx,
            theme_switcher::RendererKind::default(),
            theme_switcher::DarkMode::default(),
        );

        // 5. Open a single window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1280.0), px(900.0)),
                cx,
            ))),
            ..Default::default()
        };
        let app_entity = cx.new(crate::state::GalleryApp::new);
        let _ = cx.open_window(options, |_, _cx| app_entity);
    });
}
