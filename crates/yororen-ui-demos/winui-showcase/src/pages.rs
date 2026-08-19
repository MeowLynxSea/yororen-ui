//! Page builders for the WinUI showcase app.
//!
//! Layout follows the real WinUI Gallery: a dark app background,
//! big 12px-corner translucent cards, a hero header on Home, and
//! comfortable WinUI spacing (32px page padding, 24px gaps).

use gpui::{
    Context, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window, div, px,
};

use yororen_ui::ActionVariantKind;
use yororen_ui::headless::button::button;
use yororen_ui::headless::card::card as card_props;
use yororen_ui::headless::checkbox::checkbox;
use yororen_ui::headless::combo_box::combo_box;
use yororen_ui::headless::dropdown_menu::dropdown_menu;
use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
use yororen_ui::headless::file_path_input::file_path_input;
use yororen_ui::headless::heading::{HeadingLevel, heading};
use yororen_ui::headless::icon::IconSource;
use yororen_ui::headless::icon::icon;
use yororen_ui::headless::icon_button::icon_button;
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{AlignItems, Spacing, column, row, wrap};
use yororen_ui::headless::listbox::listbox;
use yororen_ui::headless::menu::menu;
use yororen_ui::headless::number_input::number_input;
use yororen_ui::headless::password_input::password_input;
use yororen_ui::headless::radio::radio;
use yororen_ui::headless::radio_group::radio_group;
use yororen_ui::headless::search_input::search_input;
use yororen_ui::headless::select::select;
use yororen_ui::headless::slider::slider;
use yororen_ui::headless::split_button::split_button;
use yororen_ui::headless::switch::switch;
use yororen_ui::headless::text_area::text_area;
use yororen_ui::headless::text_input::text_input;
use yororen_ui::headless::toggle_button::toggle_button;
use yororen_ui::theme::ActiveTheme;

use crate::app::{WinuiApp, WinuiPage};

/// Dispatch to the active page.
pub fn build(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    match app.page {
        WinuiPage::Home => home_page(app, cx),
        WinuiPage::Buttons => buttons_page(app, cx),
        WinuiPage::Inputs => inputs_page(app, window, cx),
        WinuiPage::Toggles => toggles_page(app, cx),
        WinuiPage::Lists => lists_page(app, window, cx),
        WinuiPage::Status => status_page(app, cx),
        WinuiPage::Dialogs => dialogs_page(app, window, cx),
        WinuiPage::Text => text_page(app, window, cx),
        WinuiPage::Data => data_page(app, window, cx),
        WinuiPage::Surfaces => surfaces_page(app, window, cx),
    }
}

// ---------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------

/// Page header: big title + subtitle + a hairline, like a WinUI
/// Gallery sample page.
fn page_header(page: WinuiPage, cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
    let stroke = cx
        .theme()
        .get_color("winui.card_stroke")
        .unwrap_or_else(|| cx.theme().get_color("border.default").unwrap_or_default());
    column("winui-page-header", cx)
        .gap(Spacing::Sm)
        .child(heading("winui-page-title", HeadingLevel::H1, page.title(), cx).render(cx))
        .child(
            label("winui-page-subtitle", page.subtitle(), cx)
                .muted(true)
                .render(cx),
        )
        .child(div().my(px(16.0)).h(px(1.0)).w_full().bg(stroke))
        .render(cx)
        .into_any_element()
}

/// A WinUI card: all visual decisions (translucent fill, hairline
/// stroke, 12px corners, shadow, padding) belong to the WinUI
/// `CardRenderer`; this helper only supplies content and width.
fn card(
    id: impl std::fmt::Display,
    child: impl IntoElement,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    let el = card_props(format!("winui-card-{id}"), cx).render(cx);
    el.w_full().child(child).into_any_element()
}

