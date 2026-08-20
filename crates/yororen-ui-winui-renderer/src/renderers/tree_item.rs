//! `WinUITreeItemRenderer` — default `TreeItemRenderer` impl.
//!
//! Renders a single tree row: chevron (always reserved to keep
//! labels aligned across depths) + label.
//!
//! Visual states:
//! - **default**   — `surface.base` bg, `content.primary` fg
//! - **hover**     — `surface.hover` bg (only when not selected / disabled)
//! - **selected**  — `action.primary.bg` bg, `action.primary.fg` fg
//! - **disabled**  — `content.disabled` fg, 0.5 opacity
//!
//! Click semantics:
//! - **single click on the row body** → `props.on_click`
//! - **single click on the chevron**  → `props.on_toggle`
//! - **double click on the row body** → `props.on_double_click`
//!   if set, else falls back to `props.on_toggle`
//!
//! The double-click detector uses `window.use_keyed_state` keyed
//! by the row's `id`; the second click within
//! [`DOUBLE_CLICK_THRESHOLD`](yororen_ui_core::headless::tree_item::DOUBLE_CLICK_THRESHOLD)
//! is treated as a double-click. The chevron's click is
//! unaffected by the detector — clicks on the chevron always
//! toggle.

use std::sync::Arc;

use gpui::{
    App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use yororen_ui_core::headless::checkbox::checkbox;
use yororen_ui_core::headless::tree_item::{DOUBLE_CLICK_THRESHOLD, LastClick, TreeItemProps};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::animation::{AnimatedStateElement, lerp_hsla, set_interaction_hovered};

pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};

use crate::animation::control_config;

pub struct WinUITreeItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl WinUITreeItemRenderer {
    pub fn bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn hover_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("winui.subtle_fill_secondary")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }
    pub fn selected_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        // WinUI tree selection is a subtle highlight; text keeps its
        // primary brush.
        theme
            .get_color("winui.subtle_fill_secondary")
            .or_else(|| theme.get_color("surface.hover"))
            .unwrap_or_default()
    }
    pub fn fg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn disabled_fg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or_default()
    }
    pub fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step = theme
            .get_number("tokens.control.tree_item.indent")
            .unwrap_or_else(|| theme.get_number("tokens.spacing.inset_md").unwrap_or(0.0))
            as f32;
        let step = step.max(12.0);
        px(state.depth as f32 * step)
    }
    pub fn padding(&self, _state: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.tree_item.horizontal_padding")
                .unwrap_or_else(|| theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0))
                as f32),
            px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32),
        )
    }
    pub fn min_height(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree_item.min_height")
            .unwrap_or(28.0) as f32)
    }
    pub fn chevron_size(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree_item.chevron_size")
            .unwrap_or_else(|| {
                theme
                    .get_number("tokens.control.list_item.chevron_size")
                    .unwrap_or(18.0)
            }) as f32)
    }
    pub fn border_radius(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(0.0) as f32)
    }
    pub fn gap(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
}

impl TreeItemRenderer for WinUITreeItemRenderer {
    fn compose(&self, props: &TreeItemProps, cx: &mut App, window: &mut Window) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme().clone();
        let state = TreeItemRenderState {
            selected: props.selected,
            expanded: props.expanded,
            depth: props.depth.min(u8::MAX as usize) as u8,
            is_leaf: !props.has_children,
            checked: props.checked,
            indeterminate: props.indeterminate,
        };

        let bg = if state.selected {
            self.selected_bg(&state, &theme)
        } else {
            self.bg(&state, &theme)
        };
        let hover_bg = self.hover_bg(&state, &theme);
        let fg = self.fg(&state, &theme);
        let pad = self.padding(&state, &theme);
        let h = self.min_height(&state, &theme);
        let indent = self.indent(&state, &theme);
        let chevron_size = self.chevron_size(&state, &theme);
        let gap = self.gap(&state, &theme);
        let radius = self.border_radius(&state, &theme);

        // Chevron slot — always reserves space so labels at the
        // same depth align whether or not a row has children.
        // When `has_children` is true the slot is a clickable
        // div that fires `props.on_toggle` and uses `.occlude()`
        // so the click doesn't pass through to the row body
        // underneath (which would otherwise fire the row's
        // `on_click`).
        let chevron_slot: gpui::AnyElement = if props.has_children {
            // Fluent chevron: right-pointing when collapsed,
            // down-pointing when expanded.
            let chevron_id: ElementId = format!("{}-chevron", props.id).into();
            let toggle_cb = props.on_toggle.clone();
            let disabled = props.disabled;
            let chevron_color = if props.disabled {
                self.disabled_fg(&state, &theme)
            } else {
                fg
            };
            let chevron_icon = yororen_ui_core::headless::icon::IconProps {
                id: (chevron_id.clone(), "icon").into(),
                source: yororen_ui_core::headless::icon::IconSource::Builtin(
                    if props.expanded {
                        "arrow-down"
                    } else {
                        "arrow-right"
                    }
                    .into(),
                ),
                size: Some(chevron_size),
                color: Some(chevron_color),
            }
            .render(cx);
            div()
                .id(chevron_id)
                .w(chevron_size)
                .h(chevron_size)
                .flex()
                .items_center()
                .justify_center()
                .text_color(chevron_color)
                .when(!disabled, |s| s.cursor_pointer())
                .occlude()
                .child(chevron_icon)
                .on_click(move |ev, window, cx| {
                    if disabled {
                        return;
                    }
                    if let Some(cb) = toggle_cb.as_ref() {
                        cb(ev, window, cx);
                    }
                })
                .into_any_element()
        } else {
            div()
                .w(chevron_size)
                .h(chevron_size)
                .flex()
                .into_any_element()
        };

