//! yororen-ui Inputs Demo
//!
//! One window, vertical scroll, one panel per input component:
//!
//! 1. `text_input` — single-line text with on_change
//! 2. `password_input` — masked input
//! 3. `number_input` — numeric with + / - stepper
//! 4. `search_input` — search with clear button
//! 5. `file_path_input` — file path with browse button
//! 6. `keybinding_input` — keybinding capture (Idle / Capturing)
//! 7. `text_area` — multi-line text
//!
//! ## Theme
//!
//! Uses the modern black/white/grey palette from the layers demo
//! (`#0A0A0A` primary, `#2A2A2A` hover, `#1A1A1A` active,
//! `#E5E5E5` border, etc.) so the input borders and hover
//! deltas are visually obvious.

use gpui::{App, AppContext, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui_default_renderer as default_renderer;
use yororen_ui_default_renderer::Theme;

mod inputs_app;

// Modern black/white/grey theme. Mirrors the layers-demo theme
// so the input borders (default → hover → focus) are clearly
// visible against the surface.
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
      "input":              { "min_height": 36, "horizontal_padding": 12, "vertical_padding": 8 },
      "search_input":       { "min_height": 36, "horizontal_padding": 12, "input_gap": 8, "icon_size": 16 },
      "number_input":       { "min_height": 36, "horizontal_padding": 12, "stepper_button_size": 32 },
      "file_path_input":    { "min_height": 36, "horizontal_padding": 12, "action_gap": 8, "icon_size": 16 },
      "keybinding_input":   { "icon_size": 18, "kbd_padding_x": 8, "kbd_padding_y": 4, "kbd_min_width": 80 },
      "button":             { "min_height": 36, "icon_button_min_size": 32, "horizontal_padding": 16, "vertical_padding": 8, "icon_gap": 8, "radius": 6 }
    },
    "sizes": { "icon_sm": 16 },
    "radii":  { "md": 6 }
  }
}"##;

fn main() {
    let app = gpui_ce_platform::application().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // Bind the text-input keymap once at startup. The
        // renderer wires the actions; this call only registers
        // the keymap (idempotent — see
        // `headless::text_input::init`).
        yororen_ui::headless::text_input::init(cx);

        // Demo-local theme: black/white/grey, with action
        // palette hover deltas of ~8% lightness so the
        // interactive states are visually obvious.
        let theme = Theme::from_json(DEMO_THEME_JSON).expect("valid theme JSON");
        default_renderer::install_with(cx, theme);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(900.0), px(900.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| inputs_app::InputsApp::new()));
    });
}
