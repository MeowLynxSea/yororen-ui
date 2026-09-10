//! `TextInputElement` — the inner painter for any single-line
//! text-input headless component.
//!
//! Lives in headless (not the renderer) because it carries no
//! theme knowledge: it shapes the line, paints the selection quad
//! and caret, and registers the IME pipeline against the
//! `TextInputState`. The colours are passed in by the caller
//! (`headless::XxxProps::render`), which reads them from the
//! registered `XxxRenderer`.

use std::time::Duration;

use gpui::{
    App, Bounds, Div, Element, ElementId, ElementInputHandler, EntityInputHandler, FocusHandle,
    GlobalElementId, Hsla, InteractiveElement, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, ShapedLine, SharedString, Stateful, Style,
    TextRun, Window, fill, hsla, point, px, relative, size,
};

use crate::action_handler;
use crate::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Escape, Home, Left, Paste, Right, SelectAll,
    SelectLeft, SelectRight, ShowCharacterPalette, TextInputActionHandler, TextInputState,
};

use std::ops::Range;

/// What the [`TextInputElement`] painter needs to read / write on
/// the state. The painter is generic over `T: TextInputPainterHost`
/// so the same painter drives both `TextInputState` (standalone
/// text inputs) and `ComboBoxState` (combo's embedded text input).
///
/// Implementors: `TextInputState` (delegates to its composed
/// `TextInputCore`) and `ComboBoxState` (composes its own
/// `TextInputCore` directly).
pub trait TextInputPainterHost: 'static {
    /// The text the painter should lay out. For most callers this
    /// is the raw value; for combo_box it's the *derived*
    /// display string (the typed query, or the label of the
    /// currently-selected option when the query is empty).
    fn display_value(&self) -> String;
    /// Placeholder shown when `display_value()` is empty.
    fn placeholder(&self) -> SharedString;
    /// Caret position, in UTF-8 bytes of `display_value()`.
    fn caret(&self) -> usize;
    /// Selected byte range (in `display_value()` bytes).
    fn selected_range(&self) -> Range<usize>;
    /// Horizontal scroll offset (pixels).
    fn scroll_x(&self) -> Pixels;
    /// Whether the caret quad should be painted.
    fn cursor_visible(&self) -> bool;
    fn set_cursor_visible(&mut self, visible: bool);
    /// Whether the cursor-blink task is currently running.
    /// `start_cursor_blink` claims the flag before spawning the
    /// task; the task releases it when focus moves elsewhere.
    fn cursor_blink_running(&self) -> bool;
    fn set_cursor_blink_running(&mut self, running: bool);
    /// Active IME composition range, in bytes of `display_value()`.
    fn marked_range(&self) -> Option<Range<usize>>;
    /// Per-line shaped layouts (empty for single-line inputs).
    fn last_line_layouts(&self) -> &[ShapedLine];
    /// Per-line byte ranges (parallel to `last_line_layouts`).
    fn last_line_byte_ranges(&self) -> &[Range<usize>];
    /// Line height in pixels.
    fn last_line_height(&self) -> Option<Pixels>;
    /// Cache the freshly-painted `ShapedLine`, the laid-out
    /// bounds, and the clamped horizontal scroll so the IME
    /// pipeline (`bounds_for_range` / `character_index_for_point`)
    /// can answer without re-shaping.
    fn update_paint_state(&mut self, line: ShapedLine, bounds: Bounds<Pixels>, scroll_x: Pixels);
    /// Focus handle for the embedded input. Used by the blink
    /// task and the IME registration.
    fn focus_handle(&self) -> FocusHandle;
}

/// How often the caret blinks while focused. 500ms matches the
/// gpui / WebKit convention.
pub const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// The inner element for any single-line text-input component.
///
/// Embedded by `headless::XxxProps::render` (for `text_input`,
/// `password_input`, `search_input`, etc.) inside the wrapper
/// `div` that supplies focus / keymap / border / padding.
pub struct TextInputElement<T: TextInputPainterHost + EntityInputHandler> {
    pub state: gpui::Entity<T>,
    pub focus_handle: FocusHandle,
    pub disabled: bool,
    pub text_color: Hsla,
    pub hint_color: Hsla,
    pub cursor_color: Hsla,
    pub selection_color: Hsla,
    pub placeholder: SharedString,
    /// When `Some`, used as the **display** text instead of
    /// `state.value` (for `password_input`'s mask). The state's
    /// real value still drives the caret, selection, IME and
    /// `on_change` — only the visual line is overridden. The
    /// override must have the same char-count as
    /// `state.value` so byte indices line up.
    pub value_override: Option<String>,
}

