//! `InputsApp` — the demo's root view.

use gpui::{
    Context, Div, IntoElement, ParentElement, Render, Stateful, Styled, Window, div, hsla, px,
};
use yororen_ui::headless::file_path_input::file_path_input;
use yororen_ui::headless::keybinding_input::{KeybindingInputMode, keybinding_input};
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{Inset, Spacing, column};
use yororen_ui::headless::number_input::number_input;
use yororen_ui::headless::password_input::password_input;
use yororen_ui::headless::scroll_view::{ScrollViewController, scroll_view};
use yororen_ui::headless::search_input::search_input;
use yororen_ui::headless::text_area::text_area;
use yororen_ui::headless::text_input::text_input;

pub struct InputsApp {
    pub text_value: String,
    pub password_value: String,
    pub number_value: f64,
    pub search_value: String,
    pub file_path_value: String,
    pub keybinding_value: String,
    pub keybinding_mode: KeybindingInputMode,
    pub text_area_value: String,
}

impl Default for InputsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl InputsApp {
    pub fn new() -> Self {
        Self {
            text_value: String::new(),
            password_value: String::new(),
            number_value: 0.0,
            search_value: String::new(),
            file_path_value: String::new(),
            keybinding_value: String::new(),
            keybinding_mode: KeybindingInputMode::Idle,
            text_area_value: String::new(),
        }
    }
}

