//! `TokenPipsPagerRenderer` — default `PipsPagerRenderer` impl.
//!
//! Renders the elided pip strip from `PipsPagerState::pip_slots`:
//! one `rounded_full` dot per `PipSlot::Page` (the current page's
//! dot elongates into a pill, matching WinUI), `⋯` glyphs for the
//! ellipsis slots, and — when `show_nav_buttons` — chevron buttons
//! flanking the strip. Clicking a pip fires `state.go_to(page, …)`;
//! clicking an ellipsis fires `skip_backward` / `skip_forward`
//! (a whole window per click).
//!
//! The shell calls `track_focus(&state.focus_handle())` and wires
//! `on_key_down` so ← / → (or ↑ / ↓ when vertical) / `Home` / `End`
//! drive the same `next` / `prev` / `go_to` methods the click
//! handlers use.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, KeyDownEvent, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, hsla, px,
};

use gpui::prelude::FluentBuilder;
use yororen_ui_core::headless::pips_pager::{PipSlot, PipsPagerOrientation, PipsPagerProps};
use yororen_ui_core::renderer::pips_pager::{PipsPagerRenderState, PipsPagerRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenPipsPagerRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenPipsPagerRenderer {
    pub fn active_color(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.bg").unwrap_or_default()
    }
    pub fn inactive_color(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("border.default")
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 0.3))
    }
    pub fn fg(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    pub fn dot_size(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.dot_size")
            .unwrap_or(6.0) as f32)
    }
    /// The current page's dot elongates into a pill of this length.
    pub fn active_dot_size(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.active_dot_size")
            .unwrap_or(16.0) as f32)
    }
    pub fn gap(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.gap")
            .unwrap_or(8.0) as f32)
    }
    pub fn nav_button_size(&self, _state: &PipsPagerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.pips_pager.nav_button_size")
            .unwrap_or(24.0) as f32)
    }
}

impl PipsPagerRenderer for TokenPipsPagerRenderer {
    fn compose(&self, props: &PipsPagerProps, cx: &App) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let read = props.state.read(cx);
        let state = PipsPagerRenderState {
            page_count: read.page_count,
            current_page: read.current_page,
            orientation: read.orientation,
            max_visible_pips: read.max_visible_pips,
            show_nav_buttons: read.show_nav_buttons,
            ellipsis_before_elides: read.ellipsis_before_elides(),
            ellipsis_after_elides: read.ellipsis_after_elides(),
        };
        let slots = read.pip_slots();
        let vertical = read.orientation == PipsPagerOrientation::Vertical;
        // `Entity` is `Clone`; cloning releases the read borrow so
        // the closure bodies below can call `state.update(…)`.
        let state_for_click = props.state.clone();
        let focus_handle = read.focus_handle();
        let _ = read;

        let active = self.active_color(&state, theme);
        let inactive = self.inactive_color(&state, theme);
        let fg = self.fg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let dot = self.dot_size(&state, theme);
        let active_dot = self.active_dot_size(&state, theme);
        let gap = self.gap(&state, theme);
        let nav_size = self.nav_button_size(&state, theme);

        let id_str = format!("{:?}", props.id);

        let mut strip: gpui::Div = if vertical {
            div().flex().flex_col()
        } else {
            div().flex().flex_row()
        }
        .gap(px(4.))
        .items_center();