impl TextInputPainterHost for TextInputState {
    fn display_value(&self) -> String {
        self.value.clone()
    }
    fn placeholder(&self) -> SharedString {
        self.placeholder.clone()
    }
    fn caret(&self) -> usize {
        self.core.caret
    }
    fn selected_range(&self) -> Range<usize> {
        self.core.selected_range()
    }
    fn scroll_x(&self) -> Pixels {
        self.core.scroll_x
    }
    fn cursor_visible(&self) -> bool {
        self.core.cursor_visible
    }
    fn set_cursor_visible(&mut self, visible: bool) {
        self.core.cursor_visible = visible;
    }
    fn cursor_blink_running(&self) -> bool {
        self.core.cursor_blink_running
    }
    fn set_cursor_blink_running(&mut self, running: bool) {
        self.core.cursor_blink_running = running;
    }
    fn marked_range(&self) -> Option<Range<usize>> {
        self.core.marked_range.clone()
    }
    fn last_line_layouts(&self) -> &[ShapedLine] {
        &self.core.last_line_layouts
    }
    fn last_line_byte_ranges(&self) -> &[Range<usize>] {
        &self.core.last_line_byte_ranges
    }
    fn last_line_height(&self) -> Option<Pixels> {
        self.core.last_line_height
    }
    fn update_paint_state(&mut self, line: ShapedLine, bounds: Bounds<Pixels>, scroll_x: Pixels) {
        self.core.last_layout = Some(line);
        self.core.last_bounds = Some(bounds);
        self.core.scroll_x = scroll_x;
    }
    fn focus_handle(&self) -> FocusHandle {
        self.core.focus_handle()
    }
}

pub struct PrepaintState {
    pub line: Option<ShapedLine>,
    pub selection: Option<PaintQuad>,
    pub cursor: Option<PaintQuad>,
    pub scroll_x: Pixels,
    pub display_text: SharedString,
    pub cursor_visible: bool,
}

impl<T: TextInputPainterHost + EntityInputHandler> IntoElement for TextInputElement<T> {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl<T: TextInputPainterHost + EntityInputHandler> Element for TextInputElement<T> {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.state.read(cx);
        let value = input.display_value();
        let caret = input.caret();
        let selection = input.selected_range();
        let placeholder = input.placeholder();
        let scroll_x_input = input.scroll_x();
        let cursor_visible = input.cursor_visible();

        let is_empty = value.is_empty();
        let (display_text, run_color) = if is_empty {
            (placeholder, self.hint_color)
        } else if let Some(override_text) = &self.value_override {
            (SharedString::from(override_text.clone()), self.text_color)
        } else {
            (SharedString::from(value.clone()), self.text_color)
        };

        let mask_char_bytes = self
            .value_override
            .as_ref()
            .and_then(|s| s.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        let (caret_disp, selection_disp) = if self.value_override.is_some() {
            let real_value_caret = caret.min(value.len());
            let caret_chars = value[..real_value_caret].chars().count();
            let start_chars = value[..selection.start.min(value.len())].chars().count();
            let end_chars = value[..selection.end.min(value.len())].chars().count();
            (
                caret_chars * mask_char_bytes,
                (start_chars * mask_char_bytes)..(end_chars * mask_char_bytes),
            )
        } else {
            (caret, selection)
        };

        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: run_color,
            background_color: None,
            underline: None,
            strikethrough: None,
            letter_spacing: None,
        };
        let line = window
            .text_system()
            .shape_line(display_text.clone(), font_size, &[run], None);

