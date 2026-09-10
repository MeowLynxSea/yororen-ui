//! Window-level keyboard-nav regression tests for the WinUI
//! renderer's collection widgets (`Listbox`, `GridView`).
//!
//! These mount a real window with the WinUI renderer installed,
//! focus the widget's state handle, and simulate keystrokes —
//! exercising the full gpui dispatch path (focus handle →
//! dispatch tree → `on_key_down` listener) rather than calling
//! the state methods directly.

use std::ops::Deref;

use gpui::{App, Context, Entity, IntoElement, Render, TestAppContext, Window, px};
use yororen_ui_core::headless::grid_view::{GridViewOption, GridViewState, grid_view};
use yororen_ui_core::headless::listbox::{ListboxOption, ListboxState, listbox};

struct ListboxRoot {
    state: Entity<ListboxState>,
}

impl Render for ListboxRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        listbox("t-listbox", self.state.clone())
            .render(cx)
            .into_any_element()
    }
}

struct GridViewRoot {
    state: Entity<GridViewState>,
}

impl Render for GridViewRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        grid_view("t-gridview", self.state.clone())
            .render(cx)
            .into_any_element()
    }
}

fn seeded_listbox(cx: &mut App) -> Entity<ListboxState> {
    let state = ListboxState::new(cx);
    state.update(cx, |s, _cx| {
        s.set_options(vec![
            ListboxOption::new("apple", "Apple"),
            ListboxOption::new("banana", "Banana"),
            ListboxOption::new("cherry", "Cherry").disabled(true),
            ListboxOption::new("durian", "Durian"),
        ]);
    });
    state
}

fn seeded_gridview(cx: &mut App) -> Entity<GridViewState> {
    let state = GridViewState::new(cx);
    state.update(cx, |s, _cx| {
        s.set_columns(3);
        s.set_options(vec![
            GridViewOption::new("a", "A"),
            GridViewOption::new("b", "B"),
            GridViewOption::new("c", "C"),
            GridViewOption::new("d", "D"),
            GridViewOption::new("e", "E"),
            GridViewOption::new("f", "F"),
        ]);
    });
    state
}

#[gpui::test]
fn winui_listbox_arrow_keys_move_highlight(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        yororen_ui_winui_renderer::install_with(cx, yororen_ui_winui_renderer::winui_dark());
        seeded_listbox(cx)
    });

    let window = cx.add_window(|_window, _cx| ListboxRoot {
        state: state.clone(),
    });

    // Focus the listbox's own handle (the state mints it in `new`)
    // and drive the arrow keys through the real dispatch path.
    window
        .update(cx, |root, window, cx| {
            root.state.read(cx).focus_handle().focus(window, cx);
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "down");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(root.state.read(cx).highlighted_index, Some(0), "down from None → 0");
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "down");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(
            root.state.read(cx).highlighted_index,
            Some(1),
            "down moves one step to banana"
        );
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "down");
    window
        .update(cx, |root, _window, cx| {
        // Cherry (index 2) is disabled — the shared walk refuses
        // to land on it, so the highlight stays on banana.
        assert_eq!(
            root.state.read(cx).highlighted_index,
            Some(1),
            "down does not land on a disabled option"
        );
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "up");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(
            root.state.read(cx).highlighted_index,
            Some(0),
            "up wraps past disabled"
        );
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "enter");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(
            root.state
                .read(cx)
                .selected_value
                .as_ref()
                .map(|v| v.to_string()),
            Some("apple".to_string()),
            "enter picks the highlighted row"
        );
        })
        .unwrap();
}

#[gpui::test]
fn winui_gridview_arrow_keys_move_highlight(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        yororen_ui_winui_renderer::install_with(cx, yororen_ui_winui_renderer::winui_dark());
        seeded_gridview(cx)
    });

    let window = cx.add_window(|_window, _cx| GridViewRoot {
        state: state.clone(),
    });

    window
        .update(cx, |root, window, cx| {
            root.state.read(cx).focus_handle().focus(window, cx);
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "right");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(
            root.state.read(cx).highlighted_index,
            Some(0),
            "right from None → 0"
        );
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "down");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(root.state.read(cx).highlighted_index, Some(3), "down steps a row stride");
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "up");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(root.state.read(cx).highlighted_index, Some(0), "up steps back");
        })
        .unwrap();

    cx.simulate_keystrokes(window.into(), "enter");
    window
        .update(cx, |root, _window, cx| {
        assert_eq!(
            root.state
                .read(cx)
                .selected_value
                .as_ref()
                .map(|v| v.to_string()),
            Some("a".to_string()),
            "enter picks the highlighted tile"
        );
        })
        .unwrap();
}

// Keep `px` referenced for window sizing helpers in future tests.
#[allow(dead_code)]
fn _unused_px() -> gpui::Pixels {
    px(1.0)
}

#[gpui::test]
fn winui_listbox_arrows_after_row_click(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        yororen_ui_winui_renderer::install_with(cx, yororen_ui_winui_renderer::winui_dark());
        seeded_listbox(cx)
    });

    let window = cx.add_window(|_window, _cx| ListboxRoot {
        state: state.clone(),
    });
    let mut cx = gpui::VisualTestContext::from_window(*window.deref(), &*cx);

    // Click the first row (near the window's top-left; the
    // listbox shell pads by 2px and rows by ~8/4px, so (20, 20)
    // lands inside row 0). The selection assert below proves the
    // click actually hit a row before we exercise the arrows.
    cx.simulate_click(gpui::Point::new(px(20.0), px(20.0)), gpui::Modifiers::default());
    window
        .update(&mut cx, |root, _window, cx| {
        assert_eq!(
            root.state
                .read(cx)
                .selected_value
                .as_ref()
                .map(|v| v.to_string()),
            Some("apple".to_string()),
            "click on row 0 selects apple (proves the click landed)"
        );
        })
        .unwrap();

    // Now press down — after a click the highlight is still
    // None (click picks directly via ), so the first down
    // lands on 0 and the second on banana (1).
    cx.simulate_keystrokes("down down");
    window
        .update(&mut cx, |root, _window, cx| {
        assert_eq!(
            root.state.read(cx).highlighted_index,
            Some(1),
            "arrow down after clicking a row must still move the highlight"
        );
        })
        .unwrap();
}
