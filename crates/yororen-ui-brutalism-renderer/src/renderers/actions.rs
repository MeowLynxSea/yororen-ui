//! Brutalist action renderers: `Button`, `IconButton`,
//! `ToggleButton`, `SplitButton`.

use gpui::{
    App, CursorStyle, Div, ElementId, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels,
    Stateful, StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::icon_button::IconButtonProps;
use yororen_ui_core::headless::toggle_button::ToggleButtonProps;
use yororen_ui_core::renderer::spec::{BorderSpec, Edges, ShadowSpec};
use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::renderer::variant::VariantState;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::animation::AnimatedPresenceElement;

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_DISABLED_OPACITY, BRUTAL_RADIUS,
    brutal_border_color, brutal_shadow,
};

// =====================================================================
// Button
// =====================================================================

pub use yororen_ui_core::renderer::button::{ButtonRenderState, ButtonRenderer};

pub struct BrutalButtonRenderer;

// Inherent helpers — *not* part of the `ButtonRenderer` trait
// surface. They exist so `compose` (below) can stay readable
// and so other code in this crate can share the palette
// lookups.
impl BrutalButtonRenderer {
    pub fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn padding(&self, _: &ButtonRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.button.horizontal_padding")
            .unwrap_or(20.0) as f32;
        let v = theme
            .get_number("tokens.control.button.vertical_padding")
            .unwrap_or(12.0) as f32;
        Edges::symmetric(px(h), px(v))
    }

    pub fn border_radius(&self, _: &ButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn border(&self, _: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec> {
        let w = theme
            .get_number("tokens.control.button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;
        Some(BorderSpec::new(w, brutal_border_color(theme)))
    }

    pub fn shadow(&self, _: &ButtonRenderState, theme: &Theme) -> Option<ShadowSpec> {
        Some(brutal_shadow(theme))
    }

    pub fn min_height(&self, _: &ButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(44.0) as f32)
    }

    pub fn disabled_opacity(&self, state: &ButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }

    pub fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "hover_bg"
        };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "active_bg"
        };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }
}

impl ButtonRenderer for BrutalButtonRenderer {
    fn compose(&self, props: &ButtonProps, focus_handle: &FocusHandle, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ButtonRenderState {
            variant: props.variant,
            disabled: props.disabled,
            ..Default::default()
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let padding = self.padding(&state, theme);
        let radius = self.border_radius(&state, theme);
        let min_h = self.min_height(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let border = self.border(&state, theme);
        let shadow = self.shadow(&state, theme);
        let icon_gap = theme
            .get_number("tokens.control.button.icon_gap")
            .unwrap_or(8.0) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .gap(px(icon_gap))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(b) = border {
            el = el.border(b.width).border_color(b.color);
        }
        if let Some(s) = shadow {
            el = el.shadow(vec![gpui::BoxShadow {
                color: s.color,
                offset: gpui::point(px(0.0), s.offset_y),
                blur_radius: s.blur,
                spread_radius: px(0.0),
                inset: false,
            }]);
        }

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render(cx);
            el = el.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            el = el.child(caption);
        }

        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
    }
}

fn action_variant_key(variant: ActionVariantKind) -> &'static str {
    match variant {
        ActionVariantKind::Neutral => "neutral",
        ActionVariantKind::Primary => "primary",
        ActionVariantKind::Danger => "danger",
    }
}

// =====================================================================
// IconButton
// =====================================================================

pub use yororen_ui_core::renderer::icon_button::{IconButtonRenderState, IconButtonRenderer};

pub struct BrutalIconButtonRenderer;

// Inherent helpers — *not* part of the `IconButtonRenderer`
// trait surface.
impl BrutalIconButtonRenderer {
    pub fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn fg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.hover_bg", key))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn active_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.active_bg", key))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn size(&self, _: &IconButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.icon_button.size")
            .unwrap_or(44.0) as f32)
    }

    pub fn border_radius(&self, _: &IconButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn disabled_opacity(&self, state: &IconButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }
}

