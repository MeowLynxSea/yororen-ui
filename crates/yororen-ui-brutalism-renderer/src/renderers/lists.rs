//! Brutalist list renderers: `ListItem`, `TreeItem`, `Tree`,
//! `Form`, `FormField`, `Table`, `VirtualList`, `UniformVirtualList`.

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled, Window,
    prelude::FluentBuilder, px,
};
use yororen_ui_core::headless::checkbox::checkbox;
use yororen_ui_core::headless::list_navigable::ListNavigable;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// ListItem
// =====================================================================

pub use yororen_ui_core::renderer::list_item::{ListItemRenderState, ListItemRenderer};

pub struct BrutalListItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalListItemRenderer {
    pub fn bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn hover_bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_bg(&self, _: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn padding(&self, _: &ListItemRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.list_item.padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(h / 2.0))
    }
    pub fn min_height(&self, _: &ListItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.list_item.min_height")
            .unwrap_or(36.0) as f32)
    }
    pub fn border_radius(&self, _: &ListItemRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl ListItemRenderer for BrutalListItemRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::list_item::ListItemProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ListItemRenderState {
            selected: props.selected,
            disabled: props.disabled,
            hovered: false,
        };
        let bg = if state.selected {
            self.selected_bg(&state, theme)
        } else {
            self.bg(&state, theme)
        };
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .px(pad.left)
            .py(pad.top)
            .min_h(h)
            .rounded(r)
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else if props.on_click.is_some() {
                CursorStyle::PointingHand
            } else {
                CursorStyle::Arrow
            })
            .child(props.title.to_string())
    }
}

// =====================================================================
// Listbox
// =====================================================================

pub use yororen_ui_core::renderer::listbox::{ListboxRenderState, ListboxRenderer};

pub struct BrutalListboxRenderer;

// Inherent helpers — *not* part of the trait surface. Brutalism
// reuses the list_item colour tokens so listbox rows look
// consistent with other list surfaces; the brutalist styling
// overrides (thick borders, hard offset shadow) come from the
// outer shell.
impl BrutalListboxRenderer {
    pub fn bg(&self, _: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn hover_bg(&self, _: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_bg(&self, _: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("border.focus")
            .or_else(|| theme.get_color("action.primary.bg"))
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, state: &ListboxRenderState, theme: &Theme) -> Hsla {
        let _ = state;
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_fg(&self, _: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.on_status")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_fg(&self, _: &ListboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
    }
    pub fn padding(&self, _: &ListboxRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.list_item.padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(h / 2.0))
    }
    pub fn min_height(&self, _: &ListboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.list_item.min_height")
            .unwrap_or(36.0) as f32)
    }
    pub fn border_radius(&self, _: &ListboxRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn gap(&self, _: &ListboxRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
    pub fn border_color(&self, _: &ListboxRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn border_width(&self, _: &ListboxRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_BORDER_WIDTH)
    }
}

impl ListboxRenderer for BrutalListboxRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::listbox::ListboxProps,
        cx: &App,
    ) -> Stateful<Div> {
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
        let border = self.border_color(&state, theme);
        let bw = self.border_width(&state, theme);

        let highlighted = read.highlighted_index;
        let selected_value = read.selected_value.clone();
        let options = read.options.clone();
        let state_for_click = props.state.clone();
        let focus_handle = read.focus_handle();
        let _ = read;

        let mut body: Div = gpui::div()
            .flex()
            .flex_col()
            .gap(gap)
            .bg(bg)
            .p(px(2.0))
            .rounded(r);

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
            let mut row: Stateful<Div> = gpui::div()
                .id(ElementId::Name(format!("brutal-listbox-row-{}", i).into()))
                .flex()
                .items_center()
                .bg(row_bg)
                .text_color(row_fg)
                .px(pad.left)
                .py(pad.top)
                .min_h(h)
                .rounded(r)
                .when(!opt.disabled, |d| d.cursor_pointer())
                .when(opt.disabled, |d| d.opacity(0.5))
                .child(opt.label.to_string());
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

        let state_for_keys = state_for_click.clone();
        gpui::div()
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
                    // See the matching comment in
                    // `TokenListboxRenderer::compose`: state-only
                    // updates need an explicit `window.refresh()`
                    // to schedule a repaint because `compose`
                    // doesn't establish a paint-time entity
                    // subscription.
                    window.refresh();
                }
            })
            .border(bw)
            .border_color(border)
            .rounded(r)
            .child(body)
    }
}

