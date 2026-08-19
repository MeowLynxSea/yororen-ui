//! `WinUISplitButtonRenderer` — default `SplitButtonRenderer` impl.
//!
//! Composes a complete split-button widget (primary action +
//! chevron toggle + optional floating dropdown) by delegating
//! to the `Button`, `ListItem` and `Panel` renderers. The
//! visual logic (variant tokens, item hover, popover bg/border/
//! shadow) is read from the active theme — the headless
//! `SplitButtonProps` only carries data.
//!
//! Layout: the returned `Div` is `relative`, with the trigger
//! row as its first child and (when `state.is_open()`) an
//! `absolute` dropdown body as the second child. This puts the
//! dropdown on its own stacking layer above the surrounding
//! page flow without shifting sibling components.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, BoxShadow, ClickEvent, CursorStyle, Div, ElementId, InteractiveElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window, deferred, div,
    point, px,
};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::dropdown_menu::DropdownItem;
use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::list_item::ListItemProps;
use yororen_ui_core::headless::split_button::{ClickCallback, SplitButtonProps};
use yororen_ui_core::theme::Theme;

use crate::animation::{
    AnimatedPresenceElement, AnimatedStateElement, control_config, flyout_in, flyout_out, lerp_f32,
    lerp_hsla, motion_ms, set_interaction_hovered, set_interaction_pressed,
};
use crate::renderers::button::WinUIButtonRenderer;
use crate::themes::default_font;

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct WinUISplitButtonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUISplitButtonRenderer {
    pub fn primary_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        // WinUI chevrons use the secondary text brush.
        theme
            .get_color("winui.text_secondary")
            .or_else(|| theme.get_color("content.secondary"))
            .unwrap_or_default()
    }
    pub fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(36.0) as f32)
    }
    pub fn border_radius(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn gap(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.separator_w")
            .unwrap_or(2.0) as f32)
    }
    pub fn chevron_width(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.chevron_width")
            .unwrap_or(32.0) as f32)
    }
    pub fn menu_width(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.menu_width")
            .unwrap_or(180.0) as f32)
    }
}

