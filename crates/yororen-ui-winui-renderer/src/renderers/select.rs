//! `WinUISelectRenderer` — default `SelectRenderer` impl.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::animation::{AnimationConfig, SlideDirection};
use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::select::SelectProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedPresenceElement, AnimatedStateElement, animated_input_border, control_config,
    flyout_in, flyout_out, lerp_f32, lerp_hsla, motion_ms, set_interaction_hovered,
    set_interaction_pressed,
};
use crate::themes::{default_font, input_field_padding};

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
            // WinUI placeholder brush is the secondary text colour.
            theme
                .get_color("winui.text_secondary")
                .or_else(|| theme.get_color("content.secondary"))
                .unwrap_or_default()
        }
    }
    pub fn hint_color(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.text_secondary")
            .or_else(|| theme.get_color("content.secondary"))
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.select.min_height")
                .or_else(|| theme.get_number("tokens.control.button.min_height"))
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        input_field_padding(theme)
    }
    pub fn border_radius(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn chevron_size(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.select.chevron_size")
                .unwrap_or(12.0) as f32,
        )
    }
    pub fn chevron_slot_w(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.select.chevron_slot_w")
                .unwrap_or(38.0) as f32,
        )
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
        let chevron_size = self.chevron_size(&state, theme);
        let chevron_slot_w = self.chevron_slot_w(&state, theme);
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
        let bottom_rest = theme
            .get_color("winui.ctrl_strong_stroke")
            .unwrap_or(border);
        let bottom_focused = theme.get_color("winui.accent").unwrap_or(border);
        let bg_hover = theme.get_color("winui.ctrl_fill_hover").unwrap_or(bg);
        let bg_focused = theme
            .get_color("winui.ctrl_fill_input_active")
            .or_else(|| theme.get_color("surface.sunken"))
            .unwrap_or(bg);
        let is_open = state_read.is_visible();
        let chevron_fg = self.hint_color(&state, theme);
        let font_size = px(theme
            .get_number("tokens.typography.font_size_md")
            .unwrap_or(14.0) as f32);

        // Chevron: Fluent ChevronDown glyph in the secondary text
        // brush, in a 38px trailing slot. On press it dips down
        // 1.875px (the reference `chevron-press` keyframe) and eases
        // back on release.
        let chevron_id: ElementId = (props.id.clone(), "chevron").into();
        let chevron_icon = IconProps {
            id: (chevron_id.clone(), "icon").into(),
            source: yororen_ui_core::headless::icon::IconSource::Builtin("arrow-down".into()),
            size: Some(chevron_size),
            color: Some(chevron_fg),
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
            control_config(theme),
            move |d: Div, _hover, pressed, _checked| {
                let dip = lerp_f32(0.0, 1.875, pressed);
                d.mt(px(dip))
            },
        );

        let state_for_toggle = props.state.clone();
        let mut trigger: Stateful<Div> = div()
            .flex()
            .items_center()
            .bg(bg)
            .border_1()
            .border_color(border)
            .font_family(default_font(theme))
            .text_size(font_size)
            .line_height(px(20.0))
            .text_color(fg)
            .pl(pad.left)
            .pr(px(0.))
            .pt(pad.top)
            .pb(pad.bottom)
            .min_h(h)
            .rounded(r)
            .cursor(CursorStyle::PointingHand)
            // WinUI ComboBox grid: `minmax(0, 1fr) 38px` — the
            // label flexes, the chevron owns a fixed trailing slot.
            .child(div().flex_1().min_w(px(0.)).child(display))
            .child(
                div()
                    .w(chevron_slot_w)
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(chevron_glyph),
            )
            .id("default-select-trigger");
        trigger = trigger
            .on_hover({
                let id = props.id.clone();
                move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
            })
            .on_mouse_down(gpui::MouseButton::Left, {
                let id = props.id.clone();
                move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
            })
            .on_mouse_up(gpui::MouseButton::Left, {
                let id = props.id.clone();
                move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
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

            // Flyout surface: acrylic-tinted fill, hairline stroke,
            // 8px OverlayCornerRadius and the reference flyout
            // shadow (0 5px 15px ~20% black).
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

            // Item geometry per the reference ComboBox flyout:
            // 32px min-height rows, 2px 5px margins, 3px radius and
            // `5px 11px 7px` content padding.
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

            let mut dropdown: Stateful<Div> = div()
                .id("default-select-dropdown")
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
                    inset: false,
                }])
                .occlude()
                .on_mouse_down_out(move |_ev, _window, cx| {
                    state_for_close.update(cx, |s, _cx| s.close());
                });

            let hover_bg = theme
                .get_color("winui.subtle_fill_tertiary")
                .or_else(|| theme.get_color("surface.hover"))
                .unwrap_or_default();
            let selected_bg = theme
                .get_color("winui.subtle_fill_secondary")
                .or_else(|| theme.get_color("surface.hover"))
                .unwrap_or_default();

            for (i, opt) in options.iter().enumerate() {
                let opt_value = opt.value.clone();
                let opt_label = opt.label.to_string();
                let state_for_opt = props.state.clone();
                let is_selected = value.as_ref() == Some(&opt.value);
                let item_bg = if is_selected {
                    selected_bg
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                };
                let item_fg = theme.get_color("content.primary").unwrap_or_default();
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
                    control_config(theme),
                    move |d: Div, hover, _pressed, checked| {
                        // `item_bg` is non-transparent only when
                        // selected; crossfade it in via `checked`,
                        // then toward the hover fill.
                        let base = lerp_hsla(gpui::hsla(0.0, 0.0, 0.0, 0.0), item_bg, checked);
                        let next = lerp_hsla(base, hover_bg, hover);
                        d.bg(next)
                    },
                );

                // Selection pill: 3px-wide accent bar pinned 1px from
                // the left edge (matching the reference's absolutely
                // positioned pill), shrinking from 16px to 10px
                // while pressed (167ms `cubic-bezier(0,0,0,1)`).
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
                        control_config(theme),
                        move |d: Div, _hover, pressed, _checked| {
                            let pill_h = lerp_f32(16.0, 10.0, pressed);
                            d.h(px(pill_h))
                        },
                    ));

                item = item.child(fill).child(pill).child(opt_label);
                item = item.on_click(move |_ev, window, cx| {
                    // Headless data action: `pick` writes
                    // value, closes the dropdown, and fires
                    // `on_change` in one call. We recover
                    // `&mut App` from the `Context` via
                    // `&mut *cx_inner` (the documented
                    // `Context<T> → App` pattern).
                    state_for_opt.update(cx, |s, cx_inner| {
                        s.pick(opt_value.clone(), window, &mut *cx_inner);
                    });
                });
                dropdown = dropdown.child(item);
            }

            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            // WinUI flyout open/close: 250ms `cubic-bezier(0.1, 0.9,
            // 0.2, 1)` in, 100ms `cubic-bezier(0.7, 0, 1, 0.5)` out.
            let enter = AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    theme,
                    "duration_menu_open_slow",
                    250.0,
                )))
                .with_easing(flyout_in);
            let exit = AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    theme,
                    "duration_menu_open_fast",
                    100.0,
                )))
                .with_easing(flyout_out);
            // The animation wrapper is absolutely positioned at the
            // top-left of the outer relative container so the dropdown
            // inside keeps its original `top/left` offsets.
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

        outer
    }
}

pub fn arc_select<T: SelectRenderer + 'static>(r: T) -> Arc<dyn SelectRenderer> {
    Arc::new(r)
}
