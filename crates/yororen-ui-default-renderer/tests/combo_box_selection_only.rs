//! Regression test for the default `TokenComboBoxRenderer`: in
//! selection-only mode (`editable == false`) the option list is
//! filtered by the selected label that `set_value` writes into
//! `text`, so after picking Apple the only row left is Apple and the
//! user — who cannot edit the text — can never reach Banana again.
//!
//! The test clicks through the real gpui dispatch path (window →
//! hitbox → `on_click` → `pick`), so a green result is user-visible
//! behaviour, not an internal detail.

use std::ops::Deref;

use gpui::{
    Context, Entity, IntoElement, Modifiers, ParentElement, Render, Styled, TestAppContext,
    VisualTestContext, Window, div, point, px,
};
use yororen_ui_core::headless::combo_box::{ComboBoxOption, ComboBoxState, combo_box};

struct ComboBoxRoot {
    state: Entity<ComboBoxState>,
}

impl Render for ComboBoxRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Fixed width so the dropdown geometry is deterministic.
        div()
            .w(px(320.))
            .child(combo_box("t-combo", self.state.clone()).render(cx, window))
    }
}

fn combo_value(cx: &mut VisualTestContext, state: &Entity<ComboBoxState>) -> Option<String> {
    cx.update(|_, cx| state.read(cx).value.as_ref().map(|v| v.to_string()))
}

/// Reset to "nothing typed, nothing picked, dropdown open" so a probe
/// click starts from a clean slate.
fn reopen_unfiltered(cx: &mut VisualTestContext, state: &Entity<ComboBoxState>) {
    cx.update(|_, cx| {
        state.update(cx, |s, cx| {
            s.text.clear();
            s.value = None;
            s.open();
            cx.notify();
        });
    });
}

/// Find a point inside the *second* dropdown row by clicking candidate
/// positions and watching for the Banana value. Doubles as the
/// calibration for the real probe: `None` means the click never
/// reached row 1 even with no filter in effect.
fn probe_row_one(cx: &mut VisualTestContext, state: &Entity<ComboBoxState>) -> Option<(f32, f32)> {
    for y in (36..=140).step_by(4) {
        reopen_unfiltered(cx, state);
        cx.simulate_click(point(px(120.), px(y as f32)), Modifiers::default());
        if combo_value(cx, state).as_deref() == Some("banana") {
            return Some((120., y as f32));
        }
    }
    None
}

#[gpui::test]
fn selection_only_combo_box_can_switch_options_after_a_pick(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        yororen_ui_default_renderer::install_with(cx, yororen_ui_default_renderer::system_dark());
        let state = ComboBoxState::new(cx);
        state.update(cx, |s, cx| {
            s.set_options(vec![
                ComboBoxOption::new("apple", "Apple"),
                ComboBoxOption::new("banana", "Banana"),
            ]);
            // Selection-only: WinUI's `IsEditable = false`.
            s.set_editable(false);
            cx.notify();
        });
        state
    });
    let window = cx.add_window(|_, _| ComboBoxRoot {
        state: state.clone(),
    });
    let mut cx = VisualTestContext::from_window(*window.deref(), &*cx);

    // Calibration: with no filter applied, row 1 must be clickable.
    let Some((x, y)) = probe_row_one(&mut cx, &state) else {
        panic!(
            "calibration failed: could not click the second option even with an empty filter, so the probe never reached a rendered row"
        );
    };

    // The reported scenario: pick Apple (which writes "Apple" into
    // `text`), then reopen and try to switch to Banana.
    cx.update(|_, cx| {
        state.update(cx, |s, cx| {
            s.set_value("apple");
            s.open();
            cx.notify();
        });
    });
    assert_eq!(
        cx.update(|_, cx| state.read(cx).text.clone()),
        "Apple",
        "set_value must sync the trigger text to the picked label"
    );

    cx.simulate_click(point(px(x), px(y)), Modifiers::default());

    assert_eq!(
        combo_value(&mut cx, &state).as_deref(),
        Some("banana"),
        "selection-only ComboBox must still list every option after a pick: `editable == false` means the user cannot clear the filter by editing the text, so a filtered list traps them on the old value"
    );
}
