//! Section 6 — Overlays. Each component is wrapped in a
//! labelled `cell`.

use std::collections::HashMap;

use gpui::{Context, IntoElement, ParentElement, Styled, div, px};

use yororen_ui::headless::button::button;
use yororen_ui::headless::disclosure::disclosure;
use yororen_ui::headless::dropdown_menu::dropdown_menu;
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{Spacing, column};
use yororen_ui::headless::menu::menu;
use yororen_ui::headless::overlay::overlay;
use yororen_ui::headless::popover::popover;
use yororen_ui::headless::tooltip::tooltip;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> impl IntoElement {
    // --- modal trigger only ---
    //
    // The modal scrim/panel itself is rendered by
    // `gallery_app::render` at the `scroll_root` level so that
    // (1) when the overlays row is virtualized off-screen, the
    // open modal stays visible, and (2) `.absolute().inset_0()`
    // pins to the window-spanning `scroll_root` rather than the
    // (smaller) row container.
    let modal_state_for_btn = app.modal_state.clone();
    let open_modal_btn = button("ovl-modal-open", cx)
        .on_click(move |_, _, cx| {
            modal_state_for_btn.update(cx, |st, _cx| st.open());
        })
        .render(cx)
        .child(cx.t("demo.modal.open"));
    let modal_wrapped = cell(
        cx.t("demo.modal.cell"),
        column("ovl-modal-cell-col", cx)
            .child(open_modal_btn)
            .render(cx),
        cx,
    );
    let is_modal_open = app.modal_state.read(cx).open;

    // --- popover: trigger + popover body ---
    let popover_state_for_btn = app.popover_state.clone();
    let popover_state = app.popover_state.clone();
    let popover_is_visible = popover_state.read(cx).is_visible();
    let popover_trigger = button("ovl-popover-trigger", cx)
        .on_click(move |_, _, cx| {
            popover_state_for_btn.update(cx, |st, _cx| st.toggle());
        })
        .render(cx)
        .child(if popover_is_visible {
            cx.t("demo.overlays.popover_close")
        } else {
            cx.t("demo.overlays.popover_open")
        });
    // The popover's body is a real menu (with items). The
    // menu renderer iterates `menu_state.items` to draw the
    // rows, so we only need to set up the on_select handler
    // and pass the menu element to `popover().content(...)`.
    let entity_for_menu = cx.entity().clone();
    let menu_state = app.menu_state.clone();
    let popover_state_for_close = popover_state.clone();
    menu_state.update(cx, |s, _cx| {
        s.set_on_select(move |id, _w, cx| {
            let id_s = id.to_string();
            entity_for_menu.update(cx, |s, _cx| {
                s.menu_demo_value = id_s;
            });
            // Selecting a menu item also closes the popover
            // so the menu dismisses itself after the click.
            popover_state_for_close.update(cx, |s, _cx| s.close());
        });
    });
    let menu_el = menu("ovl-menu", menu_state.clone()).render(cx);
    // Empty placeholder so the popover's content slot is
    // always populated; the renderer decides visibility by
    // `popover_state.is_open()`.
    let popover_content = if popover_is_visible {
        menu_el.into_any_element()
    } else {
        div().into_any_element()
    };
    let popover_el = popover("ovl-popover", popover_state.clone())
        .trigger(popover_trigger.into_any_element())
        .content(popover_content)
        .render(cx);
    let popover_wrapped = cell(cx.t("demo.overlays.popover_trigger"), popover_el, cx);

    // --- tooltip ---
    // The trigger stays in flow; the renderer wraps it in a
    // hitbox-bearing div and wires `on_hover` to the core
    // `TooltipState`. The demo only supplies the trigger content.
    let tooltip_state = app.tooltip_state.clone();
    let tooltip_trigger = label("ovl-tt-target", cx.t("demo.tooltip.hover_target"), cx)
        .render(cx)
        .into_any_element();
    let tooltip_el = tooltip(
        "ovl-tooltip",
        cx.t("demo.tooltip.text"),
        tooltip_state.clone(),
    )
    .trigger(tooltip_trigger)
    .render(cx);
    let tooltip_wrapped = cell(cx.t("demo.tooltip.cell"), tooltip_el, cx);

    // --- dropdown_menu ---
    // Body is a real menu that iterates `dropdown_menu_state.items`
    // (Cut / Copy / Paste / separator / Select all). The
    // dropdown renderer wraps the menu in a floating panel
    // when `dropdown_state.is_open()`. We use a separate
    // `dropdown_menu_state` (not `menu_state`) so the
    // dropdown's `on_select` writes `dropdown_demo_value`
    // and closes the dropdown, leaving the popover's
    // `menu_demo_value` untouched.
    let entity_for_dropdown = cx.entity().clone();
    let dropdown_state = app.dropdown_state.clone();
    let entity_for_dropdown_for_menu = entity_for_dropdown.clone();
    let menu_state_for_dropdown = app.dropdown_menu_state.clone();
    menu_state_for_dropdown.update(cx, |s, _cx| {
        s.set_on_select(move |id, _w, cx| {
            let id_s = id.to_string();
            entity_for_dropdown_for_menu.update(cx, |app, _cx| {
                app.dropdown_demo_value = id_s;
            });
            dropdown_state.update(cx, |s, _cx| s.close());
        });
    });
    let _ = entity_for_dropdown; // silence unused
    let dropdown_state_for_btn = app.dropdown_state.clone();
    let dropdown_trigger = button("ovl-dropdown-trigger", cx)
        .on_click(move |_, _, cx| {
            dropdown_state_for_btn.update(cx, |st, _cx| st.toggle());
        })
        .render(cx)
        .child(cx.t("demo.overlays.dropdown_open"));
    let dropdown_menu_el = menu("ovl-dropdown-body", menu_state_for_dropdown.clone()).render(cx);
    let dropdown_content = if app.dropdown_state.read(cx).is_visible() {
        dropdown_menu_el.into_any_element()
    } else {
        div().into_any_element()
    };
    let dropdown_el = dropdown_menu("ovl-dropdown", app.dropdown_state.clone())
        .trigger(dropdown_trigger.into_any_element())
        .content(dropdown_content)
        .render(cx);
    let dropdown_wrapped = cell(cx.t("demo.overlays.dropdown_trigger"), dropdown_el, cx);

    // --- disclosure ---
    // The renderer (default or WinUI) draws the trigger
    // (chevron + title). The demo only appends the expanded
    // body as a child, eliminating the previous "double trigger"
    // artifact.
    let entity_for_disc = cx.entity().clone();
    let disc_open = app.disclosure_open;
    let disc = disclosure("ovl-disc", cx.t("demo.disclosure.title"), cx)
        .open(disc_open)
        .on_toggle(move |_, _, cx| {
            entity_for_disc.update(cx, |s, _cx| {
                s.disclosure_open = !s.disclosure_open;
            });
        })
        .render(cx);
    let disc_with_body = if disc_open {
        disc.child(
            div()
                .pl(px(16.))
                .child(label("ovl-disc-body", cx.t("demo.disclosure.body"), cx).render(cx)),
        )
    } else {
        disc
    };
    let disc_wrapped = cell(cx.t("demo.disclosure.cell"), disc_with_body, cx);

    // --- overlay (scrim primitive; mirrors modal state) ---
    let overlay_el = overlay("ovl-overlay", cx)
        .open(is_modal_open)
        .render(cx)
        .child(
            label("ovl-overlay-info", cx.t("demo.overlays.scrim_info"), cx)
                .muted(true)
                .render(cx),
        );
    let overlay_wrapped = cell(cx.t("demo.overlays.scrim"), overlay_el, cx);

    let mut summary_args = HashMap::new();
    summary_args.insert("dropdown", app.dropdown_demo_value.clone());
    summary_args.insert("menu", app.menu_demo_value.clone());
    let summary_text = cx.t_with_named_args("demo.overlays.summary", &summary_args);
    column("overlays-root", cx)
        .gap(Spacing::Md)
        .child(modal_wrapped)
        .child(popover_wrapped)
        .child(tooltip_wrapped)
        .child(dropdown_wrapped)
        .child(disc_wrapped)
        .child(overlay_wrapped)
        .child(
            label("ovl-summary", summary_text, cx)
                .muted(true)
                .render(cx),
        )
        .render(cx)
}