        let caret_x_line = line.x_for_index(caret_disp);
        let cursor_thickness = px(2.0);
        let max_cursor_x = (bounds.size.width - cursor_thickness).max(Pixels::ZERO);
        let max_scroll_x = (line.width - max_cursor_x).max(Pixels::ZERO);
        let mut scroll_x = scroll_x_input.clamp(Pixels::ZERO, max_scroll_x);
        if caret_x_line < scroll_x {
            scroll_x = caret_x_line;
        } else if caret_x_line > scroll_x + max_cursor_x {
            scroll_x = caret_x_line - max_cursor_x;
        }
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        let (selection_quad, cursor_quad) = if !selection_disp.is_empty() && !is_empty {
            let start_x = line.x_for_index(selection_disp.start);
            let end_x = line.x_for_index(selection_disp.end);
            let quad = fill(
                Bounds::from_corners(
                    point(bounds.left() + start_x.min(end_x) - scroll_x, bounds.top()),
                    point(
                        bounds.left() + start_x.max(end_x) - scroll_x,
                        bounds.bottom(),
                    ),
                ),
                self.selection_color,
            );
            (Some(quad), None)
        } else {
            let cursor_paint_x = bounds.left() + caret_x_line - scroll_x;
            let quad = fill(
                Bounds::new(
                    point(cursor_paint_x, bounds.top()),
                    size(cursor_thickness, bounds.bottom() - bounds.top()),
                ),
                self.cursor_color,
            );
            (None, Some(quad))
        };

        PrepaintState {
            line: Some(line),
            selection: selection_quad,
            cursor: cursor_quad,
            scroll_x,
            display_text,
            cursor_visible,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !self.disabled {
            window.handle_input(
                &self.focus_handle,
                ElementInputHandler::new(bounds, self.state.clone()),
                cx,
            );
        }

        if let Some(sel) = prepaint.selection.take() {
            window.paint_quad(sel);
        }

        let line = prepaint
            .line
            .take()
            .expect("prepaint always produces a line");
        let origin_x = bounds.left() - prepaint.scroll_x;
        let _ = line.paint(
            point(origin_x, bounds.top()),
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        );

        let is_focused = self.focus_handle.is_focused(window);
        if is_focused
            && prepaint.cursor_visible
            && let Some(cur) = prepaint.cursor.take()
        {
            window.paint_quad(cur);
        }

        self.state.update(cx, |input, _cx| {
            input.update_paint_state(line, bounds, prepaint.scroll_x);
        });
    }
}

/// The submit callback (Enter / on_submit) shared by every input
/// component. Lives in headless because the action handler chain
/// references it directly.
pub type SubmitCallback = std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

/// Apply the text-input keymap (`track_focus` + `key_context` +
/// 14 `on_action` handlers + 3 mouse handlers) to `wrapper`.
///
/// Generic over the state type so the same wiring drives
/// `TextInputState` (the standalone input pipeline) and
/// `ComboBoxState` (which embeds a text input as part of its
/// trigger). Both implement [`TextInputActionHandler`].
///
/// The wrapper's `.id(...)` is NOT applied here — the caller is
/// expected to set it before calling, and the `key_context`
/// assumes the focus is in the right scope.
pub fn wire_input_keyboard<T: TextInputActionHandler>(
    mut wrapper: Stateful<Div>,
    state: gpui::Entity<T>,
    _focus_handle: FocusHandle,
    disabled: bool,
    on_submit: Option<SubmitCallback>,
) -> Stateful<Div> {
    wrapper = wrapper
        .key_context("UITextInput")
        .on_action(action_handler!(
            state.clone(),
            disabled,
            Backspace,
            backspace
        ))
        .on_action(action_handler!(state.clone(), disabled, Delete, delete))
        .on_action(action_handler!(state.clone(), disabled, Left, left))
        .on_action(action_handler!(state.clone(), disabled, Right, right))
        .on_action(action_handler!(
            state.clone(),
            disabled,
            SelectLeft,
            select_left
        ))
        .on_action(action_handler!(
            state.clone(),
            disabled,
            SelectRight,
            select_right
        ))
        .on_action(action_handler!(
            state.clone(),
            disabled,
            SelectAll,
            select_all
        ))
        .on_action(action_handler!(state.clone(), disabled, Home, home))
        .on_action(action_handler!(state.clone(), disabled, End, end))
        .on_action(action_handler!(
            state.clone(),
            disabled,
            ShowCharacterPalette,
            show_character_palette
        ))
        .on_action(action_handler!(state.clone(), disabled, Paste, paste))
        .on_action(action_handler!(state.clone(), disabled, Cut, cut))
        .on_action(action_handler!(state.clone(), disabled, Copy, copy))
        .on_action(action_handler!(state.clone(), disabled, Escape, escape));

    let state_for_enter = state.clone();
    let on_submit_for_enter = on_submit.clone();
    wrapper = wrapper.on_action(move |action: &Enter, window, cx| {
        if disabled {
            return;
        }
        state_for_enter.update(cx, |s, cx| {
            s.enter(action, window, cx);
            cx.notify();
        });
        let value = state_for_enter.read(cx).value();
        if let Some(cb) = on_submit_for_enter.as_ref() {
            cb(&value, window, cx);
        }
    });

    let state_for_mouse = state.clone();
    let state_for_up = state.clone();
    let state_for_up_out = state.clone();
    let state_for_move = state.clone();
    wrapper = wrapper
        .on_mouse_down(
            MouseButton::Left,
            move |event: &MouseDownEvent, window, cx| {
                state_for_mouse.update(cx, |s, cx| {
                    s.on_mouse_down(event.position, window, cx);
                    cx.notify();
                });
            },
        )
        .on_mouse_up(
            MouseButton::Left,
            move |event: &MouseUpEvent, window, cx| {
                state_for_up.update(cx, |s, cx| {
                    s.on_mouse_up(event, window, cx);
                    cx.notify();
                });
            },
        )
        .on_mouse_up_out(
            MouseButton::Left,
            move |event: &MouseUpEvent, window, cx| {
                state_for_up_out.update(cx, |s, cx| {
                    s.on_mouse_up(event, window, cx);
                    cx.notify();
                });
            },
        )
        // High-frequency: only repaint when the drag actually
        // moved the selection, not on every hover move.
        .on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
            state_for_move.update(cx, |s, cx| {
                if s.on_mouse_move(event, window, cx) {
                    cx.notify();
                }
            });
        });

    wrapper
}

