//! `WinUIShortcutHintRenderer` — default `ShortcutHintRenderer` impl.

use std::sync::Arc;

use gpui::{App, Div, Hsla, InteractiveElement, ParentElement, Pixels, Stateful, Styled, div, px};

use yororen_ui_core::headless::shortcut_hint::ShortcutHintProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::shortcut_hint::{ShortcutHintRenderState, ShortcutHintRenderer};

pub struct WinUIShortcutHintRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIShortcutHintRenderer {
    pub fn label_fg(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.secondary").unwrap_or_default()
    }
    pub fn kbd_bg(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn kbd_fg(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn padding_x(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_x")
            .unwrap_or(6.0) as f32)
    }
    pub fn padding_y(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.kbd_padding_y")
            .unwrap_or(2.0) as f32)
    }
    pub fn radius(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
    pub fn key_gap(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.keybinding_input.separator_gap")
            .unwrap_or(4.0) as f32)
    }
    pub fn label_gap(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0) as f32)
    }
    pub fn font_size(&self, _state: &ShortcutHintRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.typography.font_size_sm")
            .unwrap_or(12.0) as f32)
    }
}

impl ShortcutHintRenderer for WinUIShortcutHintRenderer {
    fn compose(&self, props: &ShortcutHintProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ShortcutHintRenderState {};
        let label_fg = self.label_fg(&state, theme);
        let kbd_bg = self.kbd_bg(&state, theme);
        let kbd_fg = self.kbd_fg(&state, theme);
        let px_h = self.padding_x(&state, theme);
        let px_v = self.padding_y(&state, theme);
        let r = self.radius(&state, theme);
        let key_g = self.key_gap(&state, theme);
        let label_g = self.label_gap(&state, theme);
        let fs = self.font_size(&state, theme);

        let mut keys_row = div().flex().flex_row().items_center().gap(key_g);
        for key in &props.keys {
            keys_row = keys_row.child(
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

        div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(label_g)
            .child(
                div()
                    .text_color(label_fg)
                    .text_size(fs)
                    .child(props.label.clone()),
            )
            .child(keys_row)
    }
}

pub fn arc_shortcut_hint<T: ShortcutHintRenderer + 'static>(r: T) -> Arc<dyn ShortcutHintRenderer> {
    Arc::new(r)
}