impl SplitButtonRenderer for WinUISplitButtonRenderer {
    fn compose(&self, props: &SplitButtonProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let (open, visible) = props
            .state
            .as_ref()
            .map(|s| {
                let s = s.read(cx);
                (s.is_open(), s.is_visible())
            })
            .unwrap_or((false, false));
        let state = SplitButtonRenderState {
            open,
            disabled: props.disabled,
        };

        // ---- One unified WinUI split button ----
        // WinUI paints the action half and the chevron half as a
        // single pill: shared background / border / radius with a
        // hairline vertical divider between them, rather than two
        // separate button blocks.
        let primary_id: ElementId = format!("{:?}-primary", props.id).into();
        let chevron_id: ElementId = format!("{:?}-chevron", props.id).into();
        let chevron_w = self.chevron_width(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        let base_bg = self.primary_bg(&state, theme);
        let fg = self.primary_fg(&state, theme);
        let hover_bg = theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(base_bg);
        let active_bg = theme
            .get_color("action.neutral.active_bg")
            .unwrap_or(hover_bg);
        // WinUI elevation border (neutral pair — the split button
        // uses DefaultButtonStyle).
        let button_state = yororen_ui_core::renderer::button::ButtonRenderState {
            variant: yororen_ui_core::renderer::variant::ActionVariantKind::Neutral,
            disabled: props.disabled,
            ..Default::default()
        };
        let border_color = WinUIButtonRenderer
            .border(&button_state, theme)
            .map(|b| b.color)
            .unwrap_or_default();
        let border_top = WinUIButtonRenderer.border_top(&button_state, theme);
        let divider_color = theme.get_color("border.divider").unwrap_or(border_color);
        let divider_h = {
            let mh: f32 = min_h.into();
            px((mh * 0.6).clamp(8.0, 20.0))
        };
        let font_size = px(theme
            .get_number("tokens.typography.font_size_md")
            .unwrap_or(14.0) as f32);

        // Whole-pill hover/press state, keyed by `props.id`, driven
        // by either half reporting in.
        let config = control_config(theme);
        let pill_fill = AnimatedStateElement::new(
            (props.id.clone(), "split-fill"),
            props.id.clone(),
            false,
            div().absolute().inset_0().rounded(radius),
            config,
            move |d: Div, hover, pressed, _checked| {
                let mut bg = lerp_hsla(base_bg, hover_bg, hover);
                if pressed > 0.0 {
                    bg = lerp_hsla(bg, active_bg, pressed);
                }
                d.bg(bg)
            },
        );

        let primary_click = props.primary.clone();
        let mut caption: Stateful<Div> = div()
            .id(primary_id)
            .flex_1()
            .min_w(px(0.))
            .h(min_h)
            .px(px(11.))
            .flex()
            .items_center()
            .font_family(default_font(theme))
            .text_size(font_size)
            .line_height(px(20.0))
            .text_color(fg)
            .track_focus(&props.primary_focus)
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover({
                let id = props.id.clone();
                move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
            });
        if !props.disabled {
            caption = caption.on_click(move |ev, window, cx| {
                let cb = primary_click.as_ref();
                cb(ev, window, cx);
            });
        }

        let state_for_chevron = props.state.clone();
        let chevron_click: ClickCallback =
            Arc::new(move |_ev: &ClickEvent, _w: &mut Window, cx: &mut App| {
                if let Some(s) = state_for_chevron.as_ref() {
                    s.update(cx, |st, _cx| st.toggle());
                }
            });

        // Chevron glyph: 12px Fluent chevron in the secondary text
        // brush. On press it dips down 1.875px (the reference's
        // `chevron-press` keyframe) and eases back on release.
        let chevron_size = px(theme
            .get_number("tokens.control.combo_box.chevron_size")
            .unwrap_or(12.0) as f32);
        let chevron_fg = self.chevron_fg(&state, theme);
        let chevron_icon = IconProps {
            id: (chevron_id.clone(), "icon").into(),
            source: yororen_ui_core::headless::icon::IconSource::Builtin("arrow-down".into()),
            size: Some(chevron_size),
            color: Some(chevron_fg),
        }
        .render(cx);
        let chevron_glyph = AnimatedStateElement::new(
            (chevron_id.clone(), "dip"),
            chevron_id.clone(),
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

        let mut chevron: Stateful<Div> = div()
            .id(chevron_id.clone())
            .w(chevron_w)
            .h(min_h)
            .flex()
            .items_center()
            .justify_center()
            .font_family(default_font(theme))
            .text_color(fg)
            .track_focus(&props.chevron_focus)
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover({
                let id = props.id.clone();
                move |hovered, _win, cx| set_interaction_hovered(cx, id.clone(), *hovered)
            })
            .child(chevron_glyph);
        if !props.disabled {
            chevron = chevron
                .on_mouse_down(MouseButton::Left, {
                    let id = chevron_id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), true)
                })
                .on_mouse_up(MouseButton::Left, {
                    let id = chevron_id.clone();
                    move |_, _win, cx| set_interaction_pressed(cx, id.clone(), false)
                })
                .on_click(move |_ev, window, cx| {
                    let cb = chevron_click.as_ref();
                    cb(_ev, window, cx);
                });
        }

        let trigger_row = div()
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .h(min_h)
            .rounded(radius)
            .border_1()
            .border_color(border_color)
            .overflow_hidden()
            .child(pill_fill)
            .child(
                // Elevation border: lighter 1px line across the top.
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(1.0))
                    .rounded_t(radius)
                    .bg(if props.disabled {
                        gpui::hsla(0., 0., 0., 0.)
                    } else {
                        border_top
                    }),
            )
            .child(caption)
            .child(div().w(px(1.)).h(divider_h).bg(divider_color).mx(px(2.)))
            .child(chevron);

        // `on_mouse_down_out` on the outer wrapper closes the menu
        // when the user clicks anywhere outside the trigger + menu
        // (the absolute menu body is still a layout child of this
        // div, so clicks on items are absorbed by the items and
        // don't fire `_out`). Clicks on the trigger are likewise
        // absorbed by the primary / chevron buttons. Honoured only
        // when the caller opted in via `dismiss_on_outside_click`.
        let state_for_close = props.state.clone();
        let dismiss_outside = props
            .state
            .as_ref()
            .map(|s| s.read(cx).dismiss_on_outside_click)
            .unwrap_or(true);

        // ---- Dropdown body (only when open) ----
        // Wrapped in `gpui::deferred(...)` so the popover paints
        // *after* every other sibling in the tree. Without this,
        // `.absolute()` only removes the element from layout flow
        // — paint order remains DOM order, so any later sibling
        // (e.g. the next row in the section) would draw on top of
        // the menu and you'd see "through" it.
        let mut root = div().relative().child(trigger_row);
        if dismiss_outside {
            root = root.on_mouse_down_out(move |_ev, _w, cx| {
                if let Some(st) = state_for_close.as_ref() {
                    st.update(cx, |s, _cx| s.close());
                }
            });
        }
        if visible {
            // Flyout surface: `winui.flyout_bg`/`flyout_stroke` on an
            // 8px OverlayCornerRadius with the reference flyout
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
            let panel_pad = px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32);
            let item_hover_bg = theme
                .get_color("winui.subtle_fill_secondary")
                .or_else(|| theme.get_color("surface.hover"))
                .unwrap_or_default();
            let shadow_color = theme
                .get_color("shadow.flyout")
                .or_else(|| theme.get_color("shadow.elevation_2"))
                .unwrap_or_default();
            let divider_color = theme.get_color("border.divider").unwrap_or_default();
            let menu_w = self.menu_width(&state, theme);
            let min_h = self.min_height(&state, theme);
            // Place the menu just below the trigger row.
            let menu_offset = min_h + px(4.);

