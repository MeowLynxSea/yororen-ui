//! `GalleryApp` — the demo's root view.
//!
//! The render flow:
//! 1. **Per-render renderer install** — `install_renderer` is called
//!    at the top of every `Render::render`. The renderers' `register_arc`
//!    is `last-wins` and `install_with` is idempotent, so a single click
//!    on the toolbar's renderer toggle causes the next paint to read
//!    the new renderers' tokens. See `theme_switcher.rs`.
//! 2. **Host window registration** — `center.register_host_window`
//!    is called every render so the notification auto-dismiss
//!    timer has a window to refresh. Without this the timer is
//!    never scheduled and non-sticky toasts would never disappear.
//! 3. **Locale install on change** — when the toolbar picks a new
//!    locale, we call the corresponding `yororen_ui::locale_xx::install`
//!    to overwrite the global `I18n`.
//! 4. **Virtualized section list** — the toolbar / 7 sections /
//!    footer are wired into a single top-level
//!    [`yororen_ui::headless::virtual_list::virtual_list`] so only
//!    rows whose layout overlaps the viewport (+ overdraw) are
//!    actually built. The row closure delegates to the existing
//!    `sections::xxx(app, window, ctx)` functions via
//!    `entity.update(cx, |app, ctx| ...)` so each section sees
//!    a proper `&mut Context<GalleryApp>` and `&mut GalleryApp`
//!    even though the closure itself only receives `&mut App`.
//! 5. **Modal scrim at scroll-root level** — the modal panel +
//!    dimmed scrim are constructed here (not inside
//!    `sections::overlays`) and added as a sibling of the
//!    virtual list. `.absolute().inset_0()` pins to the
//!    `.relative()` `scroll_root`, so the scrim always covers
//!    the whole window — even when the overlays row is
//!    virtualized off-screen.
//! 6. **Global notification host** —
//!    [`crate::notifications_host::deferred_host`] is the LAST child
//!    of the root, wrapped in `gpui::deferred` at priority 3 so it
//!    paints above the modal scrim and every other overlay. The
//!    toolbar's "Show toast" / "Show notification" buttons push into
//!    the global `NotificationCenter`; the host reads that queue and
//!    renders each item as a floating card in the top-right corner.

use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, Render, Stateful,
    Styled, Window, div, hsla, px,
};

use yororen_ui::ActionVariantKind;
use yororen_ui::headless::button::button;
use yororen_ui::headless::divider::divider;
use yororen_ui::headless::heading::HeadingLevel;
use yororen_ui::headless::heading::heading;
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{AlignItems, Inset, Spacing, center, column, wrap};
use yororen_ui::headless::modal::modal;
use yororen_ui::headless::toggle_button::toggle_button;
use yororen_ui::headless::virtual_list::virtual_list;
use yororen_ui::i18n::Translate;
use yororen_ui::notification::center::{Notification, NotificationCenter, ToastKind};
use yororen_ui::theme::ActiveTheme;

use crate::sections;
use crate::state::{GalleryApp, LocaleChoice};
use crate::theme_switcher::{DarkMode, RendererKind, install_renderer};
/// Number of rows in the top-level section virtual list.
///
/// Row mapping:
/// - 0: toolbar + divider
/// - 1: `sections::actions`
/// - 2: `sections::display`
/// - 3: `sections::surfaces`
/// - 4: `sections::inputs`
/// - 5: `sections::controls`
/// - 6: `sections::overlays`
/// - 7: `sections::lists`
/// - 8: footer (live counters)
///
/// Used by `state.rs` to size the `VirtualListController`.
pub const SECTION_ROW_COUNT: usize = 9;

impl Render for GalleryApp {
    // The `&mut **cx` recovers a `&mut gpui::App` from the
    // `&mut Context<Self>` (the v0.3 `DerefMut<Target = App>`
    // pattern — see `memory.md`). Clippy sees it as redundant
    // auto-deref but the conversion is intentional.
    #[allow(clippy::explicit_auto_deref)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 1. Renderer + theme install. The first frame installs
        //    the defaults; subsequent frames only reinstall
        //    when the toolbar toggle actually changes.
        let current_renderer = self.current_renderer;
        let current_dark = self.dark_mode;
        if self.install_cache.renderer != Some(current_renderer)
            || self.install_cache.dark_mode != Some(current_dark)
        {
            install_renderer(&mut **cx, current_renderer, current_dark);
            self.install_cache.renderer = Some(current_renderer);
            self.install_cache.dark_mode = Some(current_dark);
        }