impl IconButtonRenderer for BrutalIconButtonRenderer {
    fn compose(
        &self,
        props: &IconButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = IconButtonRenderState {
            variant: props.variant,
            disabled: props.disabled,
            has_custom_bg: false,
            has_custom_hover_bg: false,
            custom_style: None,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let radius = self.border_radius(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let side = self.size(&state, theme);
        let border_color = brutal_border_color(theme);
        let border_w = theme
            .get_number("tokens.control.button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .rounded(radius)
            .size(side)
            .opacity(opacity)
            .border(px(border_w))
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render(cx);
            el = el.child(icon_el);
        }

        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
    }
}

// =====================================================================
// ToggleButton
// =====================================================================

pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};

pub struct BrutalToggleButtonRenderer;

// Inherent helpers — *not* part of the `ToggleButtonRenderer`
// trait surface.
impl BrutalToggleButtonRenderer {
    pub fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme
                    .get_color("action.primary.bg")
                    .unwrap_or(BRUTAL_BORDER);
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.selected {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme
                    .get_color("action.primary.fg")
                    .unwrap_or(BRUTAL_BORDER);
            }
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.fg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or(BRUTAL_BORDER);
        }
        if state.selected {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or(BRUTAL_BORDER);
        }
        if state.selected {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn min_height(&self, _: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.toggle_button.min_height")
            .unwrap_or(44.0) as f32)
    }

    pub fn border_radius(&self, _: &ToggleButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn disabled_opacity(&self, state: &ToggleButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }
}

impl ToggleButtonRenderer for BrutalToggleButtonRenderer {
    fn compose(
        &self,
        props: &ToggleButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ToggleButtonRenderState {
            variant: props.variant,
            selected: props.selected,
            disabled: props.disabled,
            custom_style: None,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let icon_gap = theme
            .get_number("tokens.control.toggle_button.icon_gap")
            .unwrap_or(8.0) as f32;
        let border_color = brutal_border_color(theme);
        let border_w = theme
            .get_number("tokens.control.toggle_button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(px(12.))
            .py(px(6.))
            .gap(px(icon_gap))
            .opacity(opacity)
            .border(px(border_w))
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render(cx);
            el = el.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            el = el.child(caption);
        }

        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
    }
}

// =====================================================================
// SplitButton
// =====================================================================

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct BrutalSplitButtonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalSplitButtonRenderer {
    pub fn primary_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn primary_fg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn chevron_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn chevron_fg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn chevron_hover_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.min_height")
            .unwrap_or(44.0) as f32)
    }
    pub fn border_radius(&self, _: &SplitButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn gap(&self, _: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.separator_w")
            .unwrap_or(3.0) as f32)
    }
}

