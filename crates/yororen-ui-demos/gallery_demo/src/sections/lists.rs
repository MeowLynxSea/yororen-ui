//! Section 8 — Lists, Tables, Trees, Forms, Misc. Each
//! component is wrapped in a labelled `cell` so the user can
//! identify every instance.

use std::collections::BTreeSet;

use gpui::{Context, ElementId, IntoElement, ParentElement, Styled, Window, px};

use yororen_ui::headless::button::button;
use yororen_ui::headless::form::form;
use yororen_ui::headless::form_field::form_field;
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{Spacing, column, row, spacer};
use yororen_ui::headless::list_item::list_item;
use yororen_ui::headless::listbox::listbox;
use yororen_ui::headless::radio_group::radio_group;
use yororen_ui::headless::table::TableColumn;
use yororen_ui::headless::table::table;
use yororen_ui::headless::text_input::text_input;
use yororen_ui::headless::tree::TreeData;
use yororen_ui::headless::tree::node_id;
use yororen_ui::headless::tree::tree;
use yororen_ui::headless::tree_item::tree_item;
use yororen_ui::headless::virtual_list::{uniform_virtual_list, virtual_list};
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

pub fn render(
    app: &mut GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    // Sync the virtual_list controller's item_count to the demo's
    // tracked `vl_item_count`. The `on_visible_range_change`
    // callback below only bumps `vl_item_count` (a plain field
    // update); it must NOT call `controller.reset/append(...)`
    // directly, because the callback fires from inside
    // `gpui::list`'s scroll path while `ListState`'s `RefCell` is
    // already borrowed by paint — calling reset/splice (which
    // both `borrow_mut` the same RefCell) would panic with
    // "RefCell already borrowed". Doing the sync here at the top
    // of `render` (one frame later) puts the borrow in a fresh
    // stack with no outstanding aliases.
    //
    // For *growth* we use `controller.append(n)` (which is
    // `splice(end..end, n)` internally) — this preserves the
    // current `logical_scroll_top` so the user's scroll position
    // stays put when new tail items arrive. `controller.reset(n)`
    // would clear `logical_scroll_top = None` and snap the list
    // back to the top, which is the wrong UX for infinite loading.
    // For *shrink* (current > target) we fall back to `reset`
    // because there is no "remove from tail" splice helper.
    let current = app.list_controller.state().item_count();
    if current < app.vl_item_count {
        app.list_controller.append(app.vl_item_count - current);
    } else if current > app.vl_item_count {
        app.list_controller.reset(app.vl_item_count);
    }

    // --- list_item: 3 selectable rows ---
    let first_label = cx.t("demo.lists.first_item").to_string();
    let second_label = cx.t("demo.lists.second_item").to_string();
    let third_label = cx.t("demo.lists.third_item").to_string();
    let list_items: Vec<(&'static str, String)> = vec![
        ("li-1", first_label),
        ("li-2", second_label),
        ("li-3", third_label),
    ];
    let mut list_col = column("lists-list-col", cx).gap(Spacing::Xs).w(px(220.));
    for (i, (id, title)) in list_items.iter().enumerate() {
        list_col = list_col.child(
            list_item(*id, title.clone(), cx)
                .selected(app.selected_list_item == Some(i))
                .render(cx),
        );
    }
    let list_wrapped = cell(cx.t("demo.lists.cell_list"), list_col.render(cx), cx);

    // --- listbox: scrollable single-select with keyboard nav ---
    // The listbox owns its own `Entity<ListboxState>`; the
    // shared `ListNavigable` algorithm drives `highlight_next`
    // / `highlight_prev` (Arrow Down/Up) and the renderer paints
    // one row per option, marking the highlighted and selected
    // rows. Click fires `state.pick(value, …)` which writes
    // `selected_value` itself and then invokes `on_change` — the
    // callback here ONLY updates the demo's mirror string; we
    // must NOT touch `listbox_state` again because we're still
    // inside the `update` that fired `on_change` (gpui panics
    // with "cannot update … while it is already being updated").
    let entity_for_lb = cx.entity().clone();
    app.listbox_state.update(cx, |s, _cx| {
        s.set_on_change(move |value, _w, cx| {
            let v = value.to_string();
            entity_for_lb.update(cx, |s, _cx| {
                s.listbox_demo_value = v;
            });
        });
    });
    let lb_label_template = cx.t("demo.lists.listbox_selected").to_string();
    let lb_selected_text = lb_label_template.replacen(
        "{}",
        if app.listbox_demo_value.is_empty() {
            "—"
        } else {
            app.listbox_demo_value.as_str()
        },
        1,
    );
    let lb_el = listbox("lists-listbox", app.listbox_state.clone()).render(cx);
    let lb_col = column("lists-listbox-col", cx)
        .gap(Spacing::Sm)
        .w(px(240.))
        .child(lb_el)
        .child(
            label("lists-listbox-status", lb_selected_text, cx)
                .muted(true)
                .render(cx),
        );
    let listbox_wrapped = cell(cx.t("demo.lists.cell_listbox"), lb_col.render(cx), cx);

    // --- form + form_field (with a real text_input + submit button) ---
    let entity_form = cx.entity().clone();
    let entity_text = cx.entity().clone();
    let email_input_el = text_input("lists-ff-email-input")
        .placeholder(cx.t("demo.form.email_placeholder"))
        .on_change(move |new: &str, _w, cx| {
            entity_text.update(cx, |s, _cx| s.form_email_value = new.to_string());
        })
        .render(cx, window);

    let form_field_el = form_field("lists-ff-email", "email", cx)
        .label(cx.t("demo.form.email_label"))
        .required(true)
        .input(email_input_el)
        .render(cx);

    // Capture the must-contain-@ message as an owned `String` so
    // the on_submit closure can read it without re-borrowing
    // `cx` (which is already mutably borrowed by the outer
    // `entity.update`).
    let must_contain_at_msg = cx.t("demo.form.must_contain_at").to_string();
    let form_props = form("lists-form", cx)
        .value("email", app.form_email_value.clone())
        .on_submit(move |vals, _w, cx| {
            entity_form.update(cx, |s, _cx| {
                s.form_submit_count += 1;
                if let Some(e) = vals.get("email") {
                    s.form_email_value = e.to_string();
                    s.form_email_error = if e.contains('@') {
                        None
                    } else {
                        Some(must_contain_at_msg.clone())
                    };
                }
            });
        })
        .submit(cx.t("demo.form.email_label").to_string());

    let submit_btn_el = form_props.submit_button(cx).expect("submit_label was set");

    let submitted_label = cx.t("demo.form.submitted").to_string();
    let last_error_label = cx.t("demo.form.last_error").to_string();
    let form_el = form_props
        .render(cx)
        .w(px(300.))
        .child(form_field_el)
        .child(submit_btn_el)
        .child(
            label(
                "form-submit-count",
                format!(
                    "{submitted_label} {} | {last_error_label} {:?}",
                    app.form_submit_count, app.form_email_error
                ),
                cx,
            )
            .muted(true)
            .render(cx),
        );
    let form_wrapped = cell(cx.t("demo.form.cell"), form_el, cx);

    // --- table ---
    let entity_table = cx.entity().clone();
    let col_name = cx.t("demo.lists.table_col_name").to_string();
    let col_age = cx.t("demo.lists.table_col_age").to_string();
    let col_city = cx.t("demo.lists.table_col_city").to_string();
    let row_alice = cx.t("demo.lists.table_row_alice").to_string();
    let row_bob = cx.t("demo.lists.table_row_bob").to_string();
    let row_carol = cx.t("demo.lists.table_row_carol").to_string();
    let age_30 = cx.t("demo.lists.table_row_age_30").to_string();
    let age_25 = cx.t("demo.lists.table_row_age_25").to_string();
    let age_28 = cx.t("demo.lists.table_row_age_28").to_string();
    let beijing = cx.t("demo.lists.table_row_beijing").to_string();
    let shanghai = cx.t("demo.lists.table_row_shanghai").to_string();
    let shenzhen = cx.t("demo.lists.table_row_shenzhen").to_string();
    let table_el = table("lists-table", cx)
        .columns(vec![
            TableColumn::new("name", col_name.clone()).width(120.),
            TableColumn::new("age", col_age.clone()).width(60.),
            TableColumn::new("city", col_city.clone()).width(120.),
        ])
        .rows(vec![
            vec![
                row_alice.clone().into(),
                age_30.clone().into(),
                beijing.clone().into(),
            ],
            vec![
                row_bob.clone().into(),
                age_25.clone().into(),
                shanghai.clone().into(),
            ],
            vec![
                row_carol.clone().into(),
                age_28.clone().into(),
                shenzhen.clone().into(),
            ],
        ])
        .selected(app.selected_table_row.unwrap_or(0))
        .on_select(move |i, _w, cx| {
            entity_table.update(cx, |s, _cx| s.selected_table_row = Some(i));
        })
        .render(cx)
        .w(px(320.));
    let table_wrapped = cell(cx.t("demo.lists.cell_table"), table_el, cx);

    // --- tree (with tree_item rows) ---
    let mut tree_data = TreeData::new();
    tree_data.add(None, node_id("root"), cx.t("demo.lists.tree_root"));
    tree_data.add(
        Some(node_id("root")),
        node_id("child1"),
        cx.t("demo.lists.tree_child1"),
    );
    tree_data.add(
        Some(node_id("root")),
        node_id("child2"),
        cx.t("demo.lists.tree_child2"),
    );
    tree_data.add(
        Some(node_id("child1")),
        node_id("leaf1"),
        cx.t("demo.lists.tree_leaf1"),
    );
    tree_data.add(
        Some(node_id("child1")),
        node_id("leaf2"),
        cx.t("demo.lists.tree_leaf2"),
    );
    let tree_data_for_iter = tree_data.clone();
    let entity_tree = cx.entity().clone();
    let tree_expanded: BTreeSet<_> = app.tree_expanded.clone();
    let tree_selected = app.tree_selected.clone();
    let mut tree_el = tree("lists-tree", cx)
        .data(tree_data)
        .render(cx)
        .w(px(240.));
    // Use the `TreeData::flatten` helper to walk only the
    // currently-visible (expanded) nodes, in depth-first
    // pre-order — see `TreeData::flatten`.
    let visible = tree_data_for_iter.flatten(&tree_expanded);
    for (id, depth) in visible {
        let has_children = !tree_data_for_iter.children_of(&id).is_empty();
        let label_text = tree_data_for_iter.label_of(&id).unwrap_or("").to_string();
        let is_expanded = tree_expanded.contains(&id);
        let is_selected = tree_selected.as_ref() == Some(&id);

        let entity_for_toggle = entity_tree.clone();
        let entity_for_select = entity_tree.clone();
        let toggle_id = id.clone();
        let select_id = id.clone();
        // Unique ElementId per row — gpui de-duplicates by id,
        // so identical ids would collapse all rows into one.
        let row_id: ElementId = format!("lists-tree-row-{}", id.0).into();
        // Double-click toggles: the renderer falls back to
        // `on_toggle` when `on_double_click` is not set, so we
        // don't need to wire it explicitly — but we wire it
        // here to make the behaviour explicit at the call site.
        let entity_for_double = entity_tree.clone();
        let double_id = id.clone();
        tree_el = tree_el.child(
            tree_item(row_id, id.clone(), label_text, cx)
                .depth(depth)
                .has_children(has_children)
                .expanded(is_expanded)
                .selected(is_selected)
                .on_toggle(move |_, _, cx| {
                    let toggle_id = toggle_id.clone();
                    entity_for_toggle.update(cx, |s, cx| {
                        if !s.tree_expanded.remove(&toggle_id) {
                            s.tree_expanded.insert(toggle_id);
                        }
                        cx.notify();
                    });
                })
                .on_click(move |_, _, cx| {
                    entity_for_select.update(cx, |s, cx| {
                        s.tree_selected = Some(select_id.clone());
                        cx.notify();
                    });
                })
                .on_double_click(move |_, _, cx| {
                    let double_id = double_id.clone();
                    entity_for_double.update(cx, |s, cx| {
                        if !s.tree_expanded.remove(&double_id) {
                            s.tree_expanded.insert(double_id);
                        }
                        cx.notify();
                    });
                })
                .render(cx, window),
        );
    }
    let tree_wrapped = cell(cx.t("demo.lists.cell_tree"), tree_el, cx);

    // --- multi-select tree (checkboxes + modifier clicks) ---
    // Same data as the single-select cell, running in
    // `TreeSelectionMode::Multiple`:
    //   * checkbox click toggles the node's whole subtree
    //     (`TreeData::subtree_ids`) — parents show the
    //     indeterminate dash while only some descendants are
    //     checked;
    //   * plain row click selects just that node, ctrl/cmd-click
    //     toggles it, shift-click selects the visible range from
    //     the anchor (`TreeData::select_range`).
    let tree_multi_selected = app.tree_multi_selected.clone();
    let tree_multi_expanded = app.tree_multi_expanded.clone();
    let mut tree_multi_el = tree("lists-tree-multi", cx)
        .data(tree_data_for_iter.clone())
        .multi_select()
        .selected_ids(tree_multi_selected.clone())
        .render(cx)
        .w(px(240.));
    let visible_multi = tree_data_for_iter.flatten(&tree_multi_expanded);
    for (id, depth) in visible_multi {
        let has_children = !tree_data_for_iter.children_of(&id).is_empty();
        let label_text = tree_data_for_iter.label_of(&id).unwrap_or("").to_string();
        let is_expanded = tree_multi_expanded.contains(&id);
        let checked = tree_multi_selected.contains(&id);
        // Indeterminate = parent with some (but not all)
        // descendants checked.
        let indeterminate = {
            let subtree = tree_data_for_iter.subtree_ids(&id);
            let mut descendants = subtree.iter().filter(|n| *n != &id);
            let some = descendants.clone().any(|n| tree_multi_selected.contains(n));
            let all = descendants.all(|n| tree_multi_selected.contains(n));
            has_children && some && !all
        };

        let entity_for_check = entity_tree.clone();
        let check_id = id.clone();
        let check_data = tree_data_for_iter.clone();
        let entity_for_multi_click = entity_tree.clone();
        let click_id = id.clone();
        let click_data = tree_data_for_iter.clone();
        // Double-click toggles expansion exactly like the
        // single-select tree above.
        let entity_for_multi_double = entity_tree.clone();
        let double_multi_id = id.clone();
        let row_multi_id: ElementId = format!("lists-tree-multi-row-{}", id.0).into();
        tree_multi_el = tree_multi_el.child(
            tree_item(row_multi_id, id.clone(), label_text, cx)
                .depth(depth)
                .has_children(has_children)
                .expanded(is_expanded)
                .selected(false)
                .checkbox(true)
                .checked(checked)
                .indeterminate(indeterminate)
                .on_check(move |_, _, cx| {
                    let data = check_data.clone();
                    entity_for_check.update(cx, |s, cx| {
                        let subtree = data.subtree_ids(&check_id);
                        if subtree.iter().all(|n| s.tree_multi_selected.contains(n)) {
                            for n in subtree {
                                s.tree_multi_selected.remove(&n);
                            }
                        } else {
                            for n in subtree {
                                s.tree_multi_selected.insert(n);
                            }
                        }
                        cx.notify();
                    });
                })
                .on_click(move |ev, _, cx| {
                    let data = click_data.clone();
                    let mods = match ev {
                        gpui::ClickEvent::Mouse(m) => m.down.modifiers,
                        _ => gpui::Modifiers::default(),
                    };
                    entity_for_multi_click.update(cx, |s, cx| {
                        if mods.shift
                            && let Some(anchor) = s.tree_anchor.clone()
                        {
                            s.tree_multi_selected =
                                data.select_range(&s.tree_multi_expanded, &anchor, &click_id);
                        } else if mods.control || mods.platform {
                            if !s.tree_multi_selected.remove(&click_id) {
                                s.tree_multi_selected.insert(click_id.clone());
                            }
                        } else {
                            s.tree_multi_selected = BTreeSet::from([click_id.clone()]);
                        }
                        s.tree_anchor = Some(click_id.clone());
                        cx.notify();
                    });
                })
                .on_toggle(move |_, _, cx| {
                    entity_for_multi_double.update(cx, |s, cx| {
                        if !s.tree_multi_expanded.remove(&double_multi_id) {
                            s.tree_multi_expanded.insert(double_multi_id.clone());
                        }
                        cx.notify();
                    });
                })
                .render(cx, window),
        );
    }
    let tree_multi_wrapped = cell(cx.t("demo.lists.cell_tree_multi"), tree_multi_el, cx);

    // --- virtual_list ---
    // 1000+ items (grows via infinite scroll), each rendered as
    // a `list_item` via the row closure handed to `gpui::list`.
    // The closure captures the GalleryApp entity so the row reads
    // the *current* selection state on every frame and `on_click`
    // updates it. Three new bits over the basic demo:
    //   1. `on_visible_range_change` updates `app.vl_visible_range`
    //      every scroll and, once the visible end approaches the
    //      logical end, calls `controller.reset(new_count)` to
    //      simulate infinite loading.
    //   2. Two buttons (Top / Bottom) drive
    //      `controller.scroll_to_top()` /
    //      `controller.scroll_to_bottom()`.
    //   3. A live status `label` shows the current visible range,
    //      total item_count, and the auto-loaded batch counter.
    //
    // The button on_click closures route through
    // `entity.update(cx, |s, _| s.list_controller.scroll_to_…())`
    // because `ButtonProps::on_click` requires `Send + Sync` but
    // `VirtualListController` is `Rc<RefCell<…>>` and thus
    // single-threaded. Going through `Entity` keeps the closure
    // Send + Sync (Entity is) and reaches the controller on the
    // main thread inside the update callback.
    let app_entity_for_vl = cx.entity().clone();
    let app_entity_for_range = cx.entity().clone();
    let vl_item_template = cx.t("demo.lists.vl_item").to_string();
    let vl = virtual_list("lists-vl", &app.list_controller, cx)
        .row(move |ix, _window, cx| {
            let app_entity = app_entity_for_vl.clone();
            // Read the *current* selected index from the entity on
            // each row construction; the closure is `FnMut` and is
            // re-invoked per visible row per frame.
            let selected = app_entity.read(cx).selected_list_item == Some(ix);
            let row_id: ElementId = format!("vl-row-{ix}").into();
            let row_label = vl_item_template.replacen("{}", &ix.to_string(), 1);
            list_item(row_id, row_label, cx)
                .selected(selected)
                .on_click(move |_ev, _window, cx| {
                    app_entity.update(cx, |s, _cx| {
                        s.selected_list_item = Some(ix);
                    });
                })
                .render(cx)
                .into_any_element()
        })
        .on_visible_range_change(move |range, total, _window, cx| {
            // Bump the demo's tracked counts only — the actual
            // `controller.reset(...)` happens at the top of the
            // next `render` (see comment there). Calling reset
            // from inside this callback would re-enter the
            // ListState `RefCell` that the gpui scroll path is
            // currently borrowing and panic with "RefCell already
            // borrowed".
            app_entity_for_range.update(cx, |s, _cx_inner| {
                s.vl_visible_range = Some(range.clone());
                if range.end + 50 >= total && s.vl_item_count < 5_000 {
                    s.vl_item_count += 100;
                    s.vl_load_count += 1;
                }
            });
        })
        .render(cx)
        .w(px(240.))
        .h(px(180.));
    // Control buttons + status label, stacked below the scrollable list.
    let entity_for_vl_top = cx.entity().clone();
    let entity_for_vl_bottom = cx.entity().clone();
    let top_btn = button("vl-top", cx)
        .on_click(move |_, _, cx| {
            entity_for_vl_top.update(cx, |s, _| s.list_controller.scroll_to_top());
        })
        .render(cx)
        .child(cx.t("demo.common.top"));
    let bottom_btn = button("vl-bottom", cx)
        .on_click(move |_, _, cx| {
            entity_for_vl_bottom.update(cx, |s, _| s.list_controller.scroll_to_bottom());
        })
        .render(cx)
        .child(cx.t("demo.common.bottom"));
    let controls_row = row("lists-vl-controls", cx)
        .gap(Spacing::Sm)
        .child(top_btn)
        .child(bottom_btn)
        .render(cx);
    let vl_status_template = cx.t("demo.lists.vl_status").to_string();
    let visible_str = format!("{:?}", app.vl_visible_range);
    let vl_status_text = vl_status_template
        .replacen("{:?}", &visible_str, 1)
        .replacen("{}", &app.vl_item_count.to_string(), 1)
        .replacen("{}", &app.vl_load_count.to_string(), 1);
    let status_label = label("vl-status", vl_status_text, cx)
        .muted(true)
        .render(cx);
    let vl_col = column("lists-vl-col", cx)
        .gap(Spacing::Sm)
        .child(vl)
        .child(controls_row)
        .child(status_label)
        .render(cx);
    let vl_wrapped = cell(cx.t("demo.lists.cell_vl"), vl_col, cx);

    // --- uniform_virtual_list ---
    // 1000 fixed-height rows. `gpui::uniform_list` measures only
    // the first row and lays out the rest in a line — much faster
    // than `gpui::list` for large uniform-height lists. The cell
    // also has Top / Bottom buttons wired to the
    // `UniformVirtualListController` (via `entity.update` for the
    // same Send + Sync reason as virtual_list above).
    let uvl_item_template = cx.t("demo.lists.uvl_item").to_string();
    let uvl = uniform_virtual_list("lists-uvl", 1_000, &app.uniform_list_controller, cx)
        .row(move |ix, _w, cx| {
            let row_id: ElementId = format!("uvl-row-{ix}").into();
            let row_label = uvl_item_template.replacen("{}", &ix.to_string(), 1);
            list_item(row_id, row_label, cx)
                .render(cx)
                .into_any_element()
        })
        .render(cx)
        .w(px(240.))
        .h(px(180.));
    let entity_for_uvl_top = cx.entity().clone();
    let entity_for_uvl_bottom = cx.entity().clone();
    let uvl_top_btn = button("uvl-top", cx)
        .on_click(move |_, _, cx| {
            entity_for_uvl_top.update(cx, |s, _| s.uniform_list_controller.scroll_to_top());
        })
        .render(cx)
        .child(cx.t("demo.common.top"));
    let uvl_bottom_btn = button("uvl-bottom", cx)
        .on_click(move |_, _, cx| {
            entity_for_uvl_bottom.update(cx, |s, _| s.uniform_list_controller.scroll_to_bottom());
        })
        .render(cx)
        .child(cx.t("demo.common.bottom"));
    let uvl_controls = row("lists-uvl-controls", cx)
        .gap(Spacing::Sm)
        .child(uvl_top_btn)
        .child(uvl_bottom_btn)
        .render(cx);
    let uvl_col = column("lists-uvl-col", cx)
        .gap(Spacing::Sm)
        .child(uvl)
        .child(uvl_controls)
        .render(cx);
    let uvl_wrapped = cell(cx.t("demo.lists.cell_uvl"), uvl_col, cx);

    // --- spacer ---
    let sp = spacer("lists-spacer", cx).render(cx).h(px(16.)).w_full();
    let sp_wrapped = cell(cx.t("demo.lists.cell_spacer"), sp, cx);

    // --- radio_group empty (also used as a layout shell) ---
    let rg_demo = radio_group("lists-rg", cx).name("rg-2").render(cx).child(
        label(
            "rg-2-info",
            cx.t("demo.controls.radio_group_empty_label"),
            cx,
        )
        .muted(true)
        .render(cx),
    );
    let rg_wrapped = cell(cx.t("demo.controls.cell_radio_group_empty"), rg_demo, cx);

    column("lists-root", cx)
        .gap(Spacing::Md)
        .child(list_wrapped)
        .child(listbox_wrapped)
        .child(form_wrapped)
        .child(table_wrapped)
        .child(tree_wrapped)
        .child(tree_multi_wrapped)
        .child(vl_wrapped)
        .child(uvl_wrapped)
        .child(sp_wrapped)
        .child(rg_wrapped)
        .render(cx)
}