        // 2. Register the host window with the notification
        //    center. `maybe_schedule_auto_dismiss` returns
        //    early if `host_window` is `None`, so the global
        //    timer that auto-removes non-sticky toasts won't
        //    fire unless we bind the current window. Cheap
        //    (one `Mutex` lock + `Some` assignment).
        if let Some(center) = cx.try_global::<NotificationCenter>() {
            center.register_host_window(window.window_handle());
        }

        // 3. Surface color for the window background. Falls
        //    back to off-white when the active theme omits
        //    `surface.base` (the fallback is only a safety net
        //    for ad-hoc theme JSON).
        let surface = cx
            .theme()
            .get_color("surface.base")
            .unwrap_or_else(|| hsla(0.0, 0.0, 0.98, 1.0));

        // 4. Build the modal scrim/panel here (at the scroll-root
        //    level) so it always covers the whole window. If the
        //    overlays row is virtualized off-screen, an open modal
        //    would otherwise vanish.
        let modal_overlay = build_modal_overlay(self, cx);

        // 5. Build the virtual list that holds toolbar + 7
        //    sections + footer. The row closure dispatches by
        //    index to `sections::xxx(app, window, ctx)` via
        //    `entity.update(...)` so each section sees a proper
        //    `Context<GalleryApp>` even though the closure
        //    receives `&mut App` from `gpui::list`'s paint path.
        let entity = cx.entity().clone();
        let section_vl = virtual_list("gallery-sections-vl", &self.section_list_controller, cx)
            .row(move |ix, window, cx| section_row(ix, &entity, window, cx))
            .render(cx)
            .size_full()
            // The default `VirtualListRenderer` paints a 1-px
            // border + small radius on its outer wrapper. We want
            // the section list to read as the page itself, not as
            // a framed panel — flatten the border by reusing the
            // surface color and squash the radius.
            .border_color(surface)
            .rounded(px(0.));

        // 6. Root: relative for the modal scrim, full-window
        //    surface bg, then virtual list + scrim +
        //    notifications. `notifications_host::deferred_host`
        //    is the last child so its priority-3 deferred paint
        //    lands on top of the modal scrim (priority 2) and
        //    every other overlay.
        div()
            .id("gallery-scroll")
            .relative()
            .size_full()
            .bg(surface)
            .child(section_vl)
            .child(modal_overlay)
            .child(crate::notifications_host::deferred_host(cx))
    }
}

/// Build one virtualized row of the section list.
///
/// Called by `gpui::list` (via the `VirtualListRenderer`) only for
/// rows whose layout overlaps the viewport + overdraw, so
/// off-screen sections are never constructed. The closure receives
/// `&mut App`; we `entity.update(...)` to recover the
/// `&mut GalleryApp` + `&mut Context<GalleryApp>` pair that the
/// existing `sections::xxx` functions expect.
///
/// Padding is reproduced per-row to match the original
/// `scroll_root.p(px(24.))` + `.gap(px(24.))` visual: every row
/// has 24-px horizontal padding and 24-px bottom padding (which
/// acts as the inter-row gap), and row 0 also has 24-px top
/// padding so the toolbar isn't flush against the window edge.
fn section_row(
    ix: usize,
    entity: &gpui::Entity<GalleryApp>,
    window: &mut Window,
    cx: &mut gpui::App,
) -> AnyElement {
    let entity = entity.clone();
    entity.update(cx, |app, ctx| {
        let inner: AnyElement = match ix {
            0 => {
                // Toolbar + divider live together as row 0 so the
                // divider doesn't pick up the 24-px row gap on
                // both sides.
                let tb = build_toolbar(app, ctx);
                let col = column("toolbar-col", ctx)
                    .child(tb)
                    .child(divider("toolbar-divider", ctx).apply(div()).my(px(8.)))
                    .render(ctx);
                col.into_any_element()
            }
            1 => sections::actions(app, window, ctx).into_any_element(),
            2 => sections::display(app, window, ctx).into_any_element(),
            3 => sections::surfaces(app, window, ctx).into_any_element(),
            4 => sections::inputs(app, window, ctx).into_any_element(),
            5 => sections::controls(app, window, ctx).into_any_element(),
            6 => sections::overlays(app, window, ctx).into_any_element(),
            7 => sections::lists(app, window, ctx).into_any_element(),
            8 => footer_section(app, ctx).into_any_element(),
            _ => div().into_any_element(),
        };

        let mut wrapper = div().px(px(24.)).pb(px(24.));
        if ix == 0 {
            wrapper = wrapper.pt(px(24.));
        }
        wrapper.child(inner).into_any_element()
    })
}