impl SplitButtonRenderer for BrutalSplitButtonRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::split_button::SplitButtonProps,
        cx: &App,
    ) -> Div {
        use std::sync::Arc;
        use yororen_ui_core::headless::dropdown_menu::DropdownItem;
        use yororen_ui_core::headless::list_item::ListItemProps;
        use yororen_ui_core::headless::split_button::ClickCallback;
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
            toggled: props.toggled.unwrap_or(false),
        };

        // ---- Primary button ----
        // The caption is the *selected* flyout item's label when
        // set (the pick-a-list-style ToggleSplitButton pattern),
        // else the static caption. Toggle mode paints both halves
        // with the primary action variant while the toggled bit
        // is set (mirrors how the brutal `toggle_button` renders
        // selected).
        let primary_id: ElementId = format!("{:?}-primary", props.id).into();
        let primary_variant = if state.toggled {
            ActionVariantKind::Primary
        } else {
            ActionVariantKind::Neutral
        };
        let primary = ButtonProps {
            id: primary_id,
            focus_handle: props.primary_focus.clone(),
            on_click: Some(props.primary.clone()),
            disabled: props.disabled,
            clickable: true,
            variant: primary_variant,
            caption: props.display_caption(),
            icon: None,
            icon_size: px(16.),
        }
        .render(cx);

        // ---- Chevron button ----
        let state_for_chevron = props.state.clone();
        let chevron_click: ClickCallback = Arc::new(
            move |_ev: &gpui::ClickEvent, _w: &mut gpui::Window, cx: &mut App| {
                if let Some(s) = state_for_chevron.as_ref() {
                    // Notify so the open/close flip repaints even
                    // when the caller's handlers don't notify.
                    s.update(cx, |st, cx| {
                        st.toggle();
                        cx.notify();
                    });
                }
            },
        );
        let chevron_label = if open { "▴" } else { "▾" };
        let chevron_id: ElementId = format!("{:?}-chevron", props.id).into();
        let chevron_w = px(theme
            .get_number("tokens.control.split_button.chevron_width")
            .unwrap_or(36.0) as f32);
        let chevron = ButtonProps {
            id: chevron_id,
            focus_handle: props.chevron_focus.clone(),
            on_click: Some(chevron_click),
            disabled: props.disabled,
            clickable: true,
            variant: primary_variant,
            caption: Some(chevron_label.into()),
            icon: None,
            icon_size: px(16.),
        }
        .render(cx)
        .w(chevron_w)
        .px(px(0.));

        // ---- Dropdown body ----
        // `gpui::deferred(...)` so the popover paints *after*
        // every other sibling in the tree. `.absolute()` only
        // takes the menu out of layout flow; without `deferred`
        // any later sibling would draw on top of it.
        let gap = self.gap(&state, theme);
        let root = div()
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .child(primary)
            .child(chevron);
        if visible {
            let panel_bg = theme
                .get_color("surface.popover")
                .or_else(|| theme.get_color("surface.raised"))
                .unwrap_or(BRUTAL_BORDER);
            let panel_border = brutal_border_color(theme);
            let item_hover_bg = theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER);
            let divider_color = theme.get_color("border.divider").unwrap_or(BRUTAL_BORDER);
            let menu_w = px(theme
                .get_number("tokens.control.split_button.menu_width")
                .unwrap_or(200.0) as f32);
            let min_h = self.min_height(&state, theme);
            let menu_offset = min_h + px(4.);

            let shadow_spec = brutal_shadow(theme);
            let mut menu = div()
                .absolute()
                .top(menu_offset)
                .left_0()
                .w(menu_w)
                .bg(panel_bg)
                .border(px(BRUTAL_BORDER_WIDTH))
                .border_color(panel_border)
                .rounded(px(BRUTAL_RADIUS))
                .p(px(4.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .shadow(vec![gpui::BoxShadow {
                    color: shadow_spec.color,
                    offset: gpui::point(px(0.0), shadow_spec.offset_y),
                    blur_radius: shadow_spec.blur,
                    spread_radius: px(0.0),
                    inset: false,
                }])
                // popover pattern: occlude_mouse blocks
                // events from reaching elements painted behind
                // the menu, and on_mouse_down_out fires when the
                // user clicks *anywhere* outside the menu (other
                // cells, the toolbar, the title) to dismiss.
                .occlude()
                .on_mouse_down_out({
                    let state_for_close = props.state.clone();
                    move |_ev, _window, cx| {
                        if let Some(st) = state_for_close.as_ref() {
                            st.update(cx, |s, cx| {
                                s.close();
                                cx.notify();
                            });
                        }
                    }
                });

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
                        let is_selected_item = props.selected_item.as_ref() == Some(&item_id_str);
                        let list_item_el = ListItemProps {
                            id: row_id,
                            title: item_label,
                            description: None,
                            leading_icon: None,
                            trailing_icon: None,
                            selected: is_selected_item,
                            disabled: item_disabled,
                            on_click: None,
                        }
                        .render(cx);

                        // Hover on the selected item stays within the
                        // primary pair instead of washing the solid
                        // selected fill out.
                        let item_hover_target = if is_selected_item {
                            theme
                                .get_color("action.primary.hover_bg")
                                .unwrap_or(item_hover_bg)
                        } else {
                            item_hover_bg
                        };
                        let item_el = if !item_disabled {
                            list_item_el
                                .w_full()
                                .cursor_pointer()
                                .hover(move |s| s.bg(item_hover_target))
                                .on_click(move |_ev, window, cx| {
                                    if let Some(st) = state_for_click.as_ref() {
                                        st.update(cx, |s, cx| {
                                            s.close();
                                            cx.notify();
                                        });
                                    }
                                    if let Some(cb) = on_select_for_click.as_ref() {
                                        cb(item_id_for_callback.clone(), window, cx);
                                    }
                                })
                        } else {
                            list_item_el.w_full()
                        };
                        menu = menu.child(item_el);
                    }
                    DropdownItem::Separator => {
                        menu = menu.child(div().h(px(1.)).bg(divider_color).my(px(2.)));
                    }
                    DropdownItem::Group(_) => {}
                }
            }

            // The animation wrapper is absolutely positioned at the
            // top-left of the root relative container so the menu
            // inside keeps its original `top/left` offset.
            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            let state_entity = props
                .state
                .clone()
                .expect("visible implies state is present");
            root.child(
                gpui::deferred(div().absolute().top_0().left_0().child(
                    AnimatedPresenceElement::new(
                        state_entity,
                        (props.id.clone(), "menu"),
                        SlideDirection::Down,
                        distance,
                        div().child(menu),
                    ),
                ))
                .with_priority(1),
            )
        } else {
            root
        }
    }
}

