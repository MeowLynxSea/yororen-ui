//! Regression test: keyboard arrows must drive the Listbox
//! highlight inside the *real* showcase window structure
//! (titlebar + sidebar + scrollable content pane), not just in an
//! isolated renderer harness.
//!
//! Boots the actual `WinuiApp` on the Lists page, brute-force
//! scans for the listbox by clicking across the page until a row
//! gets selected, then presses arrow keys and asserts the
//! highlight index moved.

#[gpui::test]
fn showcase_listbox_arrows_after_click(cx: &mut gpui::TestAppContext) {
    use gpui::{AppContext, Modifiers, px};
    use std::ops::Deref;

    use crate::app::{WinuiApp, WinuiPage};

    cx.update(|cx| {
        yororen_ui_winui_renderer::install_with(cx, yororen_ui_winui_renderer::winui_dark());
    });
    let app_entity = cx.new(|cx: &mut gpui::Context<WinuiApp>| {
        let mut app = WinuiApp::new(cx);
        app.page = WinuiPage::Lists;
        app
    });
    let listbox_state = cx.update(|cx| app_entity.read(cx).listbox_state.clone());

    let window = cx
        .update(|cx| {
            cx.open_window(gpui::WindowOptions::default(), |_, _| app_entity.clone())
        })
        .unwrap();
    let mut cx = gpui::VisualTestContext::from_window(window.into(), &cx.deref());
    cx.run_until_parked();

    // Brute-force: click across the page until the listbox
    // selects something (proving we hit a row), then stop.
    let mut clicked_at = None;
    'scan: for y in (120..700i32).step_by(16) {
        for x in [260, 400, 540] {
            cx.simulate_click(
                gpui::Point::new(px(x as f32), px(y as f32)),
                Modifiers::default(),
            );
            let selected = cx.update(|_window, app| {
                listbox_state
                    .read(app)
                    .selected_value
                    .as_ref()
                    .map(|v: &gpui::SharedString| v.to_string())
            });
            if let Some(v) = selected {
                clicked_at = Some((x, y, v));
                break 'scan;
            }
        }
    }
    let Some((x, y, v)) = clicked_at else {
        panic!("scan never hit the listbox — page layout changed?");
    };
    eprintln!("REPRO clicked listbox at ({x},{y}) → selected {v:?}");

    // The user-reported broken path: after clicking a row, do
    // the arrow keys still move the highlight?
    let before = cx.update(|_window, app| listbox_state.read(app).highlighted_index);
    cx.simulate_keystrokes("down");
    let after = cx.update(|_window, app| listbox_state.read(app).highlighted_index);
    eprintln!("REPRO highlight {before:?} → {after:?}");
    assert_ne!(
        before, after,
        "arrow down after clicking a listbox row must move the highlight"
    );
}
