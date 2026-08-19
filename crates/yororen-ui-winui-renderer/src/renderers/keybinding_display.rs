//! `WinUIKeybindingDisplayRenderer` — default
//! `KeybindingDisplayRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Stateful, Styled, div, px};

use yororen_ui_core::headless::keybinding_display::KeybindingDisplayProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::keybinding_display::{
    KeybindingDisplayRenderState, KeybindingDisplayRenderer,
};

pub struct WinUIKeybindingDisplayRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIKeybindingDisplayRenderer {
    pub fn kbd_bg(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn kbd_fg(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn padding_x(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_x")
            .unwrap_or(6.0) as f32)
    }
    pub fn padding_y(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_y")
            .unwrap_or(2.0) as f32)
    }
    pub fn radius(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
    pub fn gap(&self, _state: &KeybindingDisplayRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.separator_gap")
            .unwrap_or(4.0) as f32)
    }
    pub fn font_size(&self, _state: &KeybindingDisplayRenderState, _theme: &Theme) -> Pixels {
        px(12.)
    }
}

impl KeybindingDisplayRenderer for WinUIKeybindingDisplayRenderer {
    fn compose(&self, props: &KeybindingDisplayProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = KeybindingDisplayRenderState {};
        let kbd_bg = self.kbd_bg(&state, theme);
        let kbd_fg = self.kbd_fg(&state, theme);
        let px_h = self.padding_x(&state, theme);
        let px_v = self.padding_y(&state, theme);
        let r = self.radius(&state, theme);
        let g = self.gap(&state, theme);
        let fs = self.font_size(&state, theme);
        let mut row = div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(g);
        for key in &props.keys {
            row = row.child(
                div()
                    .bg(kbd_bg)
                    .text_color(kbd_fg)
                    .rounded(r)
                    .px(px_h)
                    .py(px_v)
                    .text_size(fs)
                    .child(key.clone()),
            );
        }
        row
    }
}

pub fn arc_keybinding_display<T: KeybindingDisplayRenderer + 'static>(
    r: T,
) -> Arc<dyn KeybindingDisplayRenderer> {
    Arc::new(r)
}
