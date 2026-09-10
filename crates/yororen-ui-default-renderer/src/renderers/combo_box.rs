//! `TokenComboBoxRenderer` — default `ComboBoxRenderer` impl.

use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::combo_box::ComboBoxProps;
use yororen_ui_core::headless::text_input_element::{
    TextInputElement, start_cursor_blink, wire_input_keyboard,
};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::AnimatedPresenceElement;

pub use yororen_ui_core::renderer::combo_box::{ComboBoxRenderState, ComboBoxRenderer};

pub struct TokenComboBoxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenComboBoxRenderer {
    pub fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            state
                .custom_bg
                .unwrap_or_else(|| theme.get_color("surface.base").unwrap_or_default())
        }
    }
    pub fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            state
                .custom_border
                .unwrap_or_else(|| theme.get_color("border.default").unwrap_or_default())
        }
    }
    pub fn focus_border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        state
            .custom_focus_border
            .unwrap_or_else(|| theme.get_color("border.focus").unwrap_or_default())
    }
    pub fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
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
    pub fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32),
            gpui::px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(0.0) as f32),
        )
    }
    pub fn border_radius(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl ComboBoxRenderer for TokenComboBoxRenderer {
    fn compose(&self, props: &ComboBoxProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

        let theme = cx.theme().clone();
        let (state, text, value, options, is_open, is_visible, placeholder) = {
            let state_read = props.state.read(cx);
            let state = ComboBoxRenderState {
                open: state_read.is_open(),
                disabled: false,
                has_value: state_read.value.is_some(),
                editable: state_read.editable,
                custom_bg: None,
                custom_border: None,
                custom_focus_border: None,
                custom_fg: None,
            };
            (
                state,
                state_read.text.clone(),
                state_read.value.clone(),
                state_read.options.clone(),
                state_read.is_open(),
                state_read.is_visible(),
                state_read.placeholder.clone(),
            )
        };
        let bg = self.bg(&state, &theme);
        let border = self.border(&state, &theme);
        let _fg = self.fg(&state, &theme);
        let pad = self.padding(&state, &theme);
        let h = self.min_height(&state, &theme);
        let r = self.border_radius(&state, &theme);

        // The combo's trigger is a real text input backed directly by
        // `ComboBoxState.core`. No separate `TextInputState` entity.
        // In selection-only mode (`editable == false`) the field is
        // display-only: no caret blink, no IBeam cursor.
        let focus_handle = props.state.read(cx).core.focus_handle();
        let focused = focus_handle.is_focused(window);
        if focused && state.editable {
            start_cursor_blink(props.state.clone(), window, cx);
        } else if state.editable {
            props
                .state
                .update(cx, |s, _cx| s.core.cursor_visible = true);
        } else {
            props
                .state
                .update(cx, |s, _cx| s.core.cursor_visible = false);
        }

        let display_str: String = if !text.is_empty() {
            text.clone()
        } else if let Some(v) = &value {
            options
                .iter()
                .find(|o| &o.value == v)
                .map(|o| o.label.to_string())
                .unwrap_or_else(|| v.to_string())
        } else {
            String::new()
        };

        let hint_color = theme.get_color("content.tertiary").unwrap_or_default();
        let text_color = theme.get_color("content.primary").unwrap_or_default();
        let cursor_color = theme.get_color("border.focus").unwrap_or_default();
        let selection_color = {
            let c = theme.get_color("border.focus").unwrap_or_default();
            gpui::hsla(c.color.hue.into_degrees() / 360.0, c.saturation, c.lightness, 0.25)
        };

        let ti_element = TextInputElement {
            state: props.state.clone(),
            focus_handle: focus_handle.clone(),
            disabled: false,
            text_color,
            hint_color,
            cursor_color,
            selection_color,
            placeholder,
            value_override: Some(display_str),
        }
        .into_any_element();

        let mut trigger: Stateful<Div> = div()
            .flex()
            .items_center()
            .bg(bg)
            .border_1()
            .border_color(border)
            .px(pad.left)
            .min_h(h)
            .rounded(r)
            .id("default-combo-trigger")
            .track_focus(&focus_handle)
            .cursor(if state.editable {
                CursorStyle::IBeam
            } else {
                CursorStyle::PointingHand
            })
            .child(div().flex_1().min_w(px(0.)).child(ti_element))
            .child(
                div()
                    .w(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(hint_color)
                    .cursor(CursorStyle::PointingHand)
                    .child(if is_open { "▴" } else { "▾" }),
            );
        let combo_state_for_open = props.state.clone();
        trigger = trigger.on_click(move |_ev, _window, cx| {
            combo_state_for_open.update(cx, |s, _cx| s.toggle());
        });

        let trigger = wire_input_keyboard(trigger, props.state.clone(), focus_handle, false, None);

        // Filtered options: case-insensitive `contains`.
        let needle = text.to_lowercase();
        let filtered: Vec<(usize, &yororen_ui_core::headless::combo_box::ComboBoxOption)> = options
            .iter()
            .enumerate()
            .filter(|(_, opt)| needle.is_empty() || opt.label.to_lowercase().contains(&needle))
            .collect();

        let mut outer = div().relative().child(trigger);

        if is_visible && !filtered.is_empty() {
            let h_f32: f32 = h.into();
            let state_for_close = props.state.clone();
            let mut dropdown: Stateful<Div> = div()
                .id("default-combo-dropdown")
                .absolute()
                .top(px(h_f32 + 4.0))
                .left_0()
                .right_0()
                .bg(theme.get_color("surface.raised").unwrap_or_default())
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
                    inset: false,
                }])
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                });

            for (orig_i, opt) in filtered.iter() {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.to_string();
                let state_for_opt = props.state.clone();
                let is_selected = value.as_ref() == Some(&opt.value);
                let item_bg = if is_selected {
                    theme.get_color("action.primary.bg").unwrap_or_default()
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let hover_bg = theme.get_color("surface.hover").unwrap_or_default();
                let item_fg = if is_selected {
                    theme.get_color("action.primary.fg").unwrap_or_default()
                } else {
                    theme.get_color("content.primary").unwrap_or_default()
                };
                let mut item: Stateful<Div> = div()
                    .id(ElementId::Name(
                        format!("default-combo-opt-{}", orig_i).into(),
                    ))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(4.))
                    .bg(item_bg)
                    .text_color(item_fg)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.bg(hover_bg))
                    .child(opt_label);
                item = item.on_click(move |_ev, window, cx| {
                    // Headless data action: `pick` writes
                    // value (which also resyncs `text` to the
                    // label), closes the dropdown, and fires
                    // `on_change` in one call. Recover
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

        outer.into_any_element()
    }
}

pub fn arc_combo_box<T: ComboBoxRenderer + 'static>(r: T) -> Arc<dyn ComboBoxRenderer> {
    Arc::new(r)
}
