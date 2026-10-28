//! The `GalleryApp` view. The per-section UI lives in XML
//! files under `src/ui/sections/`; the top-level section
//! virtual list itself is declared in `src/ui/sections.xml`
//! as a `<VirtualList>` whose row body dispatches to those
//! files via `controller.section_element`. This file owns
//! only the scroll-root, the modal overlay, and the
//! notification host.

use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, hsla, px,
};
use yororen_ui::theme::ActiveTheme;

use yororen_ui::headless::modal::modal;
use yororen_ui::notification::center::NotificationCenter;
use yororen_ui::xml_file;
use yororen_ui::{t, t_named};

use crate::controller::Controller;
use crate::state::StateRef;
use crate::theme_switcher::{DarkMode, RendererKind, install_renderer};

pub struct GalleryApp {
    controller: Controller,
    // Cached "last installed" toolbar choice so we only call
    // `install_renderer` (which HashMap-inserts 39 renderers
    // + sets the global theme) when the toolbar toggle
    // actually changed. The defaults match `RendererKind::default()`
    // and `DarkMode::default()` so the first frame's install
    // is a no-op for the common case.
    last_renderer: Option<RendererKind>,
    last_dark_mode: Option<DarkMode>,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>, controller: Controller) -> Self {
        // Re-render whenever the underlying state entity changes.
        // Note: GalleryState is a single entity that holds every
        // demo field, so any mutation triggers a full view re-render.
        // This is fine for the demo scale; a production app with
        // many independent fields should split state into multiple
        // entities and observe each one individually.
        let state = cx.global::<StateRef>().state.clone();
        cx.observe(&state, |_this, _state, cx| cx.notify()).detach();
        Self {
            controller,
            last_renderer: None,
            last_dark_mode: None,
        }
    }
}

impl Render for GalleryApp {
    #[allow(clippy::explicit_auto_deref)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 1. Snapshot the toolbar fields once and install the
        //    renderer only when the choice actually changes.
        //    Re-installing every frame is cheap (39 inserts +
        //    1 set_global) but unnecessary — the renderer is
        //    fully owned by the registry global.
        let current_renderer = self.controller.current_renderer(cx);
        let current_dark = self.controller.dark_mode(cx);
        if self.last_renderer != Some(current_renderer) || self.last_dark_mode != Some(current_dark)
        {
            install_renderer(&mut **cx, current_renderer, current_dark);
            self.last_renderer = Some(current_renderer);
            self.last_dark_mode = Some(current_dark);
        }

        // 2. Register the host window with the notification center
        //    so toasts can auto-dismiss.
        if let Some(center) = cx.try_global::<NotificationCenter>() {
            center.register_host_window(window.window_handle());
        }

        // 3. Surface color for the window background. The
        //    `surface.base` key is part of every bundled theme,
        //    so the fallback is only a safety net (e.g. for a
        //    custom theme JSON that omits it).
        let surface = cx
            .theme()
            .get_color("surface.base")
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.98, 1.0));

        // 4. Modal scrim/panel at scroll-root level so it survives
        //    virtualization.
        let is_modal_open = self.controller.is_modal_open(cx);
        let modal_overlay = build_modal_overlay(self, is_modal_open, window, cx);

        // 5. Section virtual list — declared in `ui/sections.xml`.
        //    The `<VirtualList>` tag mints and persists its own
        //    `VirtualListController` (keyed by element id), so this
        //    view no longer owns a controller field or a per-row
        //    dispatch match. `controller` is bound into scope for
        //    the XML's row body (`controller.section_element`).
        let controller = self.controller.clone();
        let sections = xml_file!(cx = cx, window = window, "ui/sections.xml");

        div()
            .id("gallery-scroll")
            .relative()
            .size_full()
            .bg(surface)
            .child(sections)
            .child(modal_overlay)
            .child(crate::notifications_host::deferred_host(cx))
    }
}

fn build_modal_overlay(
    app: &GalleryApp,
    is_modal_open: bool,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> gpui::Deferred {
    if !is_modal_open {
        return gpui::deferred(div()).priority(2);
    }

    // `controller` is bound for the modal body XML, which
    // references it by name (`controller.t(...)`,
    // `controller.close_modal`).
    let controller = app.controller.clone();
    let modal_panel = modal("ov-modal", app.controller.modal_state(cx))
        .child(xml_file!(
            cx = cx,
            window = window,
            "ui/sections/modal_body.xml"
        ))
        .render(cx)
        .w(px(360.));

    gpui::deferred(
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.55))
            .child(modal_panel),
    )
    .priority(2)
}
