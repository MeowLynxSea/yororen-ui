//! `WinUIListboxRenderer` — default `ListboxRenderer` impl.
//!
//! Paints a vertical stack of option rows: the highlighted row
//! takes `surface.hover`, the selected row takes
//! `action.primary.bg` + `action.primary.fg`, and disabled
//! rows fade to `content.disabled`. Clicking a row fires
//! `state.pick(value, …)` which writes the value and invokes
//! the user-supplied `on_change` callback.
//!
//! The shell calls `track_focus(&state.focus_handle())` and
//! wires `on_key_down` so arrow keys / `Home` / `End` /
//! `Enter` move highlight and pick the highlighted row. The
//! keyboard nav is opt-in: a user who clicks the listbox
//! focuses it, then arrow keys work. A user who tabs into it
//! (Tab focus traversal) gets the same behaviour. The same
//! `ListNavigable` algorithm the headless layer uses drives
//! every transition — this renderer is just the input
//! surface.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, KeyDownEvent, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, px,
};

use gpui::prelude::FluentBuilder;
use yororen_ui_core::animation::AnimationConfig;
use yororen_ui_core::headless::list_navigable::ListNavigable;
use yororen_ui_core::headless::listbox::ListboxProps;
use yororen_ui_core::renderer::spec::Edges;

use crate::animation::{
    AnimatedStateElement, lerp_hsla, set_interaction_hovered,
};
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::listbox::{ListboxRenderState, ListboxRenderer};

pub struct WinUIListboxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIListboxRenderer {
    pub fn bg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn fg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn selected_fg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    pub fn disabled_fg(&self, _state: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or_default()
    }
    pub fn padding(&self, _state: &ListboxRenderState, theme: &Theme) -> Edges<Pixels> {
        // Reuse the list_item tokens so the listbox looks
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
    pub fn min_height(&self, _state: &ListboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.list_item.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &ListboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
    pub fn gap(&self, _state: &ListboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(2.0) as f32)
    }
}

impl ListboxRenderer for WinUIListboxRenderer {
    fn compose(&self, props: &ListboxProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let read = props.state.read(cx);
        let state = ListboxRenderState {
            row_count: read.options.len(),
        };
        let bg = self.bg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let selected_bg = self.selected_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let selected_fg = self.selected_fg(&state, theme);
        let disabled_fg = self.disabled_fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let gap = self.gap(&state, theme);

        let highlighted = read.highlighted_index;
        let selected_value = read.selected_value.clone();
        let options = read.options.clone();
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
            .flex()
            .flex_col()
            .gap(gap)
            .bg(bg)
            .rounded(r)
            .p(px(2.0));

        for (i, opt) in options.iter().enumerate() {
            let is_highlighted = highlighted == Some(i);
            let is_selected = selected_value.as_ref() == Some(&opt.value);
            let row_fg = if opt.disabled {
                disabled_fg
            } else if is_selected {
                selected_fg
            } else {
                fg
            };
            let row_bg = if is_selected {
                selected_bg
            } else if is_highlighted {
                hover_bg
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };
            let value_for_click = opt.value.clone();
            let row_id = ElementId::Name(format!("listbox-row-{}", i).into());
            let mut row: Stateful<Div> = div()
                .id(row_id.clone())
                .relative()
                .flex()
                .items_center()
                .text_color(row_fg)
                .px(pad.left)
                .py(pad.top)
                .min_h(h)
                .rounded(r)
                .when(!opt.disabled, |d| d.cursor(CursorStyle::PointingHand))
                .when(opt.disabled, |d| d.opacity(0.6));

            if !opt.disabled && !is_selected {
                let hov_id = row_id.clone();
                row = row.on_hover(move |hovered, _win, cx| {
                    set_interaction_hovered(cx, hov_id.clone(), *hovered);
                });
                let config = AnimationConfig::default().with_duration(Duration::from_millis(100));
                let fill = AnimatedStateElement::new(
                    (row_id.clone(), "fill"),
                    row_id.clone(),
                    false,
                    div().absolute().inset_0().rounded(r),
                    config,
                    move |d: Div, hover, _pressed, _checked| {
                        d.bg(lerp_hsla(row_bg, hover_bg, hover))
                    },
                );
                row = row.child(fill);
            } else {
                row = row.bg(row_bg);
            }

            row = row.child(opt.label.to_string());
            if !opt.disabled {
                let state_for_this_row = state_for_click.clone();
                row = row.on_click(move |_ev, window, cx| {
                    state_for_this_row.update(cx, |s, cx_inner| {
                        s.pick(value_for_click.clone(), window, &mut *cx_inner);
                    });
                });
            }
            body = body.child(row);
        }

        // Keyboard nav — drives the same `ListNavigable` algorithm
        // the headless layer uses, so highlight / skip-disabled
        // semantics are identical to clicking. `track_focus` makes
        // `on_key_down` fire when the listbox has focus (after
        // click or Tab). `Enter` calls `select_highlighted` to
        // commit the highlighted row.
        //
        // `window.refresh()` is required because `compose` runs
        // outside a paint context: `props.state.read(cx)` during
        // compose does NOT register a paint-time subscription, so
        // a state-only update (no GalleryApp field changes) would
        // otherwise never redraw. `window.refresh()` marks the
        // window dirty so the next frame re-invokes `render`,
        // which re-reads the state and re-paints. Mouse clicks
        // don't need this because `on_change` mutates a
        // GalleryApp field, which already triggers re-render via
        // the entity observer graph.
        let state_for_keys = state_for_click.clone();
        div()
            .id(props.id.clone())
            .track_focus(&focus_handle)
            .on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                let handled = match ks.key.as_str() {
                    "down" => {
                        state_for_keys.update(cx, |s, _cx| s.highlight_next());
                        true
                    }
                    "up" => {
                        state_for_keys.update(cx, |s, _cx| s.highlight_prev());
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
                    // `listbox_state` at paint time, so the
                    // window has no reason to redraw.
                    window.refresh();
                }
            })
            .child(body)
    }
}

pub fn arc_listbox<T: ListboxRenderer + 'static>(r: T) -> Arc<dyn ListboxRenderer> {
    Arc::new(r)
}