/// Build the modal scrim + centered panel. Returns the
/// deferred-paint wrapper to be added as a child of the
/// scroll-root, regardless of whether the modal is open
/// (when closed, an empty deferred placeholder keeps the
/// element-tree shape stable across frames).
fn build_modal_overlay(app: &GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::Deferred {
    let is_modal_visible = app.modal_state.read(cx).is_visible();
    if !is_modal_visible {
        return gpui::deferred(div()).with_priority(2);
    }

    let modal_state_for_close = app.modal_state.clone();
    let modal_panel = modal("ov-modal", app.modal_state.clone())
        .child(
            label("ov-modal-title", cx.t("demo.modal.title"), cx)
                .strong(true)
                .render(cx),
        )
        .child(label("ov-modal-body", cx.t("demo.modal.body"), cx).render(cx))
        .child(
            button("ov-modal-close", cx)
                .on_click(move |_, _, cx| {
                    modal_state_for_close.update(cx, |st, _cx| st.close());
                })
                .render(cx)
                .child(cx.t("common.close")),
        )
        .render(cx)
        .w(px(360.));

    gpui::deferred(
        center("modal-scrim", cx)
            .child(modal_panel)
            .render(cx)
            .absolute()
            .inset_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.55)),
    )
    .with_priority(2)
}