// =====================================================================
// TreeItem
// =====================================================================

pub use yororen_ui_core::renderer::tree_item::{TreeItemRenderState, TreeItemRenderer};

pub struct BrutalTreeItemRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTreeItemRenderer {
    pub fn bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn hover_bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_bg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn indent(&self, state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        let step = theme
            .get_number("tokens.control.tree_item.indent_step")
            .unwrap_or(16.0) as f32;
        px(state.depth as f32 * step)
    }
    pub fn padding(&self, _: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.tree_item.padding")
            .unwrap_or(8.0) as f32;
        Edges::symmetric(px(p), px(p / 2.0))
    }
    pub fn min_height(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree_item.min_height")
            .unwrap_or(32.0) as f32)
    }
    pub fn chevron_size(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree_item.chevron_size")
            .unwrap_or_else(|| {
                theme
                    .get_number("tokens.control.list_item.chevron_size")
                    .unwrap_or(18.0)
            }) as f32)
    }
    pub fn disabled_fg(&self, _: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border_radius(&self, _: &TreeItemRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn gap(&self, _: &TreeItemRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
}

impl TreeItemRenderer for BrutalTreeItemRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::tree_item::TreeItemProps,
        cx: &mut App,
        window: &mut Window,
    ) -> Stateful<Div> {
        use yororen_ui_core::headless::tree_item::{DOUBLE_CLICK_THRESHOLD, LastClick};
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TreeItemRenderState {
            selected: props.selected,
            expanded: props.expanded,
            depth: props.depth.min(u8::MAX as usize) as u8,
            is_leaf: !props.has_children,
            checked: props.checked,
            indeterminate: props.indeterminate,
        };
        let bg = if state.selected {
            self.selected_bg(&state, theme)
        } else {
            self.bg(&state, theme)
        };
        let hover_bg = self.hover_bg(&state, theme);
        let fg = self.fg(&state, theme);
        let pad = self.padding(&state, theme);
        let h = self.min_height(&state, theme);
        let indent = self.indent(&state, theme);
        let chevron_size = self.chevron_size(&state, theme);
        let gap = self.gap(&state, theme);
        let radius = self.border_radius(&state, theme);

        let chevron_slot = if props.has_children {
            let glyph: SharedString = if props.expanded {
                "▾".into()
            } else {
                "▸".into()
            };
            let chevron_id: ElementId = format!("{}-chevron", props.id).into();
            let toggle_cb = props.on_toggle.clone();
            let disabled = props.disabled;
            let chevron_color = if disabled {
                self.disabled_fg(&state, theme)
            } else {
                fg
            };
            gpui::div()
                .id(chevron_id)
                .w(chevron_size)
                .h(chevron_size)
                .flex()
                .items_center()
                .justify_center()
                .text_color(chevron_color)
                .when(!disabled, |s| s.cursor_pointer())
                .occlude()
                .child(glyph)
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
            gpui::div()
                .w(chevron_size)
                .h(chevron_size)
                .flex()
                .into_any_element()
        };

        let label_color = if props.disabled {
            self.disabled_fg(&state, theme)
        } else {
            fg
        };

        // Multi-select checkbox slot — composed through the
        // registered `CheckboxRenderer` so the brutal checkbox
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
                gpui::div()
                    .flex()
                    .items_center()
                    .occlude()
                    .child(checkbox_el)
                    .into_any_element(),
            )
        } else {
            None
        };

        // Double-click detector — same approach as the default
        // renderer: stamp `now` on every click; if the prior
        // stamp is within the threshold, treat as a
        // double-click and fire `on_double_click` (or
        // `on_toggle` as the fallback).
        let last_click = window.use_keyed_state(props.id.clone(), cx, |_, _| LastClick::default());
        let on_click_cb = props.on_click.clone();
        let on_toggle_cb = props.on_toggle.clone();
        let on_double_click_cb = props.on_double_click.clone();
        let disabled = props.disabled;

        let mut row = gpui::div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .w_full()
            .min_h(h)
            .bg(bg)
            .text_color(label_color)
            .pl(indent + pad.left)
            .pr(pad.right)
            .py(pad.top)
            .rounded(radius);

        if !props.selected && !props.disabled {
            row = row.hover(move |s| s.bg(hover_bg));
        }
        if !props.disabled {
            row = row.cursor_pointer();
        }
        if props.disabled {
            row = row.opacity(0.5);
        }

        if !disabled {
            row = row.on_click(move |ev, window, cx| {
                let prior = last_click.read(cx).clone();
                let is_double = prior.within(DOUBLE_CLICK_THRESHOLD);
                last_click.update(cx, |s, _cx| *s = LastClick::stamp_now());

                if is_double {
                    if let Some(cb) = on_double_click_cb.as_ref() {
                        cb(ev, window, cx);
                    } else if let Some(cb) = on_toggle_cb.as_ref() {
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

// =====================================================================
// Form
// =====================================================================

pub use yororen_ui_core::renderer::form::{FormRenderState, FormRenderer};

pub struct BrutalFormRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalFormRenderer {
    pub fn gap(&self, _: &FormRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.control.form.gap").unwrap_or(12.0) as f32)
    }
    pub fn label_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn error_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("status.error.fg").unwrap_or(BRUTAL_BORDER)
    }
    pub fn helper_color(&self, _: &FormRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
}

impl FormRenderer for BrutalFormRenderer {
    fn compose(&self, _props: &yororen_ui_core::headless::form::FormProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FormRenderState {};
        let g = self.gap(&state, theme);
        gpui::div().flex().flex_col().gap(g)
    }
}

// =====================================================================
// VirtualList
// =====================================================================

pub use yororen_ui_core::renderer::virtual_list::{VirtualListRenderState, VirtualListRenderer};

pub struct BrutalVirtualListRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalVirtualListRenderer {
    pub fn bg(&self, _: &VirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border_color(&self, _: &VirtualListRenderState, theme: &Theme) -> Hsla {
        crate::style::brutal_border_color(theme)
    }
    pub fn border_width(&self, _: &VirtualListRenderState, _: &Theme) -> Pixels {
        px(crate::style::BRUTAL_BORDER_WIDTH)
    }
    pub fn border_radius(&self, _: &VirtualListRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl VirtualListRenderer for BrutalVirtualListRenderer {
    fn compose(
        &self,
        mut props: yororen_ui_core::headless::virtual_list::VirtualListProps,
        render_row: yororen_ui_core::headless::virtual_list::RenderRowFn,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = VirtualListRenderState {
            item_count: props.item_count,
            alignment: props.alignment,
            overdraw: props.overdraw,
            sizing_behavior: props.sizing_behavior,
        };
        let bg = self.bg(&state, theme);
        let border = self.border_color(&state, theme);
        let bw = self.border_width(&state, theme);
        let radius = self.border_radius(&state, theme);

        // Wire the visible-range callback through
        // `ListState::set_scroll_handler` (see TokenVirtualListRenderer
        // for the rationale — same approach here).
        if let Some(mut cb) = props.on_visible_range_change.take() {
            props.state.set_scroll_handler(move |ev, window, cx_inner| {
                cb(ev.visible_range.clone(), ev.count, window, cx_inner);
            });
        }

        // The inner list is constructed inline and forced to
        // fill the parent — same `size_full().flex_grow().min_h_0()`
        // pattern as the default renderer. Brutalism could add
        // its own offsets (e.g. a hard offset shadow on the
        // scroll surface) here without sharing code with default.
        let list_el =
            gpui::list(props.state, render_row).with_sizing_behavior(props.sizing_behavior);
        let inner = list_el.size_full().flex_grow().min_h_0();

        // The outer div is the brutalist frame: thick black
        // border, square corners, surface background, and
        // overflow hidden so scrolled content is clipped to
        // the frame. `flex().flex_col()` makes the inner list's
        // `flex_grow()` work and lets any sibling children
        // (e.g. an info label added by the caller) take their
        // natural size at the top.
        //
        // The trailing `on_scroll_wheel` stops propagation after
        // the inner list has handled the scroll. `gpui::list`
        // scrolls correctly but does **not** call
        // `stop_propagation()`, so without this the wheel event
        // bubbles up to the page / outer scrollable container
        // and scrolls *that* in addition to the list — a
        // regression the wrapping div introduced (the
        // `VirtualList` was the styled list itself previously, no
        // outer hitbox to bubble past). `occlude()` ensures hitboxes
        // behind the list are not considered for scroll handling.
        gpui::div()
            .id(props.id)
            .flex()
            .flex_col()
            .bg(bg)
            .border_color(border)
            .border_l(bw)
            .border_r(bw)
            .border_t(bw)
            .border_b(bw)
            .rounded(radius)
            .overflow_hidden()
            .child(inner)
            .on_scroll_wheel(|_event, _window, cx| {
                cx.stop_propagation();
            })
            .occlude()
    }
}

// =====================================================================
// UniformVirtualList
// =====================================================================

pub use yororen_ui_core::renderer::uniform_virtual_list::{
    UniformVirtualListRenderState, UniformVirtualListRenderer,
};

pub struct BrutalUniformVirtualListRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalUniformVirtualListRenderer {
    pub fn bg(&self, _: &UniformVirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border_color(&self, _: &UniformVirtualListRenderState, theme: &Theme) -> Hsla {
        crate::style::brutal_border_color(theme)
    }
    pub fn border_width(&self, _: &UniformVirtualListRenderState, _: &Theme) -> Pixels {
        px(crate::style::BRUTAL_BORDER_WIDTH)
    }
    pub fn border_radius(&self, _: &UniformVirtualListRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl UniformVirtualListRenderer for BrutalUniformVirtualListRenderer {
    fn compose(
        &self,
        props: yororen_ui_core::headless::virtual_list::UniformVirtualListProps,
        render_row: yororen_ui_core::headless::virtual_list::UniformRenderRowFn,
        cx: &App,
    ) -> Stateful<Div> {
        use std::cell::RefCell;
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = UniformVirtualListRenderState {
            item_count: props.item_count,
            sizing_behavior: props.sizing_behavior,
        };
        let bg = self.bg(&state, theme);
        let border = self.border_color(&state, theme);
        let bw = self.border_width(&state, theme);
        let radius = self.border_radius(&state, theme);

        // Inner element id derived by suffixing the outer id —
        // gpui de-duplicates by id and ElementId implements
        // Display (gpui window.rs:5041).
        let inner_id: ElementId = format!("{}-inner", props.id).into();

        // Bridge FnMut → Fn (uniform_list's signature). Same
        // RefCell trick the default renderer uses; brutalism only
        // changes the styling.
        let row_cell = RefCell::new(render_row);
        let list_el = gpui::uniform_list(
            inner_id,
            props.item_count,
            move |range, window, cx_inner| {
                let mut f = row_cell.borrow_mut();
                range.map(|ix| f(ix, window, cx_inner)).collect::<Vec<_>>()
            },
        )
        .with_sizing_behavior(props.sizing_behavior)
        .track_scroll(&props.handle);

        let inner = list_el.size_full();

        gpui::div()
            .id(props.id)
            .flex()
            .flex_col()
            .bg(bg)
            .border_color(border)
            .border_l(bw)
            .border_r(bw)
            .border_t(bw)
            .border_b(bw)
            .rounded(radius)
            .overflow_hidden()
            .child(inner)
            .on_scroll_wheel(|_event, _window, cx| {
                cx.stop_propagation();
            })
            .occlude()
    }
}

// =====================================================================
// Tree
// =====================================================================

pub use yororen_ui_core::renderer::tree::{TreeRenderState, TreeRenderer};

pub struct BrutalTreeRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTreeRenderer {
    pub fn border_color(&self, _state: &TreeRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn border_width(&self, _state: &TreeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }
    pub fn padding(&self, _state: &TreeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tree.padding")
            .unwrap_or(4.0) as f32)
    }
    pub fn gap(&self, _state: &TreeRenderState, theme: &Theme) -> Pixels {
        // Brutalism defaults to 0 row gap so adjacent tree items
        // form a continuous column (their own thick borders
        // already separate them visually).
        px(theme.get_number("tokens.control.tree.gap").unwrap_or(0.0) as f32)
    }
    pub fn border_radius(&self, _state: &TreeRenderState, _theme: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn bg(&self, _state: &TreeRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
}

impl TreeRenderer for BrutalTreeRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::tree::TreeProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TreeRenderState {
            has_selection: props.selected.is_some() || !props.selected_ids.is_empty(),
            multi_select: props.selection_mode
                == yororen_ui_core::headless::tree::TreeSelectionMode::Multiple,
        };
        let border = self.border_color(&state, theme);
        let bw = self.border_width(&state, theme);
        let pad = self.padding(&state, theme);
        let gap = self.gap(&state, theme);
        let radius = self.border_radius(&state, theme);
        let bg = self.bg(&state, theme);

        gpui::div()
            .id(props.id.clone())
            .flex()
            .flex_col()
            .bg(bg)
            .gap(gap)
            .p(pad)
            .rounded(radius)
            .border(bw)
            .border_color(border)
    }
}

// =====================================================================
// FormField
// =====================================================================

pub use yororen_ui_core::renderer::form_field::{FormFieldRenderState, FormFieldRenderer};

pub struct BrutalFormFieldRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalFormFieldRenderer {
    pub fn label_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn error_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("status.error.bg")
            .or_else(|| theme.get_color("status.danger.bg"))
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn helper_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn gap(&self, _state: &FormFieldRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.form_field.gap")
            .or_else(|| theme.get_number("tokens.spacing.gap_2"))
            .unwrap_or(8.0) as f32)
    }
    pub fn font_size(&self, _state: &FormFieldRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.form_field.label_font_size")
            .or_else(|| theme.get_number("tokens.typography.font_size_xs"))
            .unwrap_or(12.0) as f32)
    }
}

impl FormFieldRenderer for BrutalFormFieldRenderer {
    fn compose(
        &self,
        props: &mut yororen_ui_core::headless::form_field::FormFieldProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FormFieldRenderState {
            has_error: props.error.is_some(),
            required: props.required,
        };
        let label_color = self.label_color(&state, theme);
        let error_color = self.error_color(&state, theme);
        let helper_color = self.helper_color(&state, theme);
        let gap = self.gap(&state, theme);
        let font_size = self.font_size(&state, theme);

        let mut wrapper = gpui::div().id(props.id.clone()).flex().flex_col().gap(gap);

        // 1) Label row — brutalism keeps the required indicator
        //    as a plain `*` (no extra styling) so it does not
        //    fight with the bold all-caps brutalist aesthetic.
        if let Some(label) = &props.label {
            let label_text: SharedString = if props.required {
                SharedString::from(format!("{} *", label))
            } else {
                label.clone()
            };
            wrapper = wrapper.child(
                gpui::div()
                    .text_size(font_size)
                    .text_color(label_color)
                    .child(label_text),
            );
        }

        // 2) Input — taken out of the props so the renderer
        //    owns it. Same pattern as `TokenFormFieldRenderer`.
        if let Some(input) = props.input.take() {
            wrapper = wrapper.child(input);
        }

        // 3) Error text (above help text — errors are more
        //    urgent in the brutalist palette and the danger
        //    background already screams).
        if let Some(error) = &props.error {
            wrapper = wrapper.child(
                gpui::div()
                    .text_size(font_size)
                    .text_color(error_color)
                    .child(error.clone()),
            );
        }

        // 4) Help text.
        if let Some(help) = &props.help {
            wrapper = wrapper.child(
                gpui::div()
                    .text_size(font_size)
                    .text_color(helper_color)
                    .child(help.clone()),
            );
        }

        wrapper
    }
}

// =====================================================================
// Table
// =====================================================================

pub use yororen_ui_core::renderer::table::{TableRenderState, TableRenderer};

pub struct BrutalTableRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTableRenderer {
    pub fn header_bg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        // The header row should be visibly heavier than body
        // rows. Brutalism uses `action.primary.bg` (typically a
        // very dark or solid colour) so the header reads like a
        // brutalist title bar.
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn header_text_color(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn row_text_color(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn row_bg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_bg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        // Selection is the brutalist "FOCUS COLOR" (a sharp
        // signal pink/red) so picking a row is unmistakable.
        theme
            .get_color("border.focus")
            .or_else(|| theme.get_color("action.primary.hover_bg"))
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn selected_fg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.on_status")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn hover_bg(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border_color(&self, _state: &TableRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn border_width(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.table.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }
    pub fn cell_padding(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.table.cell_padding")
            .or_else(|| theme.get_number("tokens.spacing.inset_sm"))
            .unwrap_or(10.0) as f32)
    }
    pub fn header_font_size(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.table.header_font_size")
            .or_else(|| theme.get_number("tokens.typography.font_size_xs"))
            .unwrap_or(12.0) as f32)
    }
    pub fn row_font_size(&self, _state: &TableRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.table.row_font_size")
            .or_else(|| theme.get_number("tokens.typography.font_size_sm"))
            .unwrap_or(13.0) as f32)
    }
    pub fn border_radius(&self, _state: &TableRenderState, _theme: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn header_cell(
        &self,
        col: &yororen_ui_core::headless::table::TableColumn,
        state: &TableRenderState,
        theme: &Theme,
    ) -> Div {
        let mut cell = gpui::div()
            .child(col.header.clone())
            .text_color(self.header_text_color(state, theme))
            .text_size(self.header_font_size(state, theme))
            .px(self.cell_padding(state, theme))
            .py(self.cell_padding(state, theme))
            .flex()
            .items_center();
        if let Some(w) = col.width_px {
            cell = cell.w(px(w));
        } else {
            cell = cell.flex_1();
        }
        cell
    }

    fn body_cell(
        &self,
        value: &SharedString,
        state: &TableRenderState,
        theme: &Theme,
        is_selected: bool,
    ) -> Div {
        let fg = if is_selected {
            self.selected_fg(state, theme)
        } else {
            self.row_text_color(state, theme)
        };
        gpui::div()
            .child(value.clone())
            .text_color(fg)
            .text_size(self.row_font_size(state, theme))
            .px(self.cell_padding(state, theme))
            .py(self.cell_padding(state, theme))
            .flex()
            .items_center()
            .flex_1()
    }
}

impl TableRenderer for BrutalTableRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::table::TableProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TableRenderState {
            selected_row: props.selected_row,
        };
        let header_bg = self.header_bg(&state, theme);
        let border = self.border_color(&state, theme);
        let bw = self.border_width(&state, theme);
        let selected_bg = self.selected_bg(&state, theme);
        let hover_bg = self.hover_bg(&state, theme);
        let row_bg = self.row_bg(&state, theme);
        let radius = self.border_radius(&state, theme);

        // Header row — the heavy brutalist title bar at the top
        // of the table. A thick bottom border separates it from
        // the body.
        let mut header = gpui::div()
            .flex()
            .flex_row()
            .bg(header_bg)
            .border_b(bw)
            .border_color(border);
        for col in &props.columns {
            header = header.child(self.header_cell(col, &state, theme));
        }

        // Body rows. Selected rows get a focus-pink background;
        // other rows hover to `surface.hover`. Rows are
        // separated by a thin bottom border in the brutalist
        // border colour for visual rhythm.
        let mut body = gpui::div().flex().flex_col();
        let row_count = props.rows.len();
        for (row_idx, row) in props.rows.iter().enumerate() {
            let is_selected = props.selected_row == Some(row_idx);
            let bg = if is_selected { selected_bg } else { row_bg };
            let mut row_el =
                gpui::div()
                    .flex()
                    .flex_row()
                    .bg(bg)
                    .cursor(if props.on_select_row.is_some() {
                        CursorStyle::PointingHand
                    } else {
                        CursorStyle::Arrow
                    });
            // Inner separator between rows (omit on the last row
            // — the table container's bottom border closes it).
            if row_idx + 1 < row_count {
                row_el = row_el.border_b(px(1.0)).border_color(border);
            }
            // Only register hover styling for non-selected
            // rows. Without an id this is harmless — gpui will
            // simply paint the hover colour when the pointer is
            // over the row's bounding box.
            if !is_selected {
                row_el = row_el.hover(move |s| s.bg(hover_bg));
            }
            for (cell_idx, cell_value) in row.iter().enumerate() {
                let mut cell = self.body_cell(cell_value, &state, theme, is_selected);
                if let Some(col) = props.columns.get(cell_idx)
                    && let Some(w) = col.width_px
                {
                    cell = cell.w(px(w));
                }
                row_el = row_el.child(cell);
            }
            body = body.child(row_el);
        }

        gpui::div()
            .id(props.id.clone())
            .flex()
            .flex_col()
            .rounded(radius)
            .border(bw)
            .border_color(border)
            .overflow_hidden()
            .child(header)
            .child(body)
    }
}