impl Render for InputsApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Unified scroll control for the page root: the native
        // `.overflow_scroll()` column is replaced by a `scroll_view`
        // (renderer-drawn scrollbar, thumb drag, track paging).
        let scroll_entity = window.use_keyed_state(
            gpui::ElementId::Name("inputs-scroll-controller".into()),
            cx,
            |_window, _cx: &mut gpui::Context<ScrollViewController>| ScrollViewController::new(),
        );
        let scroll = scroll_entity.read(cx).clone();
        let page = column("inputs-scroll", cx)
            .w_full()
            .gap(Spacing::Xl)
            .p(Inset::Xl)
            // Panel 1: text_input.
            .child(panel_with_label(
                "1. text_input",
                "Click to focus, type to insert. The status line shows the live value.",
                text_input("demo-text-input")
                    .placeholder("Type here…")
                    .on_change({
                        let entity = cx.entity();
                        move |new: &str, _window, cx| {
                            entity.update(cx, |s, _cx| s.text_value = new.to_string());
                        }
                    })
                    .render(cx, window),
                cx,
            ))
            .child(status_line(&format!("text_input value: {:?}", self.text_value)))
            // Panel 2: password_input.
            .child(panel_with_label(
                "2. password_input",
                "Masked display; real value lives in state.",
                password_input("demo-password")
                    .placeholder("Enter password…")
                    .mask_char('•')
                    .on_change({
                        let entity = cx.entity();
                        move |new: &str, _window, cx| {
                            entity.update(cx, |s, _cx| s.password_value = new.to_string());
                        }
                    })
                    .render(cx, window),
                cx,
            ))
            .child(status_line(&format!("password_input value: {:?}", self.password_value)))
            // Panel 3: number_input.
            .child(panel_with_label(
                "3. number_input",
                "− / + stepper buttons increment / decrement (clamped to [0, 100]).",
                number_input("demo-number")
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .value(self.number_value)
                    .on_change({
                        let entity = cx.entity();
                        move |new: f64, _window, cx| {
                            entity.update(cx, |s, _cx| s.number_value = new);
                        }
                    })
                    .on_increment({
                        let entity = cx.entity();
                        move |next: f64, _window, cx| {
                            entity.update(cx, |s, _cx| s.number_value = next);
                        }
                    })
                    .on_decrement({
                        let entity = cx.entity();
                        move |next: f64, _window, cx| {
                            entity.update(cx, |s, _cx| s.number_value = next);
                        }
                    })
                    .render(cx, window),
                cx,
            ))
            .child(status_line(&format!("number_input value: {}", self.number_value)))
            // Panel 4: search_input.
            .child(panel_with_label(
                "4. search_input",
                "Search icon at left, clear (×) button at right when non-empty. Escape clears.",
                search_input("demo-search")
                    .placeholder("Search…")
                    .on_change({
                        let entity = cx.entity();
                        move |new: &str, _window, cx| {
                            entity.update(cx, |s, _cx| s.search_value = new.to_string());
                        }
                    })
                    .on_clear(|_window, _cx| {
                        // The renderer's Escape handler
                        // already clears the value; this hook
                        // is for the caller to do extra work.
                    })
                    .render(cx, window),
                cx,
            ))
            .child(status_line(&format!("search_input value: {:?}", self.search_value)))
            // Panel 5: file_path_input.
            .child(panel_with_label(
                "5. file_path_input",
                "Folder icon at left, browse button at right. Click it to open a native file picker; the chosen path is written into the input.",
                file_path_input("demo-file-path")
                    .placeholder("/path/to/file")
                    .on_change({
                        let entity = cx.entity();
                        move |new: &str, _window, cx| {
                            entity.update(cx, |s, _cx| s.file_path_value = new.to_string());
                        }
                    })
                    .on_browse(|picked: &str, _window, _cx| {
                        // Renderer already wrote the picked
                        // path into the input's state and
                        // fired `on_change`; this hook is
                        // for the caller to do extra work
                        // (e.g. log, validate, store).
                        eprintln!("file_path_input picked: {picked:?}");
                    })
                    .render(cx, window),
                cx,
            ))
            .child(status_line(&format!(
                "file_path_input value: {:?}",
                self.file_path_value
            )))
            // Panel 6: keybinding_input.
            .child({
                let keybinding_value = self.keybinding_value.clone();
                let keybinding_mode = self.keybinding_mode;
                let entity = cx.entity();
                let entity_for_mode = entity.clone();
                let entity_for_cancel = entity.clone();
                let panel_el = panel_with_label(
                    "6. keybinding_input",
                    "Click to enter Capturing mode, then press a key combo (e.g. Ctrl+Shift+P). Escape cancels.",
                    keybinding_input("demo-keybinding")
                        .mode(keybinding_mode)
                        .on_change(move |new: &str, _window, cx| {
                            entity.update(cx, |s, _cx| s.keybinding_value = new.to_string());
                        })
                        .on_start_capture(move |_window, cx| {
                            entity_for_mode.update(cx, |s, _cx| {
                                s.keybinding_mode = KeybindingInputMode::Capturing;
                            });
                        })
                        .on_cancel_capture(move |_window, cx| {
                            entity_for_cancel.update(cx, |s, _cx| {
                                s.keybinding_mode = KeybindingInputMode::Idle;
                            });
                        })
                        .render(cx, window),
                    cx,
                );
                let mut s = panel_el;
                s = s.child(status_line(&format!(
                    "value: {}    mode: {}",
                    if keybinding_value.is_empty() {
                        "(unset)"
                    } else {
                        &keybinding_value
                    },
                    match keybinding_mode {
                        KeybindingInputMode::Idle => "Idle",
                        KeybindingInputMode::Capturing => "Capturing",
                    }
                )));
                s
            })
            // Panel 7: text_area.
            .child(panel_with_label(
                "7. text_area",
                "Multi-line. Enter inserts '\\n'. Backspace extends across line boundaries.",
                text_area("demo-text-area")
                    .placeholder("Type a multi-line message…")
                    .on_change({
                        let entity = cx.entity();
                        move |new: &str, _window, cx| {
                            entity.update(cx, |s, _cx| s.text_area_value = new.to_string());
                        }
                    })
                    .render(cx, window),
                cx,
            ))
            .child(status_line(&format!(
                "text_area value: {:?}",
                self.text_area_value
            )))
            .render(cx);

        scroll_view("inputs-scroll-view", &scroll, cx)
            .child(page.bg(hsla(0.0, 0.0, 0.97, 1.0)))
            .render(cx)
            .size_full()
    }
}

/// One panel: white card with title (strong label) + blurb
/// (small secondary label) + body. Mirrors layers_demo's
/// `panel_body`.
fn panel_with_label(
    title: &str,
    blurb: &str,
    body: impl IntoElement,
    cx: &mut Context<InputsApp>,
) -> Stateful<Div> {
    column("input-panel", cx)
        .w_full()
        .p(Inset::Lg)
        .gap(Spacing::Sm)
        .child(label("title", title, cx).strong(true).render(cx))
        .child(
            label("blurb", blurb, cx)
                .render(cx)
                .text_color(hsla(0.0, 0.0, 0.4, 1.0))
                .text_size(px(13.)),
        )
        .child(body)
        .render(cx)
        .bg(hsla(0.0, 0.0, 1.0, 1.0))
        .rounded(px(8.))
}

/// One-line status caption.
fn status_line(text: &str) -> Div {
    div()
        .text_color(hsla(0.0, 0.0, 0.4, 1.0))
        .text_size(px(12.))
        .child(text.to_string())
}
