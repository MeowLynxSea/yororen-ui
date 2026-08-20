//! `TokenGridViewRenderer` — default `GridViewRenderer` impl.
//!
//! Paints the options as a CSS grid with `columns` equal-width
//! tracks (`grid-template-columns: repeat(n, minmax(0, 1fr))`
//! under the hood — Taffy auto-placement gives the short last
//! row correctly-sized tiles instead of stretching them). The
//! highlighted tile takes `surface.hover`, the selected tile
//! takes `action.primary.bg` + `action.primary.fg`, and disabled
//! tiles fade to `content.disabled`. Clicking a tile fires
//! `state.pick(value, …)` which writes the value and invokes
//! the user-supplied `on_change` callback.
//!
//! The shell calls `track_focus(&state.focus_handle())` and
//! wires `on_key_down` so ← / → / ↑ / ↓ / `Home` / `End` /
//! `Enter` move highlight and pick the highlighted tile. ← / →
//! reuse the shared `ListNavigable` wrap-around walk; ↑ / ↓
//! step a whole `columns` stride without wrapping. The same
//! algorithms the headless layer uses drive every transition —
//! this renderer is just the input surface.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, KeyDownEvent, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use gpui::prelude::FluentBuilder;
use yororen_ui_core::headless::grid_view::GridViewProps;
use yororen_ui_core::headless::list_navigable::ListNavigable;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::grid_view::{GridViewRenderState, GridViewRenderer};