fn status_line(
    id: impl std::fmt::Display,
    text: String,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    label(format!("winui-status-{id}"), text, cx)
        .muted(true)
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Home — hero + clickable feature cards
// ---------------------------------------------------------------------

fn home_page(app: &mut WinuiApp, cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
    let hero = hero_header(cx);

    let grid = wrap("winui-home-cards", cx)
        .items(AlignItems::Stretch)
        .gap(Spacing::Lg)
        .child(feature_card(
            "home-card-buttons",
            "Buttons & Actions",
            "Standard, primary, danger, icon, toggle and split buttons.",
            WinuiPage::Buttons,
            cx,
        ))
        .child(feature_card(
            "home-card-inputs",
            "Inputs",
            "Text, password, number, search and multiline inputs.",
            WinuiPage::Inputs,
            cx,
        ))
        .child(feature_card(
            "home-card-toggles",
            "Toggles & Sliders",
            "Checkbox, switch, radio and slider — all animated.",
            WinuiPage::Toggles,
            cx,
        ))
        .child(feature_card(
            "home-card-lists",
            "Lists & Menus",
            "Select, combo box, listbox and dropdown menus.",
            WinuiPage::Lists,
            cx,
        ))
        .child(feature_card(
            "home-card-status",
            "Status & Feedback",
            "Progress, skeleton, badges, tags, tooltip and avatars.",
            WinuiPage::Status,
            cx,
        ))
        .child(feature_card(
            "home-card-dialogs",
            "Dialogs & Flyouts",
            "Modal dialog, popover and disclosure surfaces.",
            WinuiPage::Dialogs,
            cx,
        ))
        .child(feature_card(
            "home-card-text",
            "Text & Typography",
            "Headings, labels, dividers and keyboard shortcuts.",
            WinuiPage::Text,
            cx,
        ))
        .child(feature_card(
            "home-card-data",
            "Data & Tables",
            "Tables, trees and virtualized lists.",
            WinuiPage::Data,
            cx,
        ))
        .child(feature_card(
            "home-card-surfaces",
            "Surfaces & Layout",
            "Cards, panels, images, segmented groups and forms.",
            WinuiPage::Surfaces,
            cx,
        ))
        .render(cx);

    let _ = app;
    column("winui-home", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(hero)
        .child(grid)
        .render(cx)
        .into_any_element()
}

/// WinUI Gallery style hero: version chip + big title + subtitle in
/// a wide WinUI card. The card's visuals are owned by the renderer.
fn hero_header(cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
    let accent = {
        let t = cx.theme();
        t.get_color("winui.accent")
            .unwrap_or_else(|| t.get_color("action.primary.bg").unwrap_or_default())
    };
    let content = column("winui-hero", cx)
        .gap(Spacing::Sm)
        .child(label("winui-hero-version", "1.0.0-Insider", cx).render(cx))
        .child(
            heading(
                "winui-hero-title",
                HeadingLevel::H1,
                "WinUI on Web Gallery",
                cx,
            )
            .render(cx),
        )
        .child(
            label(
                "winui-hero-subtitle",
                "Bring WinUI experience to the web.",
                cx,
            )
            .muted(true)
            .render(cx),
        )
        .child(div().w(px(48.0)).h(px(3.0)).rounded(px(2.0)).bg(accent))
        .render(cx);

    card_props("winui-hero", cx)
        .render(cx)
        .child(content)
        .into_any_element()
}

fn feature_card(
    id: impl std::fmt::Display,
    title: &str,
    description: &str,
    page: WinuiPage,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    let entity = cx.entity().clone();
    let accent = {
        let t = cx.theme();
        t.get_color("winui.accent")
            .unwrap_or_else(|| t.get_color("action.primary.bg").unwrap_or_default())
    };
    let content = div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .w_full()
                .child(
                    heading(format!("winui-feature-{id}-t"), HeadingLevel::H3, title, cx)
                        .render(cx),
                )
                .child(
                    icon(
                        format!("winui-feature-{id}-arrow"),
                        IconSource::Builtin("arrow-right".into()),
                        cx,
                    )
                    .color(accent)
                    .size(px(14.0))
                    .render(cx),
                ),
        )
        .child(
            label(format!("winui-feature-{id}-d"), description, cx)
                .muted(true)
                .render(cx),
        );

    card_props(format!("winui-feature-{id}"), cx)
        .interactive(true)
        .render(cx)
        .w(px(250.0))
        .h(px(180.0))
        .child(content)
        .on_click(move |_ev, _w, cx| {
            entity.update(cx, |s, _cx| s.page = page);
        })
        .into_any_element()
}

// ---------------------------------------------------------------------
// Buttons & Actions
// ---------------------------------------------------------------------

fn buttons_page(app: &mut WinuiApp, cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
    let entity = cx.entity().clone();
    let entity_primary = entity.clone();
    let header = page_header(WinuiPage::Buttons, cx);

    let standard_row = wrap("winui-buttons-row", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Md)
        .child(
            button("wb-neutral", cx)
                .variant(ActionVariantKind::Neutral)
                .caption("Neutral")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            button("wb-primary", cx)
                .variant(ActionVariantKind::Primary)
                .caption("Primary")
                .on_click(move |_, _, cx| {
                    entity_primary.update(cx, |s, _cx| s.primary_clicks += 1);
                })
                .render(cx),
        )
        .child(
            button("wb-danger", cx)
                .variant(ActionVariantKind::Danger)
                .caption("Danger")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            button("wb-disabled", cx)
                .disabled(true)
                .caption("Disabled")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .render(cx);

    let icon_row = wrap("winui-buttons-icon-row", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Md)
        .child(
            icon_button("wb-ic-search", cx)
                .icon(IconSource::Builtin("search".into()))
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            icon_button("wb-ic-user", cx)
                .variant(ActionVariantKind::Primary)
                .icon(IconSource::Builtin("user".into()))
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .render(cx);

    let entity_tb = entity.clone();
    let toggle = toggle_button("wb-toggle", cx)
        .selected(app.toggle_sel)
        .caption("Toggle me")
        .on_toggle(move |_selected, _ev, _w, cx| {
            entity_tb.update(cx, |s, _cx| s.toggle_sel = !s.toggle_sel);
        })
        .render(cx);

    let entity_primary2 = entity.clone();
    let entity_select = entity.clone();
    let split_caption = "Save".to_string();
    let split = split_button(
        "wb-split",
        move |_, _, cx| {
            entity_primary2.update(cx, |s, _cx| s.primary_clicks += 1);
        },
        cx,
    )
    .state(app.split_dd_state.clone())
    .caption(split_caption)
    .items(vec![
        DropdownItem::Item(DropdownMenuItem::new("save", "Save")),
        DropdownItem::Item(DropdownMenuItem::new("save_as", "Save as…")),
        DropdownItem::Item(DropdownMenuItem::new("save_all", "Save all")),
    ])
    .on_select(move |_id, _w, cx| {
        entity_select.update(cx, |s, _cx| s.split_action += 1);
    })
    .render(cx);

    column("winui-buttons", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "buttons",
            column("winui-buttons-inner", cx)
                .gap(Spacing::Md)
                .child(standard_row)
                .child(icon_row)
                .child(toggle)
                .child(split)
                .child(status_line(
                    "split",
                    format!(
                        "Primary clicked {}× · split menu used {}×",
                        app.primary_clicks, app.split_action
                    ),
                    cx,
                ))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------

fn inputs_page(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Inputs, cx);

    let entity_text = entity.clone();
    let text_el = text_input("wi-text")
        .placeholder("Type here…")
        .on_change(move |new: &str, _w, cx| {
            entity_text.update(cx, |s, _cx| s.text = new.to_string());
        })
        .render(cx, window);

    let entity_pw = entity.clone();
    let pw_el = password_input("wi-password")
        .placeholder("Password")
        .mask_char('•')
        .on_change(move |new: &str, _w, cx| {
            entity_pw.update(cx, |s, _cx| s.password = new.to_string());
        })
        .render(cx, window);

    let entity_num = entity.clone();
    let entity_num_inc = entity.clone();
    let entity_num_dec = entity.clone();
    let num_el = number_input("wi-number")
        .min(0.0)
        .max(100.0)
        .step(1.0)
        .value(app.number)
        .on_change(move |new: f64, _w, cx| {
            entity_num.update(cx, |s, _cx| s.number = new);
        })
        .on_increment(move |next: f64, _w, cx| {
            entity_num_inc.update(cx, |s, _cx| s.number = next);
        })
        .on_decrement(move |next: f64, _w, cx| {
            entity_num_dec.update(cx, |s, _cx| s.number = next);
        })
        .render(cx, window);

    let entity_search = entity.clone();
    let search_el = search_input("wi-search")
        .placeholder("Search…")
        .on_change(move |new: &str, _w, cx| {
            entity_search.update(cx, |s, _cx| s.search = new.to_string());
        })
        .on_clear(|_w, _cx| {})
        .render(cx, window);

    let entity_fp = entity.clone();
    let fp_el = file_path_input("wi-file-path")
        .placeholder("Select a file…")
        .on_change(move |new: &str, _w, cx| {
            entity_fp.update(cx, |s, _cx| s.file_path = new.to_string());
        })
        .on_browse(|_p, _w, _cx| {})
        .render(cx, window);

    let entity_ta = entity.clone();
    let ta_el = text_area("wi-text-area")
        .placeholder("Multiline…")
        .on_change(move |new: &str, _w, cx| {
            entity_ta.update(cx, |s, _cx| s.text_area = new.to_string());
        })
        .render(cx, window);

    column("winui-inputs", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "inputs",
            column("winui-inputs-inner", cx)
                .gap(Spacing::Md)
                .child(labeled("wi-lb-text", "Text", text_el, cx))
                .child(labeled("wi-lb-pw", "Password", pw_el, cx))
                .child(labeled("wi-lb-num", "Number", num_el, cx))
                .child(labeled("wi-lb-search", "Search", search_el, cx))
                .child(labeled("wi-lb-fp", "File path", fp_el, cx))
                .child(labeled("wi-lb-ta", "Text area", ta_el, cx))
                .child(status_line(
                    "inputs",
                    format!(
                        "text={:?} · password={:?} · number={} · search={:?} · path={:?}",
                        app.text, app.password, app.number, app.search, app.file_path
                    ),
                    cx,
                ))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

fn labeled(
    id: impl std::fmt::Display,
    text: &str,
    child: impl IntoElement,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    column(format!("winui-labeled-{id}"), cx)
        // WinUI TextBox / ComboBox default width is 296px; use
        // cross-axis stretch so the input actually fills this width.
        .w(px(296.0))
        .gap(Spacing::Xs)
        .items(AlignItems::Stretch)
        .child(label(format!("winui-labeled-{id}-lbl"), text, cx).render(cx))
        .child(child)
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Toggles & Sliders
// ---------------------------------------------------------------------

fn toggles_page(app: &mut WinuiApp, cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Toggles, cx);

    let entity_cb = entity.clone();
    let cb = checkbox("wi-checkbox", cx)
        .checked(app.checkbox)
        .on_toggle(move |v, _ev, _w, cx| {
            entity_cb.update(cx, |s, _cx| s.checkbox = v);
        })
        .render(cx);

    let entity_sw = entity.clone();
    let sw = switch("wi-switch", cx)
        .checked(app.switch_on)
        .on_toggle(move |v, _ev, _w, cx| {
            entity_sw.update(cx, |s, _cx| s.switch_on = v);
        })
        .render(cx);

    let rg = radio_group("wi-radio", cx)
        .name("winui-radio-group")
        .selected(app.radio)
        .apply(div().flex().flex_row().gap(px(12.0)).items_center());
    let rg = (0..3).fold(rg, |acc, i| {
        let entity_r = entity.clone();
        acc.child(
            radio(format!("wi-radio-{i}"), cx)
                .checked(app.radio == i)
                .on_toggle(move |_v, _ev, _w, cx| {
                    entity_r.update(cx, |s, _cx| s.radio = i);
                })
                .render(cx),
        )
    });

    let entity_sl = entity.clone();
    let sl = slider("wi-slider", cx)
        .value(app.slider)
        .range(0.0, 100.0)
        .step(1.0)
        .on_change(move |v, _w, cx| {
            entity_sl.update(cx, |s, _cx| s.slider = v);
        })
        .render(cx);

    column("winui-toggles", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "toggles",
            column("winui-toggles-inner", cx)
                .gap(Spacing::Lg)
                .child(
                    row("winui-toggle-row", cx)
                        .items_center()
                        .gap(Spacing::Lg)
                        .child(cb)
                        .child(sw)
                        .render(cx),
                )
                .child(rg)
                .child(
                    column("winui-slider-col", cx)
                        .gap(Spacing::Sm)
                        .child(sl)
                        .child(status_line(
                            "toggles",
                            format!(
                                "checkbox={} · switch={} · radio={} · slider={:.1}",
                                app.checkbox, app.switch_on, app.radio, app.slider
                            ),
                            cx,
                        ))
                        .render(cx),
                )
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Lists & Menus
// ---------------------------------------------------------------------

fn lists_page(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Lists, cx);

    // select
    let entity_select = entity.clone();
    let select_state = app.select_state.clone();
    select_state.update(cx, |s, _cx| {
        s.set_on_change(move |value, _w, cx| {
            let v = value.to_string();
            entity_select.update(cx, |s, _cx| s.select_value = v);
        });
    });
    let select_el = select("wi-select", select_state.clone()).render(cx);

    // combo box
    let entity_combo = entity.clone();
    let combo_state = app.combo_state.clone();
    combo_state.update(cx, |s, _cx| {
        s.set_on_change(move |value, _w, cx| {
            let v = value.to_string();
            entity_combo.update(cx, |s, _cx| s.combo_value = v);
        });
    });
    let combo_el = combo_box("wi-combo", combo_state.clone()).render(cx, &mut *window);

    // listbox
    let entity_lb = entity.clone();
    let listbox_state = app.listbox_state.clone();
    listbox_state.update(cx, |s, _cx| {
        s.set_on_change(move |value, _w, cx| {
            let v = value.to_string();
            entity_lb.update(cx, |s, _cx| s.listbox_value = v);
        });
    });
    let lb_el = listbox("wi-listbox", listbox_state.clone()).render(cx);

    // dropdown menu trigger
    let entity_dd = entity.clone();
    let menu_state = app.menu_state.clone();
    let dropdown_state = app.dropdown_state.clone();
    menu_state.update(cx, |s, _cx| {
        s.set_on_select(move |id, _w, cx| {
            let v = id.to_string();
            entity_dd.update(cx, |s, _cx| s.dropdown_value = v);
            dropdown_state.update(cx, |s, _cx| s.close());
        });
    });
    let dd_toggle = button("wi-dd-trigger", cx)
        .on_click({
            let st = app.dropdown_state.clone();
            move |_, _, cx| {
                st.update(cx, |s, _cx| s.toggle());
            }
        })
        .render(cx)
        .child("Open menu");
    let dd_menu = menu("wi-dd-menu", menu_state.clone()).render(cx);
    let dd_is_visible = app.dropdown_state.read(cx).is_visible();
    let dd_content = if dd_is_visible {
        dd_menu.into_any_element()
    } else {
        div().into_any_element()
    };
    let dd_el = dropdown_menu("wi-dropdown", app.dropdown_state.clone())
        .trigger(dd_toggle.into_any_element())
        .content(dd_content)
        .render(cx);

    column("winui-lists", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "lists",
            column("winui-lists-inner", cx)
                .gap(Spacing::Lg)
                .child(labeled("wi-lb-select", "Select", select_el, cx))
                .child(labeled("wi-lb-combo", "Combo box", combo_el, cx))
                .child(labeled("wi-lb-listbox", "List box", lb_el, cx))
                .child(labeled("wi-lb-dd", "Menu", dd_el, cx))
                .child(status_line(
                    "lists",
                    format!(
                        "select={:?} · combo={:?} · listbox={:?} · menu={:?}",
                        app.select_value, app.combo_value, app.listbox_value, app.dropdown_value
                    ),
                    cx,
                ))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Status & Feedback
// ---------------------------------------------------------------------

/// Section heading inside a card (H3).
fn section(
    id: impl std::fmt::Display,
    title: &str,
    child: impl IntoElement,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    column(format!("winui-section-{id}"), cx)
        .gap(Spacing::Md)
        .child(heading(format!("winui-section-{id}-t"), HeadingLevel::H3, title, cx).render(cx))
        .child(child)
        .render(cx)
        .into_any_element()
}

fn status_page(app: &mut WinuiApp, cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
    use yororen_ui::headless::avatar::avatar;
    use yororen_ui::headless::badge::{BadgeVariant, badge};
    use yororen_ui::headless::empty_state::empty_state;
    use yororen_ui::headless::progress::progress;
    use yororen_ui::headless::skeleton::skeleton;
    use yororen_ui::headless::tag::tag;
    use yororen_ui::headless::tooltip::tooltip;

    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Status, cx);

    // Determinate progress driven by a slider + an indeterminate bar.
    let entity_prog = entity.clone();
    let prog_slider = slider("ws-prog-slider", cx)
        .value(app.progress)
        .range(0.0, 1.0)
        .step(0.01)
        .on_change(move |v, _w, cx| {
            entity_prog.update(cx, |s, _cx| s.progress = v);
        })
        .render(cx);
    let det = progress("ws-progress", cx)
        .value(app.progress)
        .max(1.0)
        .render(cx);
    let indet = progress("ws-indeterminate", cx)
        .indeterminate(true)
        .render(cx);
    let progress_section = column("ws-prog-col", cx)
        .gap(Spacing::Md)
        .child(
            column("ws-det-col", cx)
                .gap(Spacing::Sm)
                .child(
                    label(
                        "ws-det-lbl",
                        format!("Determinate — {:.0}%", app.progress * 100.0),
                        cx,
                    )
                    .render(cx),
                )
                .child(det)
                .child(prog_slider)
                .render(cx),
        )
        .child(
            column("ws-indet-col", cx)
                .gap(Spacing::Sm)
                .child(label("ws-indet-lbl", "Indeterminate", cx).render(cx))
                .child(indet)
                .render(cx),
        )
        .render(cx);

    // Skeletons: line, block, avatar-shaped.
    let skeletons = row("ws-skel-row", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Lg)
        .child(
            skeleton("ws-sk-line", cx)
                .w(px(180.0))
                .h(px(12.0))
                .render(cx),
        )
        .child(
            skeleton("ws-sk-block", cx)
                .block(true)
                .w(px(200.0))
                .h(px(64.0))
                .render(cx),
        )
        .child(
            skeleton("ws-sk-avatar", cx)
                .block(true)
                .w(px(36.0))
                .h(px(36.0))
                .render(cx),
        )
        .render(cx);

    // Badges: one per variant.
    let badges = wrap("ws-badges", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Sm)
        .child(badge("ws-b-neutral", "Neutral", cx).render(cx))
        .child(
            badge("ws-b-success", "Success", cx)
                .variant(BadgeVariant::Success)
                .render(cx),
        )
        .child(
            badge("ws-b-warning", "Warning", cx)
                .variant(BadgeVariant::Warning)
                .render(cx),
        )
        .child(
            badge("ws-b-danger", "Danger", cx)
                .variant(BadgeVariant::Danger)
                .render(cx),
        )
        .child(
            badge("ws-b-info", "Info", cx)
                .variant(BadgeVariant::Info)
                .render(cx),
        )
        .render(cx);

    // Tags: normal, selected (click to toggle), closable, disabled.
    let entity_tag = entity.clone();
    let tags = wrap("ws-tags", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Sm)
        .child(tag("ws-t-plain", "rust", cx).render(cx))
        .child(
            tag("ws-t-sel", "selected", cx)
                .selected(app.tag_selected)
                .on_click(move |_ev, _w, cx| {
                    entity_tag.update(cx, |s, _cx| s.tag_selected = !s.tag_selected);
                })
                .render(cx),
        )
        .child(
            tag("ws-t-close", "closable", cx)
                .closable(true)
                .on_close(|_, _, _| {})
                .render(cx),
        )
        .child(
            tag("ws-t-disabled", "disabled", cx)
                .disabled(true)
                .render(cx),
        )
        .render(cx);

    // Tooltip on a button trigger.
    let tip_trigger = button("ws-tip-btn", cx)
        .on_click(|_, _, _| {})
        .render(cx)
        .child("Hover me");
    let tip = tooltip(
        "ws-tip",
        "A WinUI-styled tooltip with flyout shadow.",
        app.tooltip_state.clone(),
    )
    .trigger(tip_trigger.into_any_element())
    .render(cx);

    // Empty state.
    let empty = empty_state("ws-empty", cx)
        .icon(IconSource::Builtin("folder".into()))
        .title("Nothing here yet")
        .description("Add a project or pick a different folder to get started.")
        .render(cx);

    // Avatars: initials, name-derived, status dot, sizes.
    let avatars = wrap("ws-avatars", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Md)
        .child(avatar("ws-av-1", cx).initials("MS").render(cx))
        .child(avatar("ws-av-2", cx).name("Ada Lovelace").render(cx))
        .child(
            avatar("ws-av-3", cx)
                .name("Grace Hopper")
                .has_status(true)
                .render(cx),
        )
        .child(
            avatar("ws-av-4", cx)
                .name("Alan Turing")
                .size(px(48.0))
                .render(cx),
        )
        .child(
            avatar("ws-av-5", cx)
                .initials("XL")
                .size(px(56.0))
                .circle(false)
                .render(cx),
        )
        .render(cx);

    column("winui-status", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "status",
            column("winui-status-inner", cx)
                .gap(Spacing::Xl)
                .child(section("status-progress", "Progress", progress_section, cx))
                .child(section(
                    "status-skeleton",
                    "Skeleton loading",
                    skeletons,
                    cx,
                ))
                .child(section("status-badges", "Badges", badges, cx))
                .child(section("status-tags", "Tags", tags, cx))
                .child(section("status-tooltip", "Tooltip", tip, cx))
                .child(section("status-avatars", "Avatars", avatars, cx))
                .child(section("status-empty", "Empty state", empty, cx))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Dialogs & Flyouts
// ---------------------------------------------------------------------

fn dialogs_page(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    use yororen_ui::headless::disclosure::disclosure;
    use yororen_ui::headless::popover::popover;

    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Dialogs, cx);

    // Modal: buttons toggle the global modal layer mounted in the
    // app root (see `WinuiApp::build_modal_overlay`).
    let modal_state_open = app.modal_state.clone();
    let open_btn = button("wd-open-modal", cx)
        .variant(ActionVariantKind::Primary)
        .caption("Open dialog")
        .on_click(move |_, _, cx| {
            modal_state_open.update(cx, |st, _cx| st.open());
        })
        .render(cx);

    // Popover with a small flyout content.
    let popover_state = app.popover_state.clone();
    let popover_trigger = button("wd-pop-trigger", cx)
        .on_click({
            let st = popover_state.clone();
            move |_, _, cx| {
                st.update(cx, |s, _cx| s.toggle());
            }
        })
        .render(cx)
        .child("Toggle popover");
    let popover_content = column("wd-pop-body", cx)
        .gap(Spacing::Sm)
        .p(px(12.0))
        .child(
            label("wd-pop-t", "A flyout surface", cx)
                .strong(true)
                .render(cx),
        )
        .child(
            label(
                "wd-pop-d",
                "Acrylic-tinted fill, hairline stroke and an 8px overlay corner.",
                cx,
            )
            .muted(true)
            .render(cx),
        )
        .render(cx);
    let pop = popover("wd-popover", popover_state.clone())
        .trigger(popover_trigger.into_any_element())
        .content(popover_content.into_any_element())
        .render(cx);

    // Disclosure: an Expander-style header; the expanded body is
    // appended as a child by the caller.
    let entity_disc = entity.clone();
    let disc_body = div()
        .flex()
        .flex_col()
        .pl(px(48.0))
        .pr(px(16.0))
        .pb(px(16.0))
        .gap(px(4.0))
        .child(
            label(
                "wd-disc-l1",
                "The header is a 48px row with a 32px chevron box.",
                cx,
            )
            .muted(true)
            .render(cx),
        )
        .child(
            label(
                "wd-disc-l2",
                "Chevron swaps between the right and down Fluent arrows.",
                cx,
            )
            .muted(true)
            .render(cx),
        );
    // The body stays mounted in both directions so the reveal
    // element can animate expand AND collapse (200ms
    // `cubic-bezier(0,0,0,1)`, matching the reference Expander).
    let disc_body_reveal = yororen_ui_winui_renderer::animation::AnimatedRevealElement::new(
        "wd-disclosure-reveal",
        app.disclosure_open,
        disc_body,
    );
    let disc = disclosure("wd-disclosure", "Expander", cx)
        .open(app.disclosure_open)
        .on_toggle(move |_ev, _w, cx| {
            entity_disc.update(cx, |s, _cx| s.disclosure_open = !s.disclosure_open);
        })
        .render(cx)
        .child(disc_body_reveal);

    let _ = window;
    column("winui-dialogs", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "dialogs",
            column("winui-dialogs-inner", cx)
                .gap(Spacing::Xl)
                .child(section("dialogs-modal", "Content dialog", open_btn, cx))
                .child(section("dialogs-popover", "Popover / flyout", pop, cx))
                .child(section("dialogs-disclosure", "Expander", disc, cx))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Text & Typography
// ---------------------------------------------------------------------

fn text_page(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    use yororen_ui::headless::divider::divider;
    use yororen_ui::headless::keybinding_display::keybinding_display;
    use yororen_ui::headless::keybinding_input::keybinding_input;
    use yororen_ui::headless::shortcut_hint::shortcut_hint;
    use yororen_ui::headless::text::text;

    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Text, cx);

    // Heading ramp H1–H6.
    let headings = column("wt-headings", cx)
        .gap(Spacing::Xs)
        .child(heading("wt-h1", HeadingLevel::H1, "Heading 1 — 28px", cx).render(cx))
        .child(heading("wt-h2", HeadingLevel::H2, "Heading 2 — 20px", cx).render(cx))
        .child(heading("wt-h3", HeadingLevel::H3, "Heading 3 — 16px", cx).render(cx))
        .child(heading("wt-h4", HeadingLevel::H4, "Heading 4 — 14px", cx).render(cx))
        .child(heading("wt-h5", HeadingLevel::H5, "Heading 5 — 12px", cx).render(cx))
        .child(heading("wt-h6", HeadingLevel::H6, "Heading 6 — 11px", cx).render(cx))
        .render(cx);

    // Label variants + plain text.
    let labels = column("wt-labels", cx)
        .gap(Spacing::Xs)
        .child(label("wt-l-plain", "Plain label — body text", cx).render(cx))
        .child(
            label("wt-l-muted", "Muted label — secondary brush", cx)
                .muted(true)
                .render(cx),
        )
        .child(
            label("wt-l-strong", "Strong label — semibold", cx)
                .strong(true)
                .render(cx),
        )
        .child(
            label("wt-l-mono", "Mono label — Cascadia Mono", cx)
                .mono(true)
                .render(cx),
        )
        .child(text("wt-text", "Text spans use the same 14px body ramp.", cx).render(cx))
        .render(cx);

    // Dividers.
    let dividers = column("wt-dividers", cx)
        .gap(Spacing::Md)
        .child(divider("wt-div-1", cx).render(cx))
        .child(
            label("wt-div-note", "Hairline divider — 6% surface stroke", cx)
                .muted(true)
                .render(cx),
        )
        .child(divider("wt-div-2", cx).render(cx))
        .render(cx);

    // Keybinding display + shortcut hint + keybinding input.
    let entity_kbd = entity.clone();
    let kbd_input = keybinding_input("wt-kbd-input")
        .placeholder("Click, then press a chord…")
        .on_change(move |chord: &str, _w, cx| {
            entity_kbd.update(cx, |s, _cx| s.kbd = chord.to_string());
        })
        .render(cx, window);
    let kbd_row = column("wt-kbd-col", cx)
        .gap(Spacing::Md)
        .child(
            row("wt-kbd-row", cx)
                .items(AlignItems::Center)
                .gap(Spacing::Lg)
                .child(keybinding_display("wt-kbd-1", ["ctrl", "shift", "p"], cx).render(cx))
                .child(
                    shortcut_hint("wt-kbd-2", "Command palette", ["ctrl", "shift", "p"], cx)
                        .render(cx),
                )
                .child(shortcut_hint("wt-kbd-3", "Settings", ["ctrl", ","], cx).render(cx))
                .render(cx),
        )
        .child(kbd_input)
        .child(status_line(
            "kbd",
            format!("Captured chord: {:?}", app.kbd),
            cx,
        ))
        .render(cx);

    column("winui-text", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "text",
            column("winui-text-inner", cx)
                .gap(Spacing::Xl)
                .child(section("text-headings", "Headings", headings, cx))
                .child(section("text-labels", "Labels & text", labels, cx))
                .child(section("text-dividers", "Dividers", dividers, cx))
                .child(section("text-kbd", "Keyboard chords", kbd_row, cx))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Data & Tables
// ---------------------------------------------------------------------

fn data_page(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    use yororen_ui::headless::list_item::list_item;
    use yororen_ui::headless::table::{TableColumn, table};
    use yororen_ui::headless::tree::tree;
    use yororen_ui::headless::tree_item::tree_item;
    use yororen_ui::headless::uniform_virtual_list;
    use yororen_ui::headless::virtual_list;

    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Data, cx);

    // Table with row selection.
    let entity_table = entity.clone();
    let table_el = table("wd-table", cx)
        .columns(vec![
            TableColumn::new("name", "Name").width(140.0),
            TableColumn::new("lang", "Language").width(110.0),
            TableColumn::new("stars", "Stars").width(80.0),
        ])
        .rows(vec![
            vec!["yororen-ui".into(), "Rust".into(), "1.2k".into()],
            vec!["WinUIonWeb".into(), "TypeScript".into(), "860".into()],
            vec!["gpui-ce".into(), "Rust".into(), "3.4k".into()],
            vec!["fluent-icons".into(), "SVG".into(), "210".into()],
        ])
        .selected(app.table_selected)
        .on_select(move |i, _w, cx| {
            entity_table.update(cx, |s, _cx| s.table_selected = i);
        })
        .render(cx);

    // List items: plain, selected, with icons, disabled.
    let entity_li = entity.clone();
    let list_items = column("wd-lis", cx)
        .gap(Spacing::Xs)
        .child(
            list_item("wd-li-1", "Documents", cx)
                .description("12 files · modified today")
                .leading_icon("folder")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            list_item("wd-li-2", "Projects", cx)
                .description("4 files · modified yesterday")
                .leading_icon("folder")
                .trailing_icon("arrow-right")
                .selected(true)
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            list_item("wd-li-3", "Trash", cx)
                .leading_icon("trash")
                .disabled(true)
                .render(cx),
        )
        .render(cx);
    let _ = entity_li;

    // Tree: flatten the visible rows from app state.
    let tree_data = app.tree_data.clone();
    let tree_expanded = app.tree_expanded.clone();
    let tree_selected = app.tree_selected.clone();
    let mut tree_el = tree("wd-tree", cx)
        .data(tree_data.clone())
        .render(cx)
        .w(px(320.0));
    let visible = tree_data.flatten(&tree_expanded);
    for (id, depth) in visible {
        let has_children = !tree_data.children_of(&id).is_empty();
        let label_text = tree_data.label_of(&id).unwrap_or("").to_string();
        let is_expanded = tree_expanded.contains(&id);
        let is_selected = tree_selected.as_ref() == Some(&id);

        let entity_toggle = entity.clone();
        let entity_select = entity.clone();
        let toggle_id = id.clone();
        let select_id = id.clone();
        let row_id: gpui::ElementId = format!("wd-tree-row-{}", id.0).into();
        tree_el = tree_el.child(
            tree_item(row_id, id.clone(), label_text, cx)
                .depth(depth)
                .has_children(has_children)
                .expanded(is_expanded)
                .selected(is_selected)
                .on_toggle(move |_, _, cx| {
                    let tid = toggle_id.clone();
                    entity_toggle.update(cx, |s, _cx| {
                        if !s.tree_expanded.remove(&tid) {
                            s.tree_expanded.insert(tid);
                        }
                    });
                })
                .on_click(move |_, _, cx| {
                    entity_select.update(cx, |s, _cx| s.tree_selected = Some(select_id.clone()));
                })
                .render(cx, window),
        );
    }

    // Virtualized lists: 10k mixed rows + 1k uniform rows.
    let vl = virtual_list("wd-vl", &app.vl_controller, cx)
        .item_count(10_000)
        .row(move |ix, _w, cx| {
            let row_id: gpui::ElementId = format!("wd-vl-row-{ix}").into();
            list_item(row_id, format!("Virtual row #{ix}"), cx)
                .render(cx)
                .into_any_element()
        })
        .render(cx)
        .h(px(220.0));
    let uvl = uniform_virtual_list("wd-uvl", 1_000, &app.uvl_controller, cx)
        .row(move |ix, _w, cx| {
            let row_id: gpui::ElementId = format!("wd-uvl-row-{ix}").into();
            list_item(row_id, format!("Uniform row #{ix}"), cx)
                .render(cx)
                .into_any_element()
        })
        .render(cx)
        .h(px(220.0));

    column("winui-data", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "data",
            column("winui-data-inner", cx)
                .gap(Spacing::Xl)
                .child(section("data-table", "Table — click a row", table_el, cx))
                .child(section("data-list-items", "List items", list_items, cx))
                .child(section("data-tree", "Tree view", tree_el, cx))
                .child(section("data-vl", "Virtual list — 10,000 rows", vl, cx))
                .child(section(
                    "data-uvl",
                    "Uniform virtual list — 1,000 rows",
                    uvl,
                    cx,
                ))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}

// ---------------------------------------------------------------------
// Surfaces & Layout
// ---------------------------------------------------------------------

fn surfaces_page(
    app: &mut WinuiApp,
    window: &mut Window,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    use yororen_ui::headless::button_group::button_group;
    use yororen_ui::headless::form::form;
    use yororen_ui::headless::form_field::form_field;
    use yororen_ui::headless::image::{ImageSource, image};
    use yororen_ui::headless::panel::panel;

    let entity = cx.entity().clone();
    let header = page_header(WinuiPage::Surfaces, cx);
    let _ = app;

    // Cards: static + interactive (animated hover).
    let cards = row("wsv-cards", cx)
        .items(AlignItems::Stretch)
        .gap(Spacing::Md)
        .child(
            card_props("wsv-card-static", cx)
                .render(cx)
                .w(px(220.0))
                .child(
                    column("wsv-card-static-body", cx)
                        .gap(Spacing::Xs)
                        .child(
                            heading("wsv-card-static-t", HeadingLevel::H4, "Static card", cx)
                                .render(cx),
                        )
                        .child(
                            label(
                                "wsv-card-static-d",
                                "4px corners, 16px padding, hairline stroke.",
                                cx,
                            )
                            .muted(true)
                            .render(cx),
                        )
                        .render(cx),
                ),
        )
        .child(
            card_props("wsv-card-hover", cx)
                .interactive(true)
                .render(cx)
                .w(px(220.0))
                .child(
                    column("wsv-card-hover-body", cx)
                        .gap(Spacing::Xs)
                        .child(
                            heading("wsv-card-hover-t", HeadingLevel::H4, "Clickable card", cx)
                                .render(cx),
                        )
                        .child(
                            label(
                                "wsv-card-hover-d",
                                "Hover: fill lightens and the stroke deepens (83ms).",
                                cx,
                            )
                            .muted(true)
                            .render(cx),
                        )
                        .render(cx),
                ),
        )
        .render(cx);

    // Panel with a title.
    let panel_el = panel("wsv-panel", cx)
        .title("Panel")
        .render(cx)
        .w(px(320.0))
        .child(
            label(
                "wsv-panel-body",
                "A card-family surface for grouped content.",
                cx,
            )
            .muted(true)
            .render(cx),
        );

    // Image (any asset; use an embedded icon-derived resource).
    let image_el = image(
        "wsv-image",
        ImageSource::Resource("icons/user.svg".into()),
        cx,
    )
    .render(cx)
    .w(px(96.0))
    .h(px(96.0));

    // Button groups: attached segmented + detached cluster.
    let segmented = button_group("wsv-segmented", cx)
        .child(
            button("wsv-seg-day", cx)
                .caption("Day")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            button("wsv-seg-week", cx)
                .caption("Week")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            button("wsv-seg-month", cx)
                .caption("Month")
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .render(cx);
    let detached = button_group("wsv-detached", cx)
        .attached(false)
        .child(
            icon_button("wsv-det-bold", cx)
                .icon(IconSource::Builtin("pencil".into()))
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .child(
            icon_button("wsv-det-user", cx)
                .icon(IconSource::Builtin("user".into()))
                .on_click(|_, _, _| {})
                .render(cx),
        )
        .render(cx);
    let groups_row = row("wsv-groups", cx)
        .items(AlignItems::Center)
        .gap(Spacing::Lg)
        .child(segmented)
        .child(detached)
        .render(cx);

    // Form with labeled field, helper and error text.
    let entity_form = entity.clone();
    let name_input = text_input("wsv-form-name")
        .placeholder("Project name")
        .on_change(move |new: &str, _w, cx| {
            entity_form.update(cx, |s, _cx| s.text = new.to_string());
        })
        .render(cx, window);
    let form_el = form("wsv-form", cx).render(cx).w(px(320.0)).child(
        form_field("wsv-form-field", "Name", cx)
            .help("Shown on your profile page.")
            .required(true)
            .error("Name is already taken.")
            .input(name_input)
            .render(cx),
    );

    column("winui-surfaces", cx)
        .p(px(32.0))
        .gap(Spacing::Lg)
        .child(header)
        .child(card(
            "surfaces",
            column("winui-surfaces-inner", cx)
                .gap(Spacing::Xl)
                .child(section("surfaces-cards", "Cards", cards, cx))
                .child(section("surfaces-panel", "Panel", panel_el, cx))
                .child(section("surfaces-image", "Image", image_el, cx))
                .child(section("surfaces-groups", "Button groups", groups_row, cx))
                .child(section("surfaces-form", "Form", form_el, cx))
                .render(cx),
            cx,
        ))
        .render(cx)
        .into_any_element()
}
