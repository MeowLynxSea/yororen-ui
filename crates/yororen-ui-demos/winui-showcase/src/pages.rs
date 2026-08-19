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
    }
}

// ---------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------

/// Page header: big title + subtitle + a hairline, like a WinUI
/// Gallery sample page.
fn page_header(
    page: WinuiPage,
    cx: &mut Context<WinuiApp>,
) -> gpui::AnyElement {
    let stroke = cx
        .theme()
        .get_color("winui.card_stroke")
        .unwrap_or_else(|| cx.theme().get_color("border.default").unwrap_or_default());
    column("winui-page-header", cx)
        .gap(Spacing::Sm)
        .child(
            heading("winui-page-title", HeadingLevel::H1, page.title(), cx).render(cx),
        )
        .child(label("winui-page-subtitle", page.subtitle(), cx).muted(true).render(cx))
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

fn status_line(id: impl std::fmt::Display, text: String, cx: &mut Context<WinuiApp>) -> gpui::AnyElement {
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
            heading("winui-hero-title", HeadingLevel::H1, "WinUI on Web Gallery", cx).render(cx),
        )
        .child(
            label("winui-hero-subtitle", "Bring WinUI experience to the web.", cx)
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
        .child(label(format!("winui-feature-{id}-d"), description, cx).muted(true).render(cx));

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