// =====================================================================
// ButtonGroup
// =====================================================================

pub use yororen_ui_core::renderer::button_group::{ButtonGroupRenderState, ButtonGroupRenderer};

pub struct BrutalButtonGroupRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalButtonGroupRenderer {
    /// Gap between children in **detached** mode. Brutalism uses
    /// a wider gap than the default renderer because the thick
    /// borders make adjacent buttons visually heavy and a tighter
    /// gap looks cramped. In attached (segmented) mode the gap is
    /// always 0.
    pub fn gap(&self, _state: &ButtonGroupRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.button_group.gap")
            .unwrap_or(4.0) as f32
    }

    /// Corner radius the first / last button inherit. Brutalism
    /// always renders sharp corners, so this is `BRUTAL_RADIUS`
    /// (0).
    pub fn radius(&self, _state: &ButtonGroupRenderState, _theme: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    /// Border colour for the shared group border (only drawn
    /// in attached mode).
    pub fn border_color(&self, _state: &ButtonGroupRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
}

impl ButtonGroupRenderer for BrutalButtonGroupRenderer {
    fn compose(
        &self,
        props: yororen_ui_core::headless::button_group::ButtonGroupProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::headless::button_group::ButtonGroupOrientation;

        let theme = cx.theme();
        let state = ButtonGroupRenderState {
            orientation: props.orientation,
            attached: props.attached,
        };
        let n = props.children.len();
        let id = props.id;
        let children = props.children;

        // Container: flex row/column. The border, radius, and
        // overflow are only applied in attached mode — in
        // detached mode each child keeps its own brutalist border.
        let mut container = match props.orientation {
            ButtonGroupOrientation::Horizontal => div().flex().flex_row().items_center(),
            ButtonGroupOrientation::Vertical => div().flex().flex_col().items_center(),
        };

        if props.attached && n > 0 {
            let radius = self.radius(&state, theme);
            let border = self.border_color(&state, theme);
            // In segmented mode the group itself owns a thick
            // brutalist border around the whole row/column and
            // the children's individual borders are clipped to
            // 0 width below to avoid double-thickness seams.
            container = container
                .overflow_hidden()
                .rounded(radius)
                .border(px(BRUTAL_BORDER_WIDTH))
                .border_color(border);
        } else {
            let gap = px(self.gap(&state, theme));
            container = container.gap(gap);
        }

        // Process children: in attached mode strip the inner
        // children's border-radius so they butt up cleanly and
        // re-add the outer corners to the first / last child.
        let mut iter = children.into_iter();
        for i in 0..n {
            let Some(mut child) = iter.next() else { break };
            if props.attached && n > 1 {
                let radius = self.radius(&state, theme);
                child = child.rounded(px(0.));
                if i == 0 {
                    if props.orientation == ButtonGroupOrientation::Horizontal {
                        child = child.rounded_tl(radius).rounded_bl(radius);
                    } else {
                        child = child.rounded_tl(radius).rounded_tr(radius);
                    }
                } else if i + 1 == n {
                    if props.orientation == ButtonGroupOrientation::Horizontal {
                        child = child.rounded_tr(radius).rounded_br(radius);
                    } else {
                        child = child.rounded_bl(radius).rounded_br(radius);
                    }
                }
            }
            container = container.child(child);
        }

        container.id(id)
    }
}
