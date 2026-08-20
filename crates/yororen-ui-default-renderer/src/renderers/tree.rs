//! `TokenTreeRenderer` — default `TreeRenderer` impl.
//!
//! Wraps tree rows in a flex column with a subtle border and
//! theme-derived gap. Individual rows are rendered by
//! `TreeItemRenderer`.

use std::sync::Arc;

use gpui::{App, Div, Hsla, InteractiveElement, Pixels, Stateful, Styled, div};

use yororen_ui_core::headless::tree::TreeProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::tree::{TreeRenderState, TreeRenderer};

pub struct TokenTreeRenderer;

impl TokenTreeRenderer {
    pub fn border_color(&self, _state: &TreeRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn gap(&self, _state: &TreeRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
    pub fn border_radius(&self, _state: &TreeRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
}

impl TreeRenderer for TokenTreeRenderer {
    fn compose(&self, props: &TreeProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = TreeRenderState {
            has_selection: props.selected.is_some() || !props.selected_ids.is_empty(),
            multi_select: props.selection_mode
                == yororen_ui_core::headless::tree::TreeSelectionMode::Multiple,
        };
        let border = self.border_color(&state, theme);
        let gap = self.gap(&state, theme);
        let radius = self.border_radius(&state, theme);

        div()
            .id(props.id.clone())
            .flex()
            .flex_col()
            .gap(gap)
            .rounded(radius)
            .border_1()
            .border_color(border)
    }
}

pub fn arc_tree<T: TreeRenderer + 'static>(r: T) -> Arc<dyn TreeRenderer> {
    Arc::new(r)
}