/// Start the cursor-blink task unless one is already running.
///
/// `compose` calls this on **every frame** while the input is
/// focused, so it must be idempotent: the `cursor_blink_running`
/// flag is claimed before spawning and released when the task
/// exits, and the task itself keeps running across re-renders.
/// Restarting the task per frame (the old epoch-based scheme)
/// constantly reset its 500ms timer, and the dying task forced
/// `cursor_visible` back on just as the replacement toggled it
/// off — so the caret never actually blinked.
///
/// The task toggles `cursor_visible` every
/// [`CURSOR_BLINK_INTERVAL`] and notifies to repaint. It exits
/// (restoring a visible caret) as soon as focus has moved
/// elsewhere or the state entity is gone.
pub fn start_cursor_blink<T: TextInputPainterHost>(
    state: gpui::Entity<T>,
    window: &mut Window,
    cx: &mut App,
) {
    if state.read(cx).cursor_blink_running() {
        return;
    }
    state.update(cx, |s, _cx| s.set_cursor_blink_running(true));
    let state_weak = state.downgrade();
    window
        .spawn(cx, async move |async_cx| {
            loop {
                async_cx
                    .background_executor()
                    .timer(CURSOR_BLINK_INTERVAL)
                    .await;
                let should_continue = async_cx
                    .update(|window, cx| {
                        state_weak
                            .update(cx, |s, cx| {
                                if !s.focus_handle().is_focused(window) {
                                    s.set_cursor_visible(true);
                                    s.set_cursor_blink_running(false);
                                    cx.notify();
                                    return false;
                                }
                                s.set_cursor_visible(!s.cursor_visible());
                                cx.notify();
                                true
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if !should_continue {
                    return;
                }
            }
        })
        .detach();
}

// `hsla` re-export so callers don't have to import gpui::hsla
// just to build a `HintColor` or `CursorColor`. Kept here so the
// headless text-input element stays renderer-agnostic.
#[allow(dead_code)]
pub(crate) fn hsla_default() -> Hsla {
    hsla(0.0, 0.0, 0.0, 1.0)
}

#[cfg(test)]
mod input_interaction_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};

    /// The blink task must spawn once per focus-in (not once per
    /// render frame), keep toggling the caret every
    /// [`CURSOR_BLINK_INTERVAL`] across the re-renders its own
    /// `notify()` triggers, and — once focus moves elsewhere —
    /// restore a solid caret and release the running flag so the
    /// next focus-in can respawn it.
    ///
    /// Regression test: `compose` calls `start_cursor_blink` on
    /// every frame while focused. Under the old epoch-based
    /// scheme each call respawned the task and reset its 500ms
    /// timer, and the dying task forced `cursor_visible` back on
    /// just as the replacement toggled it off — so the caret
    /// never actually blinked. The loop below simulates the
    /// per-frame re-render between ticks to pin that down.
    #[gpui::test]
    fn cursor_blink_task_survives_per_frame_restarts_and_releases_on_blur(cx: &mut TestAppContext) {
        let state = cx.update(|cx| cx.new(|cx| TextInputState::new(cx)));
        let cx = cx.add_empty_window();
        // The blink task parks on its interval timer between
        // ticks; run_until_parked would otherwise panic.
        cx.executor().allow_parking();

        // Focus-in frame: spawns the task.
        cx.update(|window, cx| {
            let focus_handle = state.read(cx).focus_handle();
            window.focus(&focus_handle, cx);
            assert!(focus_handle.is_focused(window));
            start_cursor_blink(state.clone(), window, cx);
        });
        assert!(state.read_with(cx, |s, _| s.core.cursor_blink_running));

        for cycle in 0..3 {
            cx.executor().advance_clock(CURSOR_BLINK_INTERVAL);
            cx.run_until_parked();
            let visible = state.read_with(cx, |s, _| s.core.cursor_visible);
            assert_eq!(
                visible,
                cycle % 2 == 1,
                "caret must alternate every interval"
            );

            // Simulate the repaint that the tick's `notify()`
            // schedules: `compose` runs again while focused and
            // re-calls `start_cursor_blink`. This must be a
            // no-op, not a respawn.
            cx.update(|window, cx| {
                let focus_handle = state.read(cx).focus_handle();
                if focus_handle.is_focused(window) {
                    start_cursor_blink(state.clone(), window, cx);
                }
            });
        }

        // Cycle 2 ended with the caret hidden; after blur the
        // task's next tick restores a solid caret and releases
        // the flag so a later focus-in can respawn.
        cx.update(|window, _| window.blur());
        cx.executor().advance_clock(CURSOR_BLINK_INTERVAL);
        cx.run_until_parked();
        assert!(state.read_with(cx, |s, _| s.core.cursor_visible));
        assert!(!state.read_with(cx, |s, _| s.core.cursor_blink_running));
    }

    /// Keymap-driven edits (backspace, delete, arrows, selection,
    /// cut, paste…) must repaint the moment the action runs.
    ///
    /// Regression test: the `action_handler!` macro used to
    /// mutate the state without calling `cx.notify()`, so the
    /// edit only reached the screen when the blink task's next
    /// tick happened to trigger a repaint — text appeared to be
    /// deleted "when the caret blinked off".
    #[gpui::test]
    fn keymap_actions_notify_without_waiting_for_blink_tick(cx: &mut TestAppContext) {
        let state = cx.update(|cx| {
            cx.new(|cx| {
                let mut s = TextInputState::new(cx);
                s.value = "hello".to_string();
                s.core.end(&s.value);
                s
            })
        });
        let cx = cx.add_empty_window();

        let notified = std::rc::Rc::new(std::cell::Cell::new(0_usize));
        let counter = notified.clone();
        cx.update(|_, cx| {
            cx.observe(&state, move |_, _| counter.set(counter.get() + 1))
                .detach();
        });

        let handler = crate::action_handler!(state, false, Backspace, backspace);
        cx.update(|window, cx| handler(&Backspace, window, cx));

        assert_eq!(state.read_with(cx, |s, _| s.value.clone()), "hell");
        assert_eq!(
            notified.get(),
            1,
            "backspace must notify (schedule a repaint) immediately"
        );

        // Selection actions go through the same macro; verify one
        // representative case also notifies.
        let handler = crate::action_handler!(state, false, SelectAll, select_all);
        cx.update(|window, cx| handler(&SelectAll, window, cx));
        assert!(state.read_with(cx, |s, _| s.core.has_selection()));
        assert_eq!(
            notified.get(),
            2,
            "select-all must notify (schedule a repaint) immediately"
        );
    }
}
