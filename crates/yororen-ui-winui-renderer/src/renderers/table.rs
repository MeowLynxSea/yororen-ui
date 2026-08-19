//! `WinUITableRenderer` — default `TableRenderer` impl.
//!
//! Paints a table with header row, body rows, selected-row highlight,
//! and column widths. All colours and spacing come from the theme.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, Hsla, InteractiveElement, ParentElement, Pixels, SharedString, Stateful,
    Styled, div, px,
};

use yororen_ui_core::headless::table::{TableColumn, TableProps};
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::table::{TableRenderState, TableRenderer};

pub struct WinUITableRenderer;

impl WinUITableRenderer {
    pub fn header_bg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.sunken").unwrap_or_default()
    }
    pub fn header_text_color(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.secondary").unwrap_or_default()
    }
    pub fn row_text_color(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn border_color(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn cell_padding(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0) as f32)
    }
    pub fn font_size(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.typography.font_size_sm")
            .unwrap_or(12.0) as f32)
    }
    pub fn border_radius(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }

    fn header_cell(&self, col: &TableColumn, state: &TableRenderState, theme: &Theme) -> Div {
        let mut cell = div()
            .child(col.header.clone())
            .text_color(self.header_text_color(state, theme))
            .text_size(self.font_size(state, theme))
            .px(self.cell_padding(state, theme))
            .py(self.cell_padding(state, theme))
            .flex()
            .items_center();
        if let Some(w) = col.width_px {
            cell = cell.w(px(w));
        } else {
            cell = cell.flex_1();
        }
        cell
    }

    fn body_cell(&self, value: &SharedString, state: &TableRenderState, theme: &Theme) -> Div {
        div()
            .child(value.clone())
            .text_color(self.row_text_color(state, theme))
            .text_size(self.font_size(state, theme))
            .px(self.cell_padding(state, theme))
            .py(self.cell_padding(state, theme))
            .flex()
            .items_center()
            .flex_1()
    }
}

impl TableRenderer for WinUITableRenderer {
    fn compose(&self, props: &TableProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = TableRenderState {
            selected_row: props.selected_row,
        };
        let header_bg = self.header_bg(&state, theme);
        let border = self.border_color(&state, theme);
        let selected_bg = self.selected_bg(&state, theme);
        let _pad = self.cell_padding(&state, theme);
        let radius = self.border_radius(&state, theme);

        // Header row
        let mut header = div()
            .flex()
            .flex_row()
            .bg(header_bg)
            .border_b_1()
            .border_color(border);
        for col in &props.columns {
            header = header.child(self.header_cell(col, &state, theme));
        }

        // Body rows
        let mut body = div().flex().flex_col();
        for (row_idx, row) in props.rows.iter().enumerate() {
            let is_selected = props.selected_row == Some(row_idx);
            let mut row_el = div()
                .flex()
                .flex_row()
                .cursor(if props.on_select_row.is_some() {
                    CursorStyle::PointingHand
                } else {
                    CursorStyle::Arrow
                });
            if is_selected {
                row_el = row_el.bg(selected_bg);
            }
            for (cell_idx, cell_value) in row.iter().enumerate() {
                let mut cell = self.body_cell(cell_value, &state, theme);
                if let Some(col) = props.columns.get(cell_idx)
                    && let Some(w) = col.width_px
                {
                    cell = cell.w(px(w));
                }
                row_el = row_el.child(cell);
            }
            body = body.child(row_el);
        }

        div()
            .id(props.id.clone())
            .flex()
            .flex_col()
            .rounded(radius)
            .border_1()
            .border_color(border)
            .overflow_hidden()
            .child(header)
            .child(body)
    }
}

pub fn arc_table<T: TableRenderer + 'static>(r: T) -> Arc<dyn TableRenderer> {
    Arc::new(r)
}
