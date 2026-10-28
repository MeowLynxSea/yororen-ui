//! `WinUIPipsPagerRenderer` — winui `PipsPagerRenderer` impl.
//!
//! Same structure as the default renderer (pip strip from
//! `pip_slots`, chevron nav buttons, keyboard nav — see
//! `yororen-ui-default-renderer::renderers::pips_pager` for the
//! full commentary), plus the Fluent touch: each pip is wrapped in
//! [`AnimatedStateElement`] so the active pill grows to its long
//! form and takes the accent colour with a 167 ms
//! fast-out-slow-in tween (the same treatment the checkbox gives
//! its check glyph), while hover warms an inactive pip halfway
//! toward the accent.

use std::sync::Arc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, KeyDownEvent, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Styled, div, hsla, px,
};

use gpui::prelude::FluentBuilder;
use yororen_ui_core::headless::pips_pager::{PipSlot, PipsPagerOrientation, PipsPagerProps};
use yororen_ui_core::renderer::pips_pager::{PipsPagerRenderState, PipsPagerRenderer};
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedStateElement, control_config, lerp_hsla, set_interaction_hovered};

pub struct WinUIPipsPagerRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIPipsPagerRenderer {
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

impl PipsPagerRenderer for WinUIPipsPagerRenderer {
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
        let config = control_config(theme);

        let id_str = format!("{:?}", props.id);

        let mut strip: Div = if vertical {
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
                    let pip_id = ElementId::Name(format!("{id_str}-pip-{page}").into());

                    // Interaction lives on the inner stateful div (the
                    // animation wrapper only implements `Element`); the
                    // wrapper then tweens size + colour: the `checked`
                    // channel grows the pip into the active pill and takes
                    // the accent colour, hover warms an inactive pip
                    // halfway.
                    let hov_id = pip_id.clone();
                    let pip_body = div()
                        .id(pip_id.clone())
                        .rounded_full()
                        .cursor(CursorStyle::PointingHand)
                        .on_click(move |_ev, window, cx| {
                            state_for_pip.update(cx, |s, cx_inner| {
                                s.go_to(page, window, &mut *cx_inner);
                            });
                        })
                        .on_hover(move |hovered, _win, cx| {
                            set_interaction_hovered(cx, hov_id.clone(), *hovered);
                        });
                    let pip_f32: f32 = dot.into();
                    let active_f32: f32 = active_dot.into();
                    // NOTE: size is NOT animated — only the color/HOFF
                    // warmth runs through the tween. Animating width on
                    // the `checked` channel is asymmetric: a brand-new
                    // current pip starts at full length while the old one
                    // tweens down, so the strip briefly widens ~
                    // (active - inactive) px on every page change.
                    let pip = AnimatedStateElement::new(
                        (pip_id.clone(), "pip"),
                        pip_id.clone(),
                        is_current,
                        pip_body,
                        config.clone(),
                        move |d: Stateful<Div>, hover, _pressed, checked| {
                            let color = lerp_hsla(inactive, active, checked.max(hover * 0.5));
                            let long = if checked > 0.5 { active_f32 } else { pip_f32 };
                            let sized = if vertical {
                                d.w(px(pip_f32)).h(px(long))
                            } else {
                                d.w(px(long)).h(px(pip_f32))
                            };
                            sized.bg(color)
                        },
                    );
                    strip = strip.child(pip);
                }
                PipSlot::EllipsisBefore | PipSlot::EllipsisAfter => {
                    let before = slot == PipSlot::EllipsisBefore;
                    // Edge slots are rendered but inert (empty run) —
                    // kept only so the control width stays constant.
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