            let mut menu = div()
                .absolute()
                .top(menu_offset)
                .left_0()
                .w(menu_w)
                .bg(panel_bg)
                .border_1()
                .border_color(panel_border)
                .rounded(panel_radius)
                .p(panel_pad)
                .flex()
                .flex_col()
                .gap(px(2.))
                .shadow(vec![BoxShadow {
                    color: shadow_color,
                    offset: point(px(0.), px(5.)),
                    blur_radius: px(15.),
                    spread_radius: px(0.),
                }])
                // popover pattern: occlude (the
                // `InteractiveElement` trait method) blocks
                // events from reaching elements painted behind
                // the menu (stops a click on an option from
                // also firing on the cell directly below the
                // split button).
                .occlude();

            for it in &props.items {
                match it {
                    DropdownItem::Item(item) => {
                        let item_id_str = item.id.clone();
                        let item_label = item.label.clone();
                        let item_disabled = item.disabled;
                        let state_for_click = props.state.clone();
                        let on_select_for_click = props.on_select.clone();
                        let item_id_for_callback = item_id_str.clone();

                        let row_id: ElementId =
                            format!("{:?}-item-{}", props.id, item_id_str).into();
                        let fill_id = row_id.clone();
                        let wrapper_id: ElementId = (row_id.clone(), "wrapper").into();
                        let list_item_el = ListItemProps {
                            id: row_id,
                            title: item_label,
                            description: None,
                            leading_icon: None,
                            trailing_icon: None,
                            selected: false,
                            disabled: item_disabled,
                            on_click: None,
                        }
                        .render(cx);

                        let item_el = if !item_disabled {
                            let wrapper_hover_id = wrapper_id.clone();
                            let wrapper: gpui::Stateful<gpui::Div> = div()
                                .id(wrapper_id.clone())
                                .relative()
                                .w_full()
                                .cursor_pointer()
                                .on_hover(move |hovered, _win, cx| {
                                    set_interaction_hovered(cx, wrapper_hover_id.clone(), *hovered)
                                })
                                .on_click(move |_ev, window, cx| {
                                    if let Some(st) = state_for_click.as_ref() {
                                        st.update(cx, |s, _cx| s.close());
                                    }
                                    if let Some(cb) = on_select_for_click.as_ref() {
                                        cb(item_id_for_callback.clone(), window, cx);
                                    }
                                });
                            let config = control_config(theme);
                            let fill = AnimatedStateElement::new(
                                (fill_id.clone(), "fill"),
                                wrapper_id.clone(),
                                false,
                                div().absolute().inset_0().rounded(px(3.)),
                                config,
                                move |d: Div, hover, _pressed, _checked| {
                                    d.bg(lerp_hsla(panel_bg, item_hover_bg, hover))
                                },
                            );
                            let inner = list_item_el.w_full().bg(gpui::hsla(0., 0., 0., 0.));
                            wrapper.child(fill).child(inner)
                        } else {
                            list_item_el.w_full().bg(panel_bg)
                        };
                        menu = menu.child(item_el);
                    }
                    DropdownItem::Separator => {
                        menu = menu.child(div().h(px(1.)).bg(divider_color).my(px(2.)));
                    }
                    DropdownItem::Group(_) => {
                        // Groups are not rendered specially in v0.3.
                    }
                }
            }

            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            let state_entity = props
                .state
                .clone()
                .expect("visible implies state is present");
            // WinUI flyout open/close: 250ms `cubic-bezier(0.1, 0.9,
            // 0.2, 1)` in, 100ms `cubic-bezier(0.7, 0, 1, 0.5)` out.
            let enter = yororen_ui_core::animation::AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    theme,
                    "duration_menu_open_slow",
                    250.0,
                )))
                .with_easing(flyout_in);
            let exit = yororen_ui_core::animation::AnimationConfig::new()
                .with_duration(Duration::from_millis(motion_ms(
                    theme,
                    "duration_menu_open_fast",
                    100.0,
                )))
                .with_easing(flyout_out);
            // The animation wrapper is absolutely positioned at the
            // top-left of the root relative container so the menu
            // inside keeps its original `top/left` offset.
            root.child(
                deferred(
                    div().absolute().top_0().left_0().child(
                        AnimatedPresenceElement::new(
                            state_entity,
                            (props.id.clone(), "menu"),
                            SlideDirection::Down,
                            distance,
                            div().child(menu),
                        )
                        .with_configs(enter, exit),
                    ),
                )
                .with_priority(1),
            )
        } else {
            root
        }
    }
}

pub fn arc_split_button<T: SplitButtonRenderer + 'static>(r: T) -> Arc<dyn SplitButtonRenderer> {
    Arc::new(r)
}
