//! `WinUIRadioGroupRenderer` — default `RadioGroupRenderer` impl.
//!
//! Wraps radio buttons in a flex row with a theme-derived gap. The
//! headless layer owns the `selected_index` and change callbacks.

use std::sync::Arc;

use gpui::{App, Div, InteractiveElement, Pixels, Stateful, Styled, div};

use yororen_ui_core::headless::radio_group::RadioGroupProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::radio_group::{RadioGroupRenderState, RadioGroupRenderer};

pub struct WinUIRadioGroupRenderer;

impl WinUIRadioGroupRenderer {
    pub fn gap(&self, _state: &RadioGroupRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_2").unwrap_or(8.0) as f32)
    }
}

impl RadioGroupRenderer for WinUIRadioGroupRenderer {
    fn compose(&self, props: &RadioGroupProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = RadioGroupRenderState {
            selected_index: props.selected_index,
        };
        let gap = self.gap(&state, theme);

        div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
    }
}

pub fn arc_radio_group<T: RadioGroupRenderer + 'static>(r: T) -> Arc<dyn RadioGroupRenderer> {
    Arc::new(r)
}
