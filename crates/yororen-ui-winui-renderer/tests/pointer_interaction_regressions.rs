//! Window-level regression tests for three pointer-interaction
//! defects in the WinUI renderer:
//!
//! 1. `ComboBox` in selection-only mode (`editable == false`) filters
//!    the option list by the *selected label* written into `text` by
//!    `set_value` / `pick`, so after picking an option every other
//!    option disappears and the user can no longer switch.
//! 2. `NumberInput`'s stepper buttons keep their `on_mouse_down`
//!    handlers when `disabled` is set, so `.disabled(true)` still
//!    mutates the value and fires the business callbacks.
//! 3. `Button` only clears its pressed state from an in-bounds
//!    `on_mouse_up`; pressing, dragging off the button and releasing
//!    leaves it visually pressed forever.
//!
//! Every test drives the real gpui dispatch path (window → hitbox →
//! listener) instead of calling renderer internals, so a green result
//! means the widget actually behaves correctly for a user.

use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gpui::{
    App, Context, Entity, IntoElement, Modifiers, MouseButton, ParentElement, Render, Styled,
    TestAppContext, VisualTestContext, Window, div, point, px,
};
use yororen_ui_core::headless::button::button;
use yororen_ui_core::headless::combo_box::{ComboBoxOption, ComboBoxState, combo_box};
use yororen_ui_core::headless::number_input::{NumberInputProps, number_input};
use yororen_ui_winui_renderer::animation::interaction_pressed;

/// Install the WinUI renderer + theme into a fresh test app.
fn install(cx: &mut App) {
    yororen_ui_winui_renderer::install_with(cx, yororen_ui_winui_renderer::winui_dark());
}

// =====================================================================
// 1. Selection-only ComboBox keeps every option reachable
// =====================================================================

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

fn seeded_combo(cx: &mut App) -> Entity<ComboBoxState> {
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
}

fn combo_value(cx: &mut VisualTestContext, state: &Entity<ComboBoxState>) -> Option<String> {
    cx.update(|_, cx| state.read(cx).value.as_ref().map(|v| v.to_string()))
}

/// Reset the combo to "nothing typed, nothing picked, dropdown open"
/// so a probe click starts from a clean slate.
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

/// Find a y coordinate inside the *second* dropdown row by clicking
/// candidate positions and watching for the Banana value. Doubles as
/// the calibration for every later probe: a `None` result means the
/// probe point never reached row 1 even with no filter in effect.
fn probe_row_one(cx: &mut VisualTestContext, state: &Entity<ComboBoxState>) -> Option<(f32, f32)> {
    // The dropdown starts just under the 32px trigger; scan a little
    // past it so a mis-estimate cannot silently pass.
    for y in (36..=140).step_by(4) {
        reopen_unfiltered(cx, state);
        let p = point(px(120.), px(y as f32));
        cx.simulate_click(p, Modifiers::default());
        if combo_value(cx, state).as_deref() == Some("banana") {
            return Some((120., y as f32));
        }
    }
    None
}

#[gpui::test]
fn selection_only_combo_box_can_switch_options_after_a_pick(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        install(cx);
        seeded_combo(cx)
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
        "selection-only ComboBox must still list every option after a pick: the dropdown is the only remaining way to Banana, and `editable == false` prevents clearing the filter by editing text"
    );
}

// =====================================================================
// 2. Disabled NumberInput ignores its stepper buttons
// =====================================================================

struct NumberInputRoot {
    disabled: bool,
    increments: Arc<AtomicUsize>,
    changes: Arc<AtomicUsize>,
}

impl Render for NumberInputRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let increments = self.increments.clone();
        let changes = self.changes.clone();
        let props = NumberInputProps {
            disabled: self.disabled,
            on_increment: Some(Arc::new(move |_v, _w, _cx| {
                increments.fetch_add(1, Ordering::SeqCst);
            })),
            on_change: Some(Arc::new(move |_v, _w, _cx| {
                changes.fetch_add(1, Ordering::SeqCst);
            })),
            ..number_input("t-number").value(3.0).step(1.0)
        };
        div().w(px(200.)).child(props.render(cx, window))
    }
}

fn count(c: &Arc<AtomicUsize>) -> usize {
    c.load(Ordering::SeqCst)
}

#[gpui::test]
fn disabled_number_input_steppers_do_not_edit_the_value(cx: &mut TestAppContext) {
    cx.update(install);
    let increments = Arc::new(AtomicUsize::new(0));
    let changes = Arc::new(AtomicUsize::new(0));
    let window = cx.add_window(|_, _| NumberInputRoot {
        disabled: false,
        increments: increments.clone(),
        changes: changes.clone(),
    });
    let mut cx = VisualTestContext::from_window(*window.deref(), &*cx);

    // Calibration: find the increment ("+") hitbox with the control
    // enabled. The stepper sits at the right edge of the 200px field,
    // 24x24px, vertically centred in the 32px row.
    let mut found: Option<f32> = None;
    for x in (120..=198).rev() {
        let before = count(&increments);
        cx.simulate_click(point(px(x as f32), px(19.)), Modifiers::default());
        if count(&increments) > before {
            found = Some(x as f32);
            break;
        }
    }
    let Some(x) = found else {
        panic!("calibration failed: the increment button could not be clicked while enabled");
    };

    // Same pixel, same widget, now disabled.
    window
        .update(&mut cx, |root, _window, cx| {
            root.disabled = true;
            cx.notify();
        })
        .unwrap();
    let before_inc = count(&increments);
    let before_change = count(&changes);

    cx.simulate_click(point(px(x), px(19.)), Modifiers::default());

    assert_eq!(
        count(&increments) - before_inc,
        0,
        "a disabled NumberInput must not fire `on_increment` when the + stepper is clicked"
    );
    assert_eq!(
        count(&changes) - before_change,
        0,
        "a disabled NumberInput must not fire `on_change` when the + stepper is clicked"
    );
}

// =====================================================================
// 3. Button releases its pressed state outside its bounds
// =====================================================================

struct ButtonRoot {
    id: gpui::ElementId,
}

impl Render for ButtonRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Bounded box at the window origin so "outside the button" is
        // a real place: a root element with an auto size would stretch
        // over the whole window and make every release look in-bounds.
        div()
            .w(px(140.))
            .h(px(32.))
            .child(button(self.id.clone(), cx).caption("Press").render(cx))
    }
}

#[gpui::test]
fn button_clears_pressed_state_when_released_outside(cx: &mut TestAppContext) {
    cx.update(install);
    let id: gpui::ElementId = "t-button".into();
    let window = cx.add_window(|_, _| ButtonRoot { id: id.clone() });
    let mut cx = VisualTestContext::from_window(*window.deref(), &*cx);

    let pressed = |cx: &mut VisualTestContext| cx.update(|_, cx| interaction_pressed(cx, &id));

    cx.simulate_mouse_down(
        point(px(5.), px(5.)),
        MouseButton::Left,
        Modifiers::default(),
    );
    assert!(
        pressed(&mut cx),
        "calibration: pressing inside the button must mark it pressed"
    );

    // Drag the pointer off the button and release there.
    cx.simulate_mouse_move(
        point(px(400.), px(400.)),
        Some(MouseButton::Left),
        Modifiers::default(),
    );
    cx.simulate_mouse_up(
        point(px(400.), px(400.)),
        MouseButton::Left,
        Modifiers::default(),
    );

    assert!(
        !pressed(&mut cx),
        "releasing the mouse outside a Button must clear its pressed state; otherwise the fill layer keeps rendering the pressed look"
    );
}
