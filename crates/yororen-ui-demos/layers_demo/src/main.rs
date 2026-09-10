//! yororen-ui Layers Demo
//!
//! Four panels demonstrating the v0.3 three-layer
//! architecture: **headless** (a11y + state), **default
//! renderer** (theme-driven visuals), and **caller custom**
//! (caller-supplied visuals).
//!
//! 1. **Headless only** — `headless::button` returns a
//!    `ButtonProps`; the caller paints a raw `div()`. The
//!    button is just a focus + click handler.
//! 2. **Headless + default-renderer** — same factory, but
//!    `.render(cx)` reads the registered
//!    `TokenButtonRenderer` and applies the default look.
//! 3. **Headless + caller custom** — same factory, no
//!    `default_render`; the caller owns bg / border / padding /
//!    radius / hover / active, bypassing the renderer entirely.
//! 4. **Default renderer also covers inputs** — same idea as
//!    panel 2, but for `headless::text_input`. Proves the
//!    headless / renderer split is not button-specific.
//!
//! ## Theme
//!
//! The demo loads a *demo-local* black/white/grey theme via
//! `default_renderer::install_with`, not the bundled
//! `system-light`. The default system's `action.primary`
//! is `#121214 → #0C0C0D → #000000` — a 2.6% lightness
//! delta that is below human perception. The demo theme
//! uses a mid-grey neutral action with a 10% lightness
//! delta (`#6E6E74 → #84848A → #5A5A60`) so hover and
//! active are visually clear.

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui_default_renderer as default_renderer;
use yororen_ui_default_renderer::Theme;

mod layers_app;
mod material_button;

// Modern black/white/grey theme for the layers demo. The
// palette is monochrome — no hues — with consistent
// `bg → hover_bg → active_bg` lightness transitions
// (~8% up, ~5% down) so interactive feedback is
// immediately visible. This is the look the user wanted;
// not the bundled `system-light` (which uses a 2.6%
// delta on `action.primary` and is below human
// perception).
const DEMO_THEME_JSON: &str = r##"{
  "surface": {
    "base": "#FFFFFF",
    "canvas": "#FAFAFA",
    "raised": "#FFFFFF",
    "sunken": "#F5F5F5",
    "hover": "#F0F0F0"
  },
  "content": {
    "primary": "#0A0A0A",
    "secondary": "#5C5C5C",
    "tertiary": "#9E9E9E",
    "disabled": "#C8C8C8",
    "on_primary": "#FFFFFF",
    "on_status": "#FFFFFF"
  },
  "border": {
    "default": "#E5E5E5",
    "muted": "#F0F0F0",
    "focus": "#0A0A0A",
    "divider": "#F0F0F0"
  },
  "action": {
    "neutral": {
      "bg": "#0A0A0A",
      "hover_bg": "#2A2A2A",
      "active_bg": "#1A1A1A",
      "fg": "#FFFFFF",
      "disabled_bg": "#C8C8C8",
      "disabled_fg": "#FFFFFF"
    },
    "primary": {
      "bg": "#0A0A0A",
      "hover_bg": "#2A2A2A",
      "active_bg": "#1A1A1A",
      "fg": "#FFFFFF",
      "disabled_bg": "#C8C8C8",
      "disabled_fg": "#FFFFFF"
    },
    "danger": {
      "bg": "#FFFFFF",
      "hover_bg": "#F5F5F5",
      "active_bg": "#E5E5E5",
      "fg": "#0A0A0A",
      "disabled_bg": "#F5F5F5",
      "disabled_fg": "#9E9E9E"
    }
  },
  "tokens": {
    "control": {
      "input":             { "min_height": 36, "horizontal_padding": 12, "vertical_padding": 8 },
      "button":            { "min_height": 36, "icon_button_min_size": 32, "horizontal_padding": 16, "vertical_padding": 8,  "icon_gap": 8, "radius": 6 }
    },
    "radii":  { "md": 6 }
  }
}"##;

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Bind the text-input keymap once at startup. The
        // renderer wires the actions (Backspace, Delete, Left,
        // Right, Enter, etc.); this call only registers the
        // keymap against the "UITextInput" key context
        // (idempotent — see `headless::text_input::init`).
        yororen_ui::headless::text_input::init(cx);

        // Demo-local theme: black/white/grey, with action
        // palette hover deltas of ~10% lightness so the
        // interactive states are visually obvious. The
        // shared `system-light.json` ships a darker default
        // (action.primary 2.6% delta) that the demo
        // overrides here.
        let theme = Theme::from_json(DEMO_THEME_JSON).expect("DEMO_THEME_JSON is valid");
        default_renderer::install_with(cx, theme);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(800.0), px(600.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| layers_app::LayersApp::new()));
    });
}