pub struct TokenGridViewRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenGridViewRenderer {
    pub fn bg(&self, _state: &GridViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &GridViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &GridViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn fg(&self, _state: &GridViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn selected_fg(&self, _state: &GridViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    pub fn disabled_fg(&self, _state: &GridViewRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or_default()
    }
    pub fn padding(&self, _state: &GridViewRenderState, theme: &Theme) -> Edges<Pixels> {
        // Reuse the list_item tokens so the grid looks
        // consistent with the rest of the lists surfaces.
        Edges::symmetric(
            gpui::px(
                theme
                    .get_number("tokens.control.list_item.horizontal_padding")
                    .unwrap_or_else(|| theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0))
                    as f32,
            ),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32),
        )
    }
    pub fn min_tile_size(&self, _state: &GridViewRenderState, theme: &Theme) -> Pixels {
        // Tiles are wider than they are tall; the height floor
        // keeps single-line labels comfortable while the width
        // comes from the equal-width grid tracks.
        gpui::px(
            theme
                .get_number("tokens.control.list_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &GridViewRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
    pub fn gap(&self, _state: &GridViewRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(2.0) as f32)
    }
}

impl GridViewRenderer for TokenGridViewRenderer {
    fn compose(&self, props: &GridViewProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let read = props.state.read(cx);
        let state = GridViewRenderState {
            item_count: read.options.len(),
            columns: read.columns,
        };
        let bg = self.bg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let selected_bg = self.selected_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let selected_fg = self.selected_fg(&state, theme);
        let disabled_fg = self.disabled_fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_tile_size(&state, theme);
        let r = self.border_radius(&state, theme);
        let gap = self.gap(&state, theme);

        let highlighted = read.highlighted_index;
        let selected_value = read.selected_value.clone();
        let options = read.options.clone();
        let columns = read.columns.max(1);
        // `Entity` is `Clone`; cloning releases the read borrow
        // so the closure body below can call `state.update(…)`
        // for the click handlers and the keyboard handler.
        let state_for_click = props.state.clone();
        // Clone the focus handle out of the read borrow so the
        // shell can `track_focus` it and the key handler can
        // also reference it.
        let focus_handle = read.focus_handle();
        let _ = read;

        let mut body: Div = div()
            .grid()
            .grid_cols(columns as u16)
            .gap(gap)
            .bg(bg)
            .rounded(r)
            .p(px(2.0));

        for (i, opt) in options.iter().enumerate() {
            let is_highlighted = highlighted == Some(i);
            let is_selected = selected_value.as_ref() == Some(&opt.value);
            let tile_fg = if opt.disabled {
                disabled_fg
            } else if is_selected {
                selected_fg
            } else {
                fg
            };
            let tile_bg = if is_selected {
                selected_bg
            } else if is_highlighted {
                hover_bg
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };
            let value_for_click = opt.value.clone();
            let mut tile: Stateful<Div> = div()
                .id(ElementId::Name(format!("gridview-tile-{}", i).into()))
                .flex()
                .items_center()
                .justify_center()
                .text_center()
                .bg(tile_bg)
                .text_color(tile_fg)
                .px(pad.left)
                .py(pad.top)
                .min_h(h)
                .rounded(r)
                .when(!opt.disabled, |d| d.cursor(CursorStyle::PointingHand))
                .when(opt.disabled, |d| d.opacity(0.6))
                .child(opt.label.to_string());
            if !opt.disabled {
                let state_for_this_tile = state_for_click.clone();
                tile = tile.on_click(move |_ev, window, cx| {
                    state_for_this_tile.update(cx, |s, cx_inner| {
                        s.pick(value_for_click.clone(), window, &mut *cx_inner);
                    });
                });
            }
            body = body.child(tile);
        }

        // Keyboard nav — drives the same algorithms the headless
        // layer uses, so highlight / skip-disabled semantics are
        // identical to clicking. `track_focus` makes
        // `on_key_down` fire when the grid has focus (after
        // click or Tab). `Enter` calls `select_highlighted` to
        // commit the highlighted tile.
        //
        // `window.refresh()` is required because `compose` runs
        // outside a paint context: `props.state.read(cx)` during
        // compose does NOT register a paint-time subscription, so
        // a state-only update (no GalleryApp field changes) would
        // otherwise never redraw. `window.refresh()` marks the
        // window dirty so the next frame re-invokes `render`,
        // which re-reads the state and re-paints.
        let state_for_keys = state_for_click.clone();
        div()
            .id(props.id.clone())
            .track_focus(&focus_handle)
            .on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                let handled = match ks.key.as_str() {
                    "right" => {
                        state_for_keys.update(cx, |s, _cx| s.highlight_next());
                        true
                    }
                    "left" => {
                        state_for_keys.update(cx, |s, _cx| s.highlight_prev());
                        true
                    }
                    "down" => {
                        state_for_keys.update(cx, |s, _cx| s.highlight_down());
                        true
                    }
                    "up" => {
                        state_for_keys.update(cx, |s, _cx| s.highlight_up());
                        true
                    }
                    "home" => {
                        state_for_keys.update(cx, |s, _cx| {
                            if let Some(i) = (0..s.options.len()).find(|&i| s.is_selectable(i)) {
                                s.set_highlighted(i);
                            }
                        });
                        true
                    }
                    "end" => {
                        state_for_keys.update(cx, |s, _cx| {
                            if let Some(i) =
                                (0..s.options.len()).rev().find(|&i| s.is_selectable(i))
                            {
                                s.set_highlighted(i);
                            }
                        });
                        true
                    }
                    "enter" => {
                        state_for_keys.update(cx, |s, cx_inner| {
                            s.select_highlighted(window, &mut *cx_inner);
                        });
                        true
                    }
                    _ => false,
                };
                if handled {
                    // Mark the window dirty so the next paint
                    // re-runs `render` and re-reads the state.
                    // Without this the highlight change is
                    // invisible: `compose` ran once during
                    // `.render(cx)` and nothing observes
                    // `gridview_state` at paint time, so the
                    // window has no reason to redraw.
                    window.refresh();
                }
            })
            .child(body)
    }
}

pub fn arc_grid_view<T: GridViewRenderer + 'static>(r: T) -> Arc<dyn GridViewRenderer> {
    Arc::new(r)
}
