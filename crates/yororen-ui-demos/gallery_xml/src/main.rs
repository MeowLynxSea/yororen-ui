//! Gallery Demo (XML edition) — a 1:1 XML-driven port of
//! `gallery_demo`. The layout lives in `src/ui/*.xml`; the
//! runtime renderer / locale switching and composite state
//! seeding live in Rust scaffolding.

mod controller;
mod i18n;
mod notifications_host;
mod state;
mod theme_switcher;
mod view;

use gpui::{
    App, AppContext, InteractiveElement, IntoElement, WindowBounds, WindowOptions, px,
    size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::notification::center::NotificationCenter;

use crate::controller::Controller;
use crate::state::{GalleryState, StateRef};
use crate::view::GalleryApp;

/// A custom widget registered through
/// `register_xml_component!`. Lives at the bottom of the
/// footer to show how a *non-trivial* runtime component is
/// wired in: it reads the active theme (so it visually
/// tracks the toolbar's dark/light toggle), composes two
/// `Div`s into a small "build info" card, and returns a
/// single `AnyElement` (the registry contract).
fn render_counter_widget(id: String, cx: &mut gpui::App) -> gpui::AnyElement {
    use gpui::{Hsla, ParentElement, Styled, div, hsla, px};
    use yororen_ui::theme::ActiveTheme;

    // `surface.base` exists in every bundled theme, so the
    // fallback below is only a safety net. The card border
    // uses `content.primary` for the same reason (the
    // grey-out colour matches the foreground token).
    let bg: Hsla = cx
        .theme()
        .get_color("surface.base")
        .unwrap_or_else(|| hsla(0.0, 0.0, 0.98, 1.0));
    let border: Hsla = cx
        .theme()
        .get_color("content.primary")
        .unwrap_or_else(|| hsla(0.0, 0.0, 0.5, 0.4));

    // The footer is already a column of labels; this
    // widget deliberately uses a different layout (a row
    // with a left rule) so the card stands out without
    // repeating the muted-label look.
    div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .p(px(6.))
        .rounded(px(4.))
        .bg(bg)
        .border_1()
        .border_color(border)
        .child(
            // Tiny accent dot on the left so the card
            // reads as "branded", not "another label".
            div()
                .w(px(6.))
                .h(px(6.))
                .rounded(px(3.))
                .bg(hsla(0.58, 0.7, 0.55, 1.0)),
        )
        .child(
            div()
                .text_xs()
                .text_color(hsla(0.0, 0.0, 0.35, 1.0))
                .child("v0.3.0 — yororen-ui custom widget (register_xml_component)"),
        )
        .into_any_element()
}

yororen_ui::register_xml_component!(CounterWidget => render_counter_widget);

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1. Install the default renderer + theme (light).
        //    The view will reinstall per render so toolbar
        //    toggles take effect immediately.
        theme_switcher::install_renderer(
            cx,
            theme_switcher::RendererKind::default(),
            theme_switcher::DarkMode::default(),
        );

        // 2. Bind the text-input keymap once (idempotent).
        yororen_ui::headless::text_input::init(cx);

        // 3. Install the notification center (toast /
        //    notification trigger from the toolbar).
        cx.set_global(NotificationCenter::new());

        // 4. Install English locale + the demo's own translations.
        crate::i18n::install_for_locale(cx, crate::state::LocaleChoice::En);

        // 5. Build the state + the controller that owns it.
        let state = cx.new(|cx| GalleryState::new_data(cx));
        let controller = Controller::new(state.clone(), cx);

        // 6. Make the state available to the view as a global.
        cx.set_global(StateRef { state });

        // 7. Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1280.0), px(900.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| {
            cx.new(|cx| GalleryApp::new(cx, controller))
        });
    });
}
