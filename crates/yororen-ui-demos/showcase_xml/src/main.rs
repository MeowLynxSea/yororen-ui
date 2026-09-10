//! yororen-ui Showcase (XML edition).
//!
//! Demonstrates the full Phase 1 + Phase 2 + Phase 3
//! feature surface of the `xml!` / `xml_file!` macro:
//! containers, leaves, event handlers, control flow
//! (`If` / `ElseIf` / `Else` / `For` / `Match` /
//! `Case`), inline expressions, two-way binding
//! (`@bind`), local state (`<State>`), event modifiers
//! (`.stop` / `.prevent` / keyboard filters), string
//! interpolation, and runtime-registered custom
//! components.
//!
//! ## Architecture
//!
//! ```
//! state.rs        — pure data: entities, enums, lists
//! controller.rs   — business logic; one method per
//!                   on_click / on_change handler
//! view.rs         — Render impl; reads state, hands
//!                   the controller to XML
//! ui/showcase.xml — purely declarative; references
//!                   controller methods by name
//! ```
//!
//! The XML file is the **single source of truth for
//! the interface**. Every business decision (counter
//! increments, status transitions, name updates) is
//! expressed in Rust — the XML never contains
//! `move |_, _, cx| { ... }` closures or `update(cx,
//! |s, _| ...)` boilerplate.
//!
//! ## Run
//!
//! ```bash
//! cargo run -p showcase-xml-demo
//! ```

mod controller;
mod state;
mod view;

use gpui::{
    App, AppContext, InteractiveElement, IntoElement, WindowBounds, WindowOptions, px,
    size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::locale_en;
use yororen_ui::renderer;

use crate::controller::Controller;
use crate::state::{ShowcaseState, StateRef};
use crate::view::ShowcaseApp;

/// A custom widget registered through
/// `register_xml_component!` — the user-facing extension
/// point for adding new XML tags at runtime.
///
/// This widget reads the **currently installed theme** and
/// paints a colour swatch + HSL label, so the rendered
/// surface visibly tracks whichever palette the toolbar
/// last picked. It is intentionally richer than
/// `div().id(id)` so the demo shows the real shape of a
/// custom component:
///
/// 1. read live state from `cx` (here, `cx.theme()`)
/// 2. compose several `Div`s (swatch + label row)
/// 3. honour the `id` attribute (so XML-driven state can
///    target it for hit-testing / focus)
/// 4. return a single `AnyElement` (the registry contract)
fn render_custom_widget(id: String, cx: &mut gpui::App) -> gpui::AnyElement {
    use gpui::{Hsla, ParentElement, Styled, div, hsla, px};
    use yororen_ui::theme::ActiveTheme;

    // The default `Hsla::default()` is fully transparent,
    // so a missing key would render the swatch invisible.
    // We fall back to a saturated magenta so the demo
    // visibly fails (rather than silently disappearing)
    // if a user wires up a theme JSON without
    // `surface.base`.
    let swatch: Hsla = cx
        .theme()
        .get_color("surface.base")
        .unwrap_or_else(|| hsla(0.83, 0.7, 0.55, 1.0));

    // `Hsla` is `(h, s, l, a)`; format with 2 decimals so
    // the label reads as a real value, not a 16-digit
    // float. We rely on the trait `Display` from gpui.
    let label = format!(
        "theme.surface.base = h {:.2}°  s {:.2}  l {:.2}  a {:.2}",
        swatch.color.hue.into_degrees(), swatch.saturation, swatch.lightness, swatch.alpha
    );

    // 2-row composition: a 48×48 swatch on top, an
    // HSL-readout underneath. Padding, radius and
    // border are theme-agnostic (no `unwrap_or_default`
    // needed for the `bg` / `border_color` here — they
    // come from the host's own styled base).
    div()
        .id(id)
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .w(px(48.))
                .h(px(48.))
                .rounded(px(6.))
                .border_1()
                .border_color(hsla(0.0, 0.0, 0.0, 0.2))
                .bg(swatch),
        )
        .child(
            div()
                .text_xs()
                .text_color(hsla(0.0, 0.0, 0.25, 1.0))
                .child(label),
        )
        .into_any_element()
}

yororen_ui::register_xml_component!(CustomWidget => render_custom_widget);

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Install the default theme + renderers.
        renderer::install(cx, cx.window_appearance());

        // Bind the global keymap for text-input
        // actions (Backspace, Delete, Left/Right,
        // SelectAll, Paste, etc.). Without this the
        // TextInput can't accept backspace or other
        // editing keys - they're bound to the
        // "UITextInput" key context that this
        // registers (idempotent — safe to call once).
        yororen_ui::headless::text_input::init(cx);

        // English translations.
        locale_en::install(cx);

        // Build the state and the controller that owns it.
        // The controller is `Clone` so each event handler
        // closure in the XML gets its own handle.
        let state = cx.new(|cx| ShowcaseState::new_data(cx));
        let controller = Controller::new(state.clone());

        // Make the state available to the view as a global
        // (the view reads it via `cx.global::<StateRef>()`).
        cx.set_global(StateRef { state });

        // Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(720.0), px(620.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| {
            cx.new(|cx| ShowcaseApp::new(cx, controller))
        });
    });
}