/// Toolbar at the top of the window.
///
/// Layout (horizontal, gap 12px):
/// ```
/// [title] | [Default | Brutalism | WinUI]  [Light | Dark]  [EN | 中文 | العربية]  [Show toast]
/// ```
fn build_toolbar(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Stateful<Div> {
    let entity = cx.entity().clone();
    let mut toolbar = wrap("toolbar", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Md)
        .child(
            heading("title", HeadingLevel::H1, cx.t("demo.title"), cx)
                .apply(div())
                .mr(px(8.)),
        );

    // RendererKind toggle: 3 toggle_buttons, mutually exclusive via
    // state.current_renderer.
    let entity_for_renderer = entity.clone();
    toolbar = toolbar.child(
        toggle_button("renderer-default", cx)
            .selected(app.current_renderer == RendererKind::Default)
            .variant(ActionVariantKind::Primary)
            .on_toggle(move |_selected, _ev, _window, cx| {
                entity_for_renderer.update(cx, |s, _cx| {
                    s.current_renderer = RendererKind::Default;
                });
            })
            .render(cx)
            .child(cx.t("demo.renderer_default")),
    );
    let entity_for_renderer = entity.clone();
    toolbar = toolbar.child(
        toggle_button("renderer-brutalism", cx)
            .selected(app.current_renderer == RendererKind::Brutalism)
            .variant(ActionVariantKind::Primary)
            .on_toggle(move |_selected, _ev, _window, cx| {
                entity_for_renderer.update(cx, |s, _cx| {
                    s.current_renderer = RendererKind::Brutalism;
                });
            })
            .render(cx)
            .child(cx.t("demo.renderer_brutalism")),
    );
    let entity_for_renderer = entity.clone();
    toolbar = toolbar.child(
        toggle_button("renderer-winui", cx)
            .selected(app.current_renderer == RendererKind::WinUI)
            .variant(ActionVariantKind::Primary)
            .on_toggle(move |_selected, _ev, _window, cx| {
                entity_for_renderer.update(cx, |s, _cx| {
                    s.current_renderer = RendererKind::WinUI;
                });
            })
            .render(cx)
            .child(cx.t("demo.renderer_winui")),
    );

    // Dark mode toggle (2 toggle_buttons).
    let entity_for_dark = entity.clone();
    toolbar = toolbar.child(
        toggle_button("dark-light", cx)
            .selected(app.dark_mode == DarkMode::Light)
            .on_toggle(move |_selected, _ev, _window, cx| {
                entity_for_dark.update(cx, |s, _cx| {
                    s.dark_mode = DarkMode::Light;
                });
            })
            .render(cx)
            .child(cx.t("demo.theme_light")),
    );
    let entity_for_dark = entity.clone();
    toolbar = toolbar.child(
        toggle_button("dark-dark", cx)
            .selected(app.dark_mode == DarkMode::Dark)
            .on_toggle(move |_selected, _ev, _window, cx| {
                entity_for_dark.update(cx, |s, _cx| {
                    s.dark_mode = DarkMode::Dark;
                });
            })
            .render(cx)
            .child(cx.t("demo.theme_dark")),
    );

    // Locale: 3 toggle_buttons. Toggling calls the gallery's
    // `i18n::install_for_locale` to call
    // `yororen_ui::locale::install_with_translations`, which installs
    // the chosen locale's component defaults + this demo's own
    // translations (see `crate::i18n`).
    //
    // The button labels are the language's *own* name in its
    // native script — not localizable text. They are language
    // identifiers, not English descriptions of a language.
    for (id, choice, label) in [
        ("locale-en", LocaleChoice::En, "EN"),
        ("locale-zh", LocaleChoice::ZhCn, "中文"),
        ("locale-ar", LocaleChoice::Ar, "العربية"),
    ] {
        let entity_for_locale = entity.clone();
        let label = label.to_string();
        let selected = app.current_locale == choice;
        let tb = toggle_button(id, cx)
            .selected(selected)
            .on_toggle(move |_selected, _ev, _window, cx| {
                entity_for_locale.update(cx, |s, _cx| {
                    s.current_locale = choice;
                });
                crate::i18n::install_for_locale(cx, choice);
            })
            .render(cx)
            .child(label);
        toolbar = toolbar.child(tb);
    }

    // Show toast button.
    let entity_for_toast = entity.clone();
    let toast_title = cx.t("demo.toast_title").to_string();
    toolbar = toolbar.child(
        button("show-toast", cx)
            .variant(ActionVariantKind::Danger)
            .on_click(move |_, _, cx| {
                let id = entity_for_toast.update(cx, |s, _cx| {
                    s.toast_count += 1;
                    s.toast_count
                });
                // Defer the global access into a separate
                // statement so the immutable borrow on `cx`
                // ends before `notify` takes a `&mut`.
                let center = cx.global::<NotificationCenter>().clone();
                let id_str = id.to_string();
                let msg = cx.t_with_args("demo.toast_message", &[&id_str]);
                center.notify(
                    Notification::new(msg)
                        .title(toast_title.clone())
                        .kind(ToastKind::Info),
                    cx,
                );
            })
            .render(cx)
            .child(cx.t("demo.show_toast")),
    );

    // Show notification (sticky) button.
    let entity_for_notify = entity.clone();
    let notification_title = cx.t("demo.notification_title").to_string();
    toolbar = toolbar.child(
        button("show-notification", cx)
            .on_click(move |_, _, cx| {
                let id = entity_for_notify.update(cx, |s, _cx| s.toast_count + 1);
                let center = cx.global::<NotificationCenter>().clone();
                let id_str = id.to_string();
                let msg = cx.t_with_args("demo.notification_message", &[&id_str]);
                center.notify(
                    Notification::new(msg)
                        .title(notification_title.clone())
                        .kind(ToastKind::Success)
                        .sticky(true),
                    cx,
                );
            })
            .render(cx)
            .child(cx.t("demo.show_notification")),
    );

    toolbar.render(cx)
}

/// Footer at the bottom: shows live counters so the user can
/// verify state changes are wired correctly.
fn footer_section(app: &GalleryApp, cx: &mut Context<GalleryApp>) -> Stateful<Div> {
    let form_submit_label = cx.t("demo.footer.form_submit_count").to_string();
    let email_label = cx.t("demo.footer.email").to_string();
    let error_label = cx.t("demo.footer.error").to_string();
    let checkbox_label = cx.t("demo.footer.checkbox").to_string();
    let switch_label = cx.t("demo.footer.switch").to_string();
    let radio_label = cx.t("demo.footer.radio").to_string();
    let slider_label = cx.t("demo.footer.slider").to_string();
    let toast_label = cx.t("demo.footer.toast_count").to_string();
    let locale_label = cx.t("demo.footer.locale").to_string();

    column("footer", cx)
        .gap(Spacing::Xs)
        .p(Inset::Md)
        .child(
            label(
                "footer-title",
                cx.t("demo.footer.live_counters"),
                cx,
            )
            .strong(true)
            .render(cx),
        )
        .child(label(
            "footer-form",
            format!(
                "{form_submit_label} {}  |  {email_label} {:?}  |  {error_label} {:?}",
                app.form_submit_count, app.form_email_value, app.form_email_error
            ),
            cx,
        )
        .render(cx))
        .child(label(
            "footer-controls",
            format!(
                "{checkbox_label} {}  |  {switch_label} {}  |  {radio_label} {}  |  {slider_label} {:.1}",
                app.checkbox_value, app.switch_value, app.radio_value, app.slider_value
            ),
            cx,
        )
        .render(cx))
        .child(label(
            "footer-toast",
            format!(
                "{toast_label} {}  |  {locale_label} {}",
                app.toast_count,
                app.current_locale.tag()
            ),
            cx,
        )
        .render(cx))
        .render(cx)
        .mt(px(16.))
        .rounded(px(6.))
        .border_1()
        .border_color(hsla(0.0, 0.0, 0.5, 0.3))
}
