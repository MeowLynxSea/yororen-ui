//! `WinUIUniformVirtualListRenderer` — default
//! `UniformVirtualListRenderer` impl.
//!
//! Mirrors [`WinUIVirtualListRenderer`](super::virtual_list::WinUIVirtualListRenderer)
//! but binds to `gpui::uniform_list` instead of `gpui::list`. The
//! uniform variant is significantly faster for large lists where
//! every row has the same height — gpui measures only the first
//! row and lays out the rest in a straight line, skipping the
//! taffy layout pass entirely.
//!
//! ## Closure adaptation
//!
//! `gpui::uniform_list` takes `impl Fn(Range<usize>, …) -> Vec<R>`
//! (note `Fn`, not `FnMut`), while our headless `render_row` is
//! `FnMut`. We bridge them by wrapping the row closure in
//! `RefCell<UniformRenderRowFn>` and producing the per-frame `Vec`
//! by iterating the range and calling `borrow_mut()` once per row.
//!
//! ## Two ids
//!
//! The outer styled `Div` and the inner `gpui::uniform_list`
//! element both need an `ElementId`; gpui de-duplicates by id, so
//! we derive the inner id by suffixing `-inner` to the props id.

use std::cell::RefCell;
use std::sync::Arc;

use gpui::{
    App, Div, ElementId, Hsla, InteractiveElement, ParentElement, Pixels, Stateful, Styled, div,
    px, uniform_list,
};

use yororen_ui_core::headless::virtual_list::{UniformRenderRowFn, UniformVirtualListProps};
use yororen_ui_core::renderer::uniform_virtual_list::{
    UniformVirtualListRenderState, UniformVirtualListRenderer,
};
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub struct WinUIUniformVirtualListRenderer;

impl WinUIUniformVirtualListRenderer {
    pub fn bg(&self, _state: &UniformVirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    pub fn border_color(&self, _state: &UniformVirtualListRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    pub fn border_radius(&self, _state: &UniformVirtualListRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.sm").unwrap_or(4.0) as f32)
    }
}

impl UniformVirtualListRenderer for WinUIUniformVirtualListRenderer {
    fn compose(
        &self,
        props: UniformVirtualListProps,
        render_row: UniformRenderRowFn,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = UniformVirtualListRenderState {
            item_count: props.item_count,
            sizing_behavior: props.sizing_behavior,
        };
        let bg = self.bg(&state, theme);
        let border = self.border_color(&state, theme);
        let radius = self.border_radius(&state, theme);

        // Derive the inner element's id — outer Div already owns
        // `props.id`, gpui de-duplicates by id, so we suffix.
        // `ElementId` implements `Display` (gpui's `window.rs`),
        // so the format-string round-trip is straightforward.
        let inner_id: ElementId = format!("{}-inner", props.id).into();

        // Bridge FnMut (our row closure) -> Fn (uniform_list's
        // signature). The RefCell is shared into the per-frame
        // closure; on each call we borrow_mut and produce the
        // Vec of elements for the requested range.
        let row_cell = RefCell::new(render_row);
        let list_el = uniform_list(
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

        // Same outer frame as the heterogeneous variant. The
        // explicit `on_scroll_wheel` stop_propagation is also
        // necessary here — uniform_list scrolls through its own
        // `Interactivity` scroll offset, but the outer wrapping
        // div would otherwise bubble the wheel event to the page.
        // `occlude()` ensures hitboxes behind the list are not
        // considered for scroll handling.
        div()
            .id(props.id)
            .flex()
            .flex_col()
            .bg(bg)
            .border_1()
            .border_color(border)
            .rounded(radius)
            .overflow_hidden()
            .child(inner)
            .on_scroll_wheel(|_event, _window, cx| {
                cx.stop_propagation();
            })
            .occlude()
    }
}

/// `ElementId` implements `Display` (`gpui::window.rs:5041`), so
/// callers compose derived ids via `format!("{id}-suffix")` rather
/// than the brittle `Debug` round-trip — keep that contract in
/// mind when the props id type evolves.
pub fn arc_uniform_virtual_list<T: UniformVirtualListRenderer + 'static>(
    r: T,
) -> Arc<dyn UniformVirtualListRenderer> {
    Arc::new(r)
}