        let label_color = if props.disabled {
            self.disabled_fg(&state, &theme)
        } else {
            fg
        };

        // Multi-select checkbox slot — composed through the
        // registered `CheckboxRenderer` so the Fluent checkbox
        // look applies. The wrapper `.occlude()` keeps the
        // checkbox's click from also firing the row's `on_click`.
        let checkbox_slot: Option<gpui::AnyElement> = if props.checkbox {
            let check_id: ElementId = format!("{}-check", props.id).into();
            let on_check_cb = props.on_check.clone();
            let on_click_cb = props.on_click.clone();
            let checked = props.checked;
            let indeterminate = props.indeterminate;
            let disabled = props.disabled;
            let checkbox_el = checkbox(check_id, cx)
                .checked(checked)
                .indeterminate(indeterminate)
                .disabled(disabled)
                .on_toggle(move |_next, ev, window, cx| {
                    // Route the checkbox click to `on_check`,
                    // falling back to the row's `on_click` when
                    // the caller didn't provide one.
                    let cb = on_check_cb.as_ref().or(on_click_cb.as_ref());
                    if let (Some(cb), Some(ev)) = (cb, ev) {
                        cb(ev, window, cx);
                    }
                })
                .render(cx);
            Some(
                div()
                    .flex()
                    .items_center()
                    .occlude()
                    .child(checkbox_el)
                    .into_any_element(),
            )
        } else {
            None
        };

        // Double-click detector — keyed by the row's id so each
        // row tracks its own click history. We stamp `now` on
        // every click; on the next click, if the previous stamp
        // is within `DOUBLE_CLICK_THRESHOLD`, we treat this as a
        // double-click.
        let last_click = window.use_keyed_state(props.id.clone(), cx, |_, _| LastClick::default());
        let on_click_cb = props.on_click.clone();
        let on_toggle_cb = props.on_toggle.clone();
        let on_double_click_cb = props.on_double_click.clone();
        let disabled = props.disabled;

        let mut row = div()
            .id(props.id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .w_full()
            .min_h(h)
            .text_color(label_color)
            .pl(indent + pad.left)
            .pr(pad.right)
            .py(pad.top)
            .rounded(radius);

        // Hoverable rows animate their fill between `surface.base`
        // and `surface.hover`; selected / disabled rows keep their
        // (static) selected background.
        if !props.selected && !props.disabled {
            let row_id = props.id.clone();
            let hover_row_id = row_id.clone();
            let config = control_config(&theme);
            let fill = AnimatedStateElement::new(
                (row_id.clone(), "fill"),
                hover_row_id,
                false,
                div().absolute().inset_0().rounded(radius),
                config,
                move |d: Div, hover, _pressed, _checked| d.bg(lerp_hsla(bg, hover_bg, hover)),
            );
            row = row.on_hover(move |hovered, _win, cx| {
                set_interaction_hovered(cx, row_id.clone(), *hovered)
            });
            row = row.bg(gpui::hsla(0.0, 0.0, 0.0, 0.0)).child(fill);
        } else {
            row = row.bg(bg);
        }
        if !props.disabled {
            row = row.cursor_pointer();
        }
        if props.disabled {
            row = row.opacity(0.5);
        }

        // Row click — handles single-click select, double-click
        // toggle (or `on_double_click` if supplied).
        if !disabled {
            row = row.on_click(move |ev, window, cx| {
                let prior = last_click.read(cx).clone();
                let is_double = prior.within(DOUBLE_CLICK_THRESHOLD);
                // Update the timestamp regardless of outcome so
                // a third click inside the window does not
                // re-trigger the double-click.
                last_click.update(cx, |s, _cx| *s = LastClick::stamp_now());

                if is_double {
                    if let Some(cb) = on_double_click_cb.as_ref() {
                        cb(ev, window, cx);
                    } else if let Some(cb) = on_toggle_cb.as_ref() {
                        // Default: double-click toggles.
                        cb(ev, window, cx);
                    }
                } else if let Some(cb) = on_click_cb.as_ref() {
                    cb(ev, window, cx);
                }
            });
        }

        let mut row = row.child(chevron_slot);
        if let Some(slot) = checkbox_slot {
            row = row.child(slot);
        }
        row.child(props.label.clone())
    }
}

pub fn arc_tree_item<T: TreeItemRenderer + 'static>(r: T) -> Arc<dyn TreeItemRenderer> {
    Arc::new(r)
}