        for slot in slots {
            match slot {
                PipSlot::Page(page) => {
                    let is_current = page == state.current_page;
                    let state_for_pip = state_for_click.clone();
                    let mut pip = div()
                        .id(ElementId::Name(format!("{id_str}-pip-{page}").into()))
                        .rounded_full()
                        .cursor(CursorStyle::PointingHand)
                        .bg(if is_current { active } else { inactive })
                        .when(is_current, |p| p.hover(move |s| s.bg(active)))
                        .when(!is_current, |p| {
                            let hover = active;
                            p.hover(move |s| s.bg(hover))
                        });
                    if vertical {
                        pip = pip.w(dot).h(if is_current { active_dot } else { dot });
                    } else {
                        pip = pip.h(dot).w(if is_current { active_dot } else { dot });
                    }
                    pip = pip.on_click(move |_ev, window, cx| {
                        state_for_pip.update(cx, |s, cx_inner| {
                            s.go_to(page, window, &mut *cx_inner);
                        });
                    });
                    strip = strip.child(pip);
                }
                PipSlot::EllipsisBefore | PipSlot::EllipsisAfter => {
                    let before = slot == PipSlot::EllipsisBefore;
                    // A slot at the very edge stands for an *empty*
                    // run — rendered purely to keep the control's
                    // width constant, so it must be inert.
                    let elides = if before {
                        state.ellipsis_before_elides
                    } else {
                        state.ellipsis_after_elides
                    };
                    let state_for_ellipsis = state_for_click.clone();
                    let mut ellipsis = div()
                        .id(ElementId::Name(
                            format!(
                                "{id_str}-ellipsis-{}",
                                if before { "before" } else { "after" }
                            )
                            .into(),
                        ))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(inactive)
                        .text_size(dot * 1.2)
                        .when(vertical, |e| e.h(active_dot).w(dot))
                        .when(!vertical, |e| e.w(active_dot).h(dot))
                        .child("⋯");
                    if elides {
                        ellipsis = ellipsis.cursor(CursorStyle::PointingHand).on_click(
                            move |_ev, window, cx| {
                                state_for_ellipsis.update(cx, |s, cx_inner| {
                                    if before {
                                        s.skip_backward(window, &mut *cx_inner);
                                    } else {
                                        s.skip_forward(window, &mut *cx_inner);
                                    }
                                });
                            },
                        );
                    } else {
                        ellipsis = ellipsis.opacity(0.3);
                    }
                    strip = strip.child(ellipsis);
                }
            }
        }

        // Nav buttons — chevrons matching the ComboBox trigger
        // glyph style. Disabled-looking (faded, no-op) at the ends;
        // the headless `prev` / `next` already refuse to wrap.
        let make_nav = |label: &'static str, forward: bool, enabled: bool| {
            let state_for_nav = state_for_click.clone();
            div()
                .id(ElementId::Name(
                    format!("{id_str}-nav-{}", if forward { "next" } else { "prev" }).into(),
                ))
                .size(nav_size)
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .text_color(fg)
                .text_size(dot * 1.6)
                .when(enabled, |b| {
                    b.cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.bg(hover_bg))
                })
                .when(!enabled, |b| b.opacity(0.35))
                .on_click(move |_ev, window, cx| {
                    state_for_nav.update(cx, |s, cx_inner| {
                        if forward {
                            s.next(window, &mut *cx_inner);
                        } else {
                            s.prev(window, &mut *cx_inner);
                        }
                    });
                })
                .child(label)
        };

        let at_first = state.current_page == 0 || state.page_count == 0;
        let at_last = state.page_count == 0 || state.current_page + 1 >= state.page_count;
        let prev_label = if vertical { "▴" } else { "‹" };
        let next_label = if vertical { "▾" } else { "›" };
        let prev = make_nav(prev_label, false, !at_first);
        let next = make_nav(next_label, true, !at_last);

        // Keyboard nav — same methods as the click handlers.
        // `window.refresh()` is required because `compose` runs
        // outside a paint context; without it a state-only update
        // never redraws (see the grid-view renderer for the full
        // rationale).
        let state_for_keys = state_for_click;
        div()
            .id(props.id.clone())
            .track_focus(&focus_handle)
            .flex()
            .items_center()
            .gap(gap)
            .when(vertical, |s| s.flex_col())
            .when(state.show_nav_buttons, |s| s.child(prev))
            .child(strip)
            .when(state.show_nav_buttons, |s| s.child(next))
            .on_key_down(move |ev: &KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                let handled = match ks.key.as_str() {
                    "right" if !vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.next(window, &mut *cx_inner));
                        true
                    }
                    "left" if !vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.prev(window, &mut *cx_inner));
                        true
                    }
                    "down" if vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.next(window, &mut *cx_inner));
                        true
                    }
                    "up" if vertical => {
                        state_for_keys.update(cx, |s, cx_inner| s.prev(window, &mut *cx_inner));
                        true
                    }
                    "home" => {
                        state_for_keys.update(cx, |s, cx_inner| {
                            s.go_to(0, window, &mut *cx_inner);
                        });
                        true
                    }
                    "end" => {
                        state_for_keys.update(cx, |s, cx_inner| {
                            let last = s.page_count.saturating_sub(1);
                            s.go_to(last, window, &mut *cx_inner);
                        });
                        true
                    }
                    _ => false,
                };
                if handled {
                    window.refresh();
                }
            })
    }
}

pub fn arc_pips_pager<T: PipsPagerRenderer + 'static>(r: T) -> Arc<dyn PipsPagerRenderer> {
    Arc::new(r)
}
