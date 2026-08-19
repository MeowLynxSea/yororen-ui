//! `WinUISelectRenderer` — default `SelectRenderer` impl.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::animation::{AnimationConfig, SlideDirection};
use yororen_ui_core::headless::select::SelectProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedPresenceElement, AnimatedStateElement, animated_input_border, lerp_hsla,
    set_interaction_hovered,
};

pub use yororen_ui_core::renderer::select::{SelectRenderState, SelectRenderer};

pub struct WinUISelectRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUISelectRenderer {
    pub fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    pub fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    pub fn focus_border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    pub fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.custom_fg.is_some() {
            state.custom_fg.unwrap()
        } else if state.has_value {
            theme.get_color("content.primary").unwrap_or_default()
        } else {
            theme.get_color("content.tertiary").unwrap_or_default()
        }
    }
    pub fn hint_color(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    pub fn border_radius(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn chevron_rotation(&self, state: &SelectRenderState, _theme: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

impl SelectRenderer for WinUISelectRenderer {
    fn compose(&self, props: &SelectProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state_read = props.state.read(cx);
        let state = SelectRenderState {
            open: state_read.is_open(),
            disabled: false,
            has_value: state_read.value.is_some(),
            custom_bg: None,
            custom_border: None,
            custom_focus_border: None,
            custom_fg: None,
        };
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let value = state_read.value.clone();
        let options = state_read.options.clone();

        let display = if let Some(v) = &value {
            options
                .iter()
                .find(|o| &o.value == v)
                .map(|o| o.label.to_string())
                .unwrap_or_else(|| v.to_string())
        } else {
            state_read.placeholder.to_string()
        };

let side_border = theme.get_color("winui.ctrl_stroke").unwrap_or(border);
        let bottom_rest = theme.get_color("winui.ctrl_strong_stroke").unwrap_or(border);
        let bottom_focused = theme.get_color("winui.accent").unwrap_or(border);
        let bg_hover = theme.get_color("winui.ctrl_fill_hover").unwrap_or(bg);
        let bg_focused = theme.get_color("surface.sunken").unwrap_or(bg);
        let is_open = state_read.is_visible();

        let state_for_toggle = props.state.clone();
        let mut trigger: Stateful<Div> = div()
            .flex()
            .items_center()
            .bg(bg)
            .border_1()
            .border_color(border)
            .text_color(fg)
            .px(pad.left)
            .py(pad.top)
            .min_h(h)
            .rounded(r)
            .cursor(CursorStyle::PointingHand)
            .child(display)
            .id("default-select-trigger");
        trigger = trigger.on_hover({
            let id = props.id.clone();
            move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
        });
        trigger = trigger.on_click(move |_ev, _window, cx| {
            state_for_toggle.update(cx, |s, _cx| s.toggle());
        });

        let trigger = animated_input_border(
            trigger,
            props.id.clone(),
            props.id.clone(),
            side_border,
            bottom_rest,
            bottom_focused,
            bg,
            bg_hover,
            bg_focused,
            is_open,
        );

        let mut outer = div().relative().child(trigger);

        if state_read.is_visible() && !options.is_empty() {
            let h_f32: f32 = h.into();
            let state_for_close = props.state.clone();
            let mut dropdown: Stateful<Div> = div()
                .id("default-select-dropdown")
                .absolute()
                .top(px(h_f32 + 4.0))
                .left_0()
                .right_0()
                .bg(theme.get_color("surface.popover").unwrap_or_default())
                .border_1()
                .border_color(border)
                .rounded(r)
                .p(px(4.))
                .flex_col()
                .gap(px(2.))
                .shadow(vec![gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, 0.12),
                    offset: gpui::point(px(0.), px(4.)),
                    blur_radius: px(12.),
                    spread_radius: px(0.),
                }])
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                });

            for (i, opt) in options.iter().enumerate() {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.to_string();
                let state_for_opt = props.state.clone();
                let is_selected = value.as_ref() == Some(&opt.value);
                let item_bg = if is_selected {
                    theme
                        .get_color("winui.subtle_fill_secondary")
                        .or_else(|| theme.get_color("surface.hover"))
                        .unwrap_or_default()
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let hover_bg = theme.get_color("surface.hover").unwrap_or_default();
                let item_fg = theme
                    .get_color("content.primary")
                    .unwrap_or_default();
                let pill_color = if is_selected {
                    theme
                        .get_color("winui.accent")
                        .or_else(|| theme.get_color("action.primary.bg"))
                        .unwrap_or_default()
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let item_id = ElementId::Name(format!("default-select-opt-{}", i).into());
                let item_hover_id = item_id.clone();
                let mut item: Stateful<Div> = div()
                    .id(item_id.clone())
                    .relative()
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(4.))
                    .text_color(item_fg)
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |hovered, _win, cx| {
                        set_interaction_hovered(cx, item_hover_id.clone(), *hovered)
                    });

                let config =
                    AnimationConfig::default().with_duration(Duration::from_millis(100));
                let fill = AnimatedStateElement::new(
                    (item_id.clone(), "fill"),
                    item_id.clone(),
                    is_selected,
                    div().absolute().inset_0().rounded(px(4.)),
                    config,
                    move |d: Div, hover, _pressed, checked| {
                        // `item_bg` is non-transparent only when
                        // selected; crossfade it in via `checked`,
                        // then toward the hover fill.
                        let base = lerp_hsla(gpui::hsla(0.0, 0.0, 0.0, 0.0), item_bg, checked);
                        let next = lerp_hsla(base, hover_bg, hover);
                        d.bg(next)
                    },
                );

                item = item
                    .child(fill)
                    .child(
                        div()
                            .w(px(3.))
                            .h(px(16.))
                            .rounded(px(1.5))
                            .bg(pill_color),
                    )
                    .child(opt_label);
                item = item.on_click(move |_ev, window, cx| {
                    // Headless data action: `pick` writes
                    // value, closes the dropdown, and fires
                    // `on_change` in one call. We recover
                    // `&mut App` from the `Context` via
                    // `&mut *cx_inner` (the documented
                    // `DerefMut<Target = App>` pattern — see
                    // memory.md "Context<T> → App").
                    state_for_opt.update(cx, |s, cx_inner| {
                        s.pick(opt_value.clone(), window, &mut *cx_inner);
                    });
                });
                dropdown = dropdown.child(item);
            }

            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            // The animation wrapper is absolutely positioned at the
            // top-left of the outer relative container so the dropdown
            // inside keeps its original `top/left/right` offsets.
            outer = outer.child(
                gpui::deferred(div().absolute().top_0().left_0().right_0().child(
                    AnimatedPresenceElement::new(
                        props.state.clone(),
                        (props.id.clone(), "dropdown"),
                        SlideDirection::Down,
                        distance,
                        div().child(dropdown),
                    ),
                ))
                .with_priority(1),
            );
        }

        outer
    }
}

pub fn arc_select<T: SelectRenderer + 'static>(r: T) -> Arc<dyn SelectRenderer> {
    Arc::new(r)
}
