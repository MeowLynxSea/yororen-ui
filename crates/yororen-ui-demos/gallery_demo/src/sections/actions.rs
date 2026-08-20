//! Section 1 — Actions.
//!
//! Each component is wrapped in a `cell` helper (defined in
//! `sections/mod.rs`) that shows a small `name` label above the
//! component itself so the user can identify what they're
//! looking at.
//!
//! Buttons / icon_buttons / toggle_buttons all accept a
//! `caption(...)` / `icon(...)` builder method so the demo
//! doesn't have to chain `.child(...)` after `.render(...)`.
//! Icon colour is derived from the renderer's `fg` token
//! automatically — no need to pass a hardcoded colour.

use gpui::{Context, IntoElement};

use yororen_ui::ActionVariantKind;
use yororen_ui::headless::button::button;
use yororen_ui::headless::button_group::button_group;
use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
use yororen_ui::headless::icon::IconSource;
use yororen_ui::headless::icon_button::icon_button;
use yororen_ui::headless::layout::{AlignItems, Spacing, column, row, wrap};
use yororen_ui::headless::split_button::split_button;
use yororen_ui::headless::toggle_button::toggle_button;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> impl IntoElement {
    let entity = cx.entity().clone();

    // --- 3 button variants + disabled ---
    let row_buttons = wrap("actions-row-buttons", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Md)
        .child(cell(
            cx.t("demo.actions.cell_button_neutral"),
            button("btn-neutral", cx)
                .variant(ActionVariantKind::Neutral)
                .caption(cx.t("button.neutral"))
                .on_click(|_, _, _| {})
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.actions.cell_button_primary"),
            button("btn-primary", cx)
                .variant(ActionVariantKind::Primary)
                .caption(cx.t("button.primary"))
                .on_click(|_, _, _| {})
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.actions.cell_button_danger"),
            button("btn-danger", cx)
                .variant(ActionVariantKind::Danger)
                .caption(cx.t("button.danger"))
                .on_click(|_, _, _| {})
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.actions.cell_button_disabled"),
            button("btn-disabled", cx)
                .disabled(true)
                .caption(cx.t("button.disabled"))
                .on_click(|_, _, _| {})
                .render(cx),
            cx,
        ));

    // --- icon_button: variant + icon only, colour is
    //     auto-derived from the renderer's `fg` token. ---
    let row_icon_button = wrap("actions-row-icon", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Md)
        .child(cell(
            cx.t("demo.actions.cell_icon_button"),
            icon_button("ibn-check", cx)
                .on_click(|_, _, _| {})
                .icon(IconSource::Builtin("check".into()))
                .render(cx),
            cx,
        ))
        .child(cell(
            cx.t("demo.actions.cell_icon_button_primary"),
            icon_button("ibn-primary-check", cx)
                .variant(ActionVariantKind::Primary)
                .on_click(|_, _, _| {})
                .icon(IconSource::Builtin("check".into()))
                .render(cx),
            cx,
        ));

    // --- toggle_button ---
    let entity_for_tb = entity.clone();
    let row_toggle = row("actions-row-toggle", cx)
        .items_center()
        .gap(Spacing::Md)
        .child(cell(
            cx.t("demo.actions.cell_toggle_button"),
            toggle_button("tgb-1", cx)
                .selected(app.toggle_btn_selected)
                .caption(cx.t("demo.actions.press_me"))
                .on_toggle(move |_selected, _ev, _window, cx| {
                    entity_for_tb.update(cx, |s, _cx| {
                        s.toggle_btn_selected = !s.toggle_btn_selected;
                    });
                })
                .render(cx),
            cx,
        ));

    // --- split_button: primary action + chevron-toggled
    //     dropdown. The renderer handles the trigger row,
    //     chevron sizing, dropdown floating layer, hover
    //     highlight and item dismissal — the demo just
    //     supplies data (caption, items, callbacks) and a
    //     `DropdownMenuState` for the open/closed bit.
    let entity_for_primary = entity.clone();
    let entity_for_select = entity.clone();
    let split_caption = cx.t("demo.actions.save").to_string();
    let split_save_as = cx.t("demo.actions.save_as").to_string();
    let split_save_all = cx.t("demo.actions.save_all").to_string();
    let row_split = row("actions-row-split", cx)
        .items_center()
        .gap(Spacing::Md)
        .child(cell(
            cx.t("demo.actions.cell_split_button"),
            split_button(
                "spb-1",
                move |_, _, cx| {
                    entity_for_primary.update(cx, |s, cx| {
                        s.toast_count += 1;
                        // gpui does not repaint on `Entity::update`
                        // by itself — notify or the click appears
                        // to do nothing.
                        cx.notify();
                    });
                },
                cx,
            )
            .state(app.split_dropdown_state.clone())
            .caption(split_caption.clone())
            .items(vec![
                DropdownItem::Item(DropdownMenuItem::new("save", split_caption)),
                DropdownItem::Item(DropdownMenuItem::new("save_as", split_save_as)),
                DropdownItem::Item(DropdownMenuItem::new("save_all", split_save_all)),
            ])
            .on_select(move |_id, _w, cx| {
                entity_for_select.update(cx, |s, cx| {
                    s.toast_count += 1;
                    cx.notify();
                });
            })
            .render(cx),
            cx,
        ));

    // --- toggle split_button (WinUI ToggleSplitButton): the
    //     trigger is "checked" while `split_toggle_on` is set,
    //     and picking a flyout option REPLACES the primary
    //     half's caption with that option (the bullet-list
    //     pattern) while also turning the toggle on. Clicking
    //     the primary half flips the checked bit but keeps the
    //     selection. Uses its own DropdownMenuState so the two
    //     split buttons open/close independently.
    let entity_for_toggle_primary = entity.clone();
    let entity_for_toggle_select = entity.clone();
    let toggle_caption = cx.t("demo.actions.mute").to_string();
    let toggle_mute = cx.t("demo.actions.mute_mic").to_string();
    let toggle_mute_speaker = cx.t("demo.actions.mute_speaker").to_string();
    let toggle_unmute = cx.t("demo.actions.unmute").to_string();
    let row_toggle_split = row("actions-row-toggle-split", cx)
        .items_center()
        .gap(Spacing::Md)
        .child(cell(
            cx.t("demo.actions.cell_toggle_split_button"),
            split_button(
                "spb-toggle",
                move |_, _, cx| {
                    entity_for_toggle_primary.update(cx, |s, cx| {
                        s.split_toggle_on = !s.split_toggle_on;
                        cx.notify();
                    });
                },
                cx,
            )
            .state(app.split_toggle_dropdown_state.clone())
            .toggled(app.split_toggle_on)
            .selected_item(app.split_toggle_selected.clone())
            .caption(toggle_caption)
            .items(vec![
                DropdownItem::Item(DropdownMenuItem::new("mute", toggle_mute)),
                DropdownItem::Item(DropdownMenuItem::new("mute_speaker", toggle_mute_speaker)),
                DropdownItem::Item(DropdownMenuItem::new("unmute", toggle_unmute)),
            ])
            .on_select(move |id, _w, cx| {
                entity_for_toggle_select.update(cx, |s, cx| {
                    s.split_toggle_selected = Some(id);
                    s.split_toggle_on = true;
                    cx.notify();
                });
            })
            .render(cx),
            cx,
        ));

    // --- button_group: set props, add children, end with
    //     .render(cx) — the same shape as every other
    //     component. Each child button is independently styled
    //     by `ButtonRenderer` (bg / fg / rounded / hover /
    //     active); the group renderer only owns the container's
    //     layout (flex direction + gap).
    let left_caption = cx.t("demo.actions.left").to_string();
    let mid_caption = cx.t("demo.actions.mid").to_string();
    let right_caption = cx.t("demo.actions.right").to_string();
    let row_group = row("actions-row-group", cx)
        .items_center()
        .gap(Spacing::Md)
        .child(cell(
            cx.t("demo.actions.cell_button_group"),
            button_group("btg-1", cx)
                .child(
                    button("btn-left", cx)
                        .variant(ActionVariantKind::Neutral)
                        .caption(left_caption)
                        .on_click(|_, _, _| {})
                        .render(cx),
                )
                .child(
                    button("btn-mid", cx)
                        .variant(ActionVariantKind::Neutral)
                        .caption(mid_caption)
                        .on_click(|_, _, _| {})
                        .render(cx),
                )
                .child(
                    button("btn-right", cx)
                        .variant(ActionVariantKind::Neutral)
                        .caption(right_caption)
                        .on_click(|_, _, _| {})
                        .render(cx),
                )
                .render(cx),
            cx,
        ));

    column("actions-root", cx)
        .gap(Spacing::Md)
        .child(row_buttons.render(cx))
        .child(row_icon_button.render(cx))
        .child(row_toggle.render(cx))
        .child(row_split.render(cx))
        .child(row_toggle_split.render(cx))
        .child(row_group.render(cx))
        .render(cx)
}
