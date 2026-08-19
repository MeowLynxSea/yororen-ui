//! `WinUIComboBoxRenderer` — default `ComboBoxRenderer` impl.

use std::sync::Arc;
use std::time::Duration;

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

use crate::animation::{
    AnimatedPresenceElement, AnimatedStateElement, animated_input_border, control_config,
    flyout_in, flyout_out, lerp_f32, lerp_hsla, motion_ms, set_interaction_hovered,
    set_interaction_pressed,
};
use crate::themes::{default_font, input_field_padding};

pub use yororen_ui_core::renderer::combo_box::{ComboBoxRenderState, ComboBoxRenderer};

pub struct WinUIComboBoxRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUIComboBoxRenderer {
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
            // WinUI placeholder brush is the secondary text colour.
            theme
                .get_color("winui.text_secondary")
                .or_else(|| theme.get_color("content.secondary"))
                .unwrap_or_default()
        }
    }
    pub fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.combo_box.min_height")
                .or_else(|| theme.get_number("tokens.control.button.min_height"))
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        input_field_padding(theme)
    }
    pub fn border_radius(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
}

impl ComboBoxRenderer for WinUIComboBoxRenderer {
    fn compose(&self, props: &ComboBoxProps, cx: &mut App, window: &mut Window) -> AnyElement {
        use yororen_ui_core::theme::ActiveTheme;

        let theme = cx.theme().clone();
        let (state, text, value, options, _is_open, is_visible, placeholder) = {
            let state_read = props.state.read(cx);
            let state = ComboBoxRenderState {
                open: state_read.is_open(),
                disabled: false,
                has_value: state_read.value.is_some(),
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
        let focus_handle = props.state.read(cx).core.focus_handle();
        let focused = focus_handle.is_focused(window);
        let trigger_border = if focused {
            self.focus_border(&state, &theme)
        } else {
            border
        };
        if focused {
            start_cursor_blink(props.state.clone(), window, cx);
        } else {
            props
                .state
                .update(cx, |s, _cx| s.core.cursor_visible = true);
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

        // Fluent TextBox colors for the combo trigger.
        let side_border = theme
            .get_color("winui.ctrl_stroke")
            .unwrap_or(trigger_border);
        let bottom_rest = theme
            .get_color("winui.ctrl_strong_stroke")
            .unwrap_or(trigger_border);
        let bottom_focused = theme.get_color("winui.accent").unwrap_or(trigger_border);
        let bg_hover = theme.get_color("winui.ctrl_fill_hover").unwrap_or(bg);
        let bg_focused = theme
            .get_color("winui.ctrl_fill_input_active")
            .or_else(|| theme.get_color("surface.sunken"))
            .unwrap_or(bg);

        let hint_color = theme
            .get_color("winui.text_secondary")
            .or_else(|| theme.get_color("content.secondary"))
            .unwrap_or_default();
        let text_color = theme.get_color("content.primary").unwrap_or_default();
        let cursor_color = theme.get_color("border.focus").unwrap_or_default();
        let selection_color = {
            let c = theme.get_color("border.focus").unwrap_or_default();
            gpui::hsla(c.h, c.s, c.l, 0.25)
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

        // Chevron: Fluent ChevronDown glyph (12px, secondary text
        // brush) in the reference's 38px trailing slot. On press it
        // dips down 1.875px (`chevron-press`) and eases back.
        let chevron_size = px(theme
            .get_number("tokens.control.combo_box.chevron_size")
            .unwrap_or(12.0) as f32);
        let chevron_slot_w = px(theme
            .get_number("tokens.control.combo_box.chevron_slot_w")
            .unwrap_or(38.0) as f32);
        let chevron_id: ElementId = (props.id.clone(), "chevron").into();
        let chevron_icon = yororen_ui_core::headless::icon::IconProps {
            id: (chevron_id.clone(), "icon").into(),
            source: yororen_ui_core::headless::icon::IconSource::Builtin("arrow-down".into()),
            size: Some(chevron_size),
            color: Some(hint_color),
        }
        .render(cx);
        let chevron_glyph = AnimatedStateElement::new(
            (chevron_id.clone(), "dip"),
            props.id.clone(),
            false,
            div()
                .flex()
                .items_center()
                .justify_center()
                .child(chevron_icon),
            control_config(&theme),
            move |d: Div, _hover, pressed, _checked| {
                let dip = lerp_f32(0.0, 1.875, pressed);
                d.mt(px(dip))
            },
        );
        let font_size = px(theme
            .get_number("tokens.typography.font_size_md")
            .unwrap_or(14.0) as f32);

        let mut trigger: Stateful<Div> = div()
            .flex()
            .items_center()
            .bg(bg)
            .border_1()
            .border_color(trigger_border)
            .font_family(default_font(&theme))
            .text_size(font_size)
            .line_height(px(20.0))
            .pl(pad.left)
            .pt(pad.top)
            .pb(pad.bottom)
            .min_h(h)
            .rounded(r)
            .id("default-combo-trigger")
            .track_focus(&focus_handle)
            .cursor(CursorStyle::IBeam)
            .child(div().flex_1().min_w(px(0.)).child(ti_element))
            .child(
                div()
                    .w(chevron_slot_w)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(hint_color)
                    .cursor(CursorStyle::PointingHand)
                    .child(chevron_glyph),
            );
        let combo_hover_id = props.id.clone();
        trigger = trigger
            .on_hover(move |hovered, _win, cx| {
                set_interaction_hovered(cx, combo_hover_id.clone(), *hovered);
            })
            .on_mouse_down(gpui::MouseButton::Left, {
                let id = props.id.clone();
                move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
            })
            .on_mouse_up(gpui::MouseButton::Left, {
                let id = props.id.clone();
                move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
            });
        let combo_state_for_open = props.state.clone();
        trigger = trigger.on_click(move |_ev, _window, cx| {
            combo_state_for_open.update(cx, |s, _cx| s.toggle());
        });

        let keyed = wire_input_keyboard(trigger, props.state.clone(), focus_handle, false, None);
        let trigger = animated_input_border(
            keyed,
            props.id.clone(),
            props.id.clone(),
            side_border,
            bottom_rest,
            bottom_focused,
            bg,
            bg_hover,
            bg_focused,
            focused,
        );

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
            // Flyout surface: acrylic fill, hairline stroke, 8px
            // OverlayCornerRadius, reference flyout shadow.
            let panel_bg = theme
                .get_color("winui.flyout_bg")
                .or_else(|| theme.get_color("surface.popover"))
                .unwrap_or_default();
            let panel_border = theme
                .get_color("winui.flyout_stroke")
                .or_else(|| theme.get_color("border.default"))
                .unwrap_or_default();
            let panel_radius = px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32);
            let shadow_color = theme
                .get_color("shadow.flyout")
                .or_else(|| theme.get_color("shadow.elevation_2"))
                .unwrap_or_default();
            let item_min_h = theme
                .get_number("tokens.control.dropdown.item_min_h")
                .unwrap_or(32.0) as f32;
            let item_m_x = theme
                .get_number("tokens.control.dropdown.item_margin_x")
                .unwrap_or(5.0) as f32;
            let item_m_y = theme
                .get_number("tokens.control.dropdown.item_margin_y")
                .unwrap_or(2.0) as f32;
            let item_radius = theme
                .get_number("tokens.control.dropdown.item_radius")
                .unwrap_or(3.0) as f32;
            let item_pad_t = theme
                .get_number("tokens.control.dropdown.item_padding_top")
                .unwrap_or(5.0) as f32;
            let item_pad_r = theme
                .get_number("tokens.control.dropdown.item_padding_right")
                .unwrap_or(11.0) as f32;
            let item_pad_b = theme
                .get_number("tokens.control.dropdown.item_padding_bottom")
                .unwrap_or(7.0) as f32;
            let item_pad_l = theme
                .get_number("tokens.control.dropdown.item_padding_left")
                .unwrap_or(11.0) as f32;
            let hover_fill = theme
                .get_color("winui.subtle_fill_tertiary")
                .or_else(|| theme.get_color("surface.hover"))
                .unwrap_or_default();
            let _ = &border;
            let mut dropdown: Stateful<Div> = div()
                .id("default-combo-dropdown")
                .absolute()
                .top(px(h_f32 + 4.0))
                .left_0()
                .right_0()
                .bg(panel_bg)
                .border_1()
                .border_color(panel_border)
                .rounded(panel_radius)
                .py(px(4.))
                .flex_col()
                .shadow(vec![gpui::BoxShadow {
                    color: shadow_color,
                    offset: gpui::point(px(0.), px(5.)),
                    blur_radius: px(15.),
                    spread_radius: px(0.),
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
                    theme
                        .get_color("winui.subtle_fill_secondary")
                        .or_else(|| theme.get_color("surface.hover"))
                        .unwrap_or_default()
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let hover_bg = hover_fill;
                let item_fg = theme.get_color("content.primary").unwrap_or_default();
                let pill_color = if is_selected {
                    theme
                        .get_color("winui.accent")
                        .or_else(|| theme.get_color("action.primary.bg"))
                        .unwrap_or_default()
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let item_id = ElementId::Name(format!("default-combo-opt-{}", orig_i).into());
                let item_hover_id = item_id.clone();
                let mut item: Stateful<Div> = div()
                    .id(item_id.clone())
                    .relative()
                    .min_h(px(item_min_h))
                    .mx(px(item_m_x))
                    .my(px(item_m_y))
                    .pt(px(item_pad_t))
                    .pr(px(item_pad_r))
                    .pb(px(item_pad_b))
                    .pl(px(item_pad_l))
                    .rounded(px(item_radius))
                    .text_color(item_fg)
                    .text_size(font_size)
                    .line_height(px(20.0))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |hovered, _win, cx| {
                        set_interaction_hovered(cx, item_hover_id.clone(), *hovered)
                    });

                let fill = AnimatedStateElement::new(
                    (item_id.clone(), "fill"),
                    item_id.clone(),
                    is_selected,
                    div().absolute().inset_0().rounded(px(item_radius)),
                    control_config(&theme),
                    move |d: Div, hover, _pressed, checked| {
                        let base = lerp_hsla(gpui::hsla(0.0, 0.0, 0.0, 0.0), item_bg, checked);
                        let next = lerp_hsla(base, hover_bg, hover);
                        d.bg(next)
                    },
                );

                // Selection pill: accent bar pinned 1px from the left
                // edge, 16px tall shrinking to 10px while pressed.
                let pill = div()
                    .absolute()
                    .left(px(1.))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .child(AnimatedStateElement::new(
                        (item_id.clone(), "pill"),
                        item_id.clone(),
                        is_selected,
                        div().w(px(3.)).rounded(px(1.5)).bg(pill_color),
                        control_config(&theme),
                        move |d: Div, _hover, pressed, _checked| {
                            let pill_h = lerp_f32(16.0, 10.0, pressed);
                            d.h(px(pill_h))
                        },
                    ));

                item = item.child(fill).child(pill).child(opt_label);
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
            // WinUI flyout open/close: 250ms in / 100ms out.
            let enter = yororen_ui_core::animation::AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    &theme,
                    "duration_menu_open_slow",
                    250.0,
                )))
                .with_easing(flyout_in);
            let exit = yororen_ui_core::animation::AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    &theme,
                    "duration_menu_open_fast",
                    100.0,
                )))
                .with_easing(flyout_out);
            outer = outer.child(
                gpui::deferred(
                    div().absolute().top_0().left_0().right_0().child(
                        AnimatedPresenceElement::new(
                            props.state.clone(),
                            (props.id.clone(), "dropdown"),
                            SlideDirection::Down,
                            distance,
                            div().child(dropdown),
                        )
                        .with_configs(enter, exit),
                    ),
                )
                .with_priority(1),
            );
        }

        outer.into_any_element()
    }
}

pub fn arc_combo_box<T: ComboBoxRenderer + 'static>(r: T) -> Arc<dyn ComboBoxRenderer> {
    Arc::new(r)
}
