//! Headless `text_input` — text state, IME, selection, on_change,
//! on_submit, key map. No visual.
//!
//! The headless layer owns the *textual* state machine: value,
//! caret/selection (in UTF-8 bytes), UTF-16 mirror for the IME
//! pipeline, scroll position, last shaped line, cursor blink.
//!
//! ## Why a `gpui::Entity<TextInputState>`
//!
//! `gpui-ce 0.3` requires the `EntityInputHandler` trait to be
//! implemented on a state held in an `Entity<T>`, so the platform
//! IME / clipboard layer can `update` it via `Window::handle_input`
//! without going through render closures. The renderer's
//! `default_render` mints the entity via `window.use_keyed_state`
//! and `paint` calls `window.handle_input(&focus_handle,
//! ElementInputHandler::new(bounds, entity.clone()), cx)` to
//! register the IME.
//!
//! ## Key dispatch
//!
//! Key dispatch lives in the renderer (the wrapper `div` has
//! `.key_context("UITextInput")` and a stack of `.on_action(...)`
//! calls). `init()` registers the keymap against the
//! `"UITextInput"` context (idempotent via `OnceLock`); apps that
//! want a different keymap simply don't call it.
//!
//! ## Shared core
//!
//! The caret / selection / IME / cursor-blink state machine is
//! the same one `ComboBoxState` needs for its embedded text
//! input. It's extracted to [`TextInputCore`](self::TextInputCore)
//! so `combo_box` can hold the same logic directly on its own
//! state — no separate `TextInputState` to sync.
//!
//! ## `Deref` pattern
//!
//! `TextInputState` composes a `TextInputCore` and implements
//! `Deref<Target = TextInputCore>`. This lets the existing
//! renderers and tests keep using `state.caret`,
//! `state.last_layout = ...`, etc. without change — those field
//! accesses auto-deref to `state.core.caret`,
//! `state.core.last_layout = ...`. The consumer-specific bits
//! (`value`, `placeholder`, `max_length`, `on_change`, …) live
//! directly on `TextInputState`; the shared caret/selection/IME
//! state lives on `TextInputCore`.

use std::ops::{Deref, DerefMut, Range};
use std::sync::{Arc, OnceLock};

use gpui::{
    App, Bounds, Context, EntityInputHandler, FocusHandle, Focusable, Hsla, KeyBinding, Pixels,
    SharedString, UTF16Selection, Window, actions,
};

pub use crate::headless::text_input_core::TextInputCore;

pub type TextChangeCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

// =====================================================================
// `actions!` — the 14 keyboard actions scoped to the `UITextInput`
// key context. The renderer wires `.on_action` to each; `init()`
// binds keys to them.
// =====================================================================

actions!(
    ui_text_input,
    [
        Backspace,
        Delete,
        Enter,
        Escape,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

/// Wire the text-input keymap into the running `App`. Idempotent —
/// subsequent calls are no-ops. Apps that want their own keymap
/// (e.g. an editor that uses `secondary-v` for paste globally)
/// simply don't call this.
pub fn init(cx: &mut App) {
    static DONE: OnceLock<()> = OnceLock::new();
    if DONE.set(()).is_err() {
        return;
    }
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("UITextInput")),
        KeyBinding::new("delete", Delete, Some("UITextInput")),
        KeyBinding::new("enter", Enter, Some("UITextInput")),
        KeyBinding::new("escape", Escape, Some("UITextInput")),
        KeyBinding::new("left", Left, Some("UITextInput")),
        KeyBinding::new("right", Right, Some("UITextInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("UITextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("UITextInput")),
        KeyBinding::new("secondary-a", SelectAll, Some("UITextInput")),
        KeyBinding::new("secondary-v", Paste, Some("UITextInput")),
        KeyBinding::new("secondary-c", Copy, Some("UITextInput")),
        KeyBinding::new("secondary-x", Cut, Some("UITextInput")),
        KeyBinding::new("home", Home, Some("UITextInput")),
        KeyBinding::new("end", End, Some("UITextInput")),
        KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UITextInput"),
        ),
    ]);
}

// =====================================================================
// `action_handler!` — for each action the renderer's wrapper div
// chains `.on_action(action_handler!(state, disabled, Backspace,
// backspace))`. The macro expands to a closure of type `Fn(&Action,
// &mut Window, &mut App)` that early-returns when disabled, then
// `Entity::update`s the state.
// =====================================================================

/// Build a `.on_action(...)` closure for one of the 14 actions.
///
/// Usage in the renderer:
/// ```ignore
/// .on_action(action_handler!(state, disabled, Backspace, backspace))
/// ```
#[macro_export]
macro_rules! action_handler {
    ($state:expr, $disabled:expr, $action:ty, $method:ident) => {{
        let state = $state.clone();
        let disabled = $disabled;
        move |action: &$action, window: &mut gpui::Window, cx: &mut gpui::App| {
            if disabled {
                return;
            }
            let _ = state.update(cx, |s, cx| {
                s.$method(action, window, cx);
                // The handler methods take `&mut App`, so they
                // can't notify themselves. Without this the edit
                // wouldn't repaint until the blink task's next
                // tick.
                cx.notify();
            });
        }
    }};
}

// =====================================================================
// `TextInputActionHandler` — the trait the renderer's `wire_input_keyboard`
// uses to wire the 14 actions + 3 mouse handlers to *any* state that
// hosts a text input. Implemented by `TextInputState` (the
// standalone input pipeline) and by `ComboBoxState` (which
// embeds a text input as part of its trigger). The trait is what
// lets the same `wire_input_keyboard` function drive both.
//
// All methods take `&mut App` (not `&mut Context<Self>`) so the
// trait is implementable by any state — the call site in the
// macro updates the entity through `cx.update(|s, cx| …)` and
// calls `cx.notify()` after the handler runs (gpui does NOT
// repaint on `Entity::update` by itself).
// =====================================================================

/// The state-machine contract that the text-input keymap wires
/// to. `wire_input_keyboard` calls each method via the
/// `action_handler!` macro; consumers must implement every
/// method (or accept the no-op default).
///
/// All method signatures take the action reference as their
/// first argument (even if unused) so the `action_handler!`
/// macro's call shape `s.$method(action, window, cx)` works
/// uniformly for all 14 actions.
pub trait TextInputActionHandler: 'static {
    /// Current text content. Used by the Enter handler to pass
    /// the value to the `on_submit` callback.
    fn value(&self) -> String;

    /// Backspace. Default: no-op.
    fn backspace(&mut self, _: &Backspace, _w: &mut Window, _cx: &mut App) {}
    /// Forward delete. Default: no-op.
    fn delete(&mut self, _: &Delete, _w: &mut Window, _cx: &mut App) {}
    /// Left arrow. Default: no-op.
    fn left(&mut self, _: &Left, _w: &mut Window, _cx: &mut App) {}
    /// Right arrow. Default: no-op.
    fn right(&mut self, _: &Right, _w: &mut Window, _cx: &mut App) {}
    /// Shift+Left. Default: no-op.
    fn select_left(&mut self, _: &SelectLeft, _w: &mut Window, _cx: &mut App) {}
    /// Shift+Right. Default: no-op.
    fn select_right(&mut self, _: &SelectRight, _w: &mut Window, _cx: &mut App) {}
    /// Cmd/Ctrl+A. Default: no-op.
    fn select_all(&mut self, _: &SelectAll, _w: &mut Window, _cx: &mut App) {}
    /// Home. Default: no-op.
    fn home(&mut self, _: &Home, _w: &mut Window, _cx: &mut App) {}
    /// End. Default: no-op.
    fn end(&mut self, _: &End, _w: &mut Window, _cx: &mut App) {}
    /// Paste. Default: no-op.
    fn paste(&mut self, _: &Paste, _w: &mut Window, _cx: &mut App) {}
    /// Copy. Default: no-op.
    fn copy(&mut self, _: &Copy, _w: &mut Window, _cx: &mut App) {}
    /// Cut. Default: no-op.
    fn cut(&mut self, _: &Cut, _w: &mut Window, _cx: &mut App) {}
    /// Show the platform character palette. Default: no-op.
    fn show_character_palette(&mut self, _: &ShowCharacterPalette, _w: &mut Window, _cx: &mut App) {
    }
    /// Enter / Return. Default: no-op. The wrapper also fires the
    /// consumer's `on_submit` callback after this hook.
    fn enter(&mut self, _: &Enter, _w: &mut Window, _cx: &mut App) {}
    /// Escape. Default: no-op.
    fn escape(&mut self, _: &Escape, _w: &mut Window, _cx: &mut App) {}

    /// Mouse-down on the input area. Default: no-op.
    fn on_mouse_down(
        &mut self,
        _position: gpui::Point<gpui::Pixels>,
        _w: &mut Window,
        _cx: &mut App,
    ) {
    }
    /// Mouse-up. Default: no-op.
    fn on_mouse_up(&mut self, _event: &gpui::MouseUpEvent, _w: &mut Window, _cx: &mut App) {}
    /// Mouse-move (drag-select). Returns `true` when the drag
    /// mutated the selection and a repaint is needed; plain
    /// hovers return `false`.
    fn on_mouse_move(
        &mut self,
        _event: &gpui::MouseMoveEvent,
        _w: &mut Window,
        _cx: &mut App,
    ) -> bool {
        false
    }
}

// =====================================================================
// `TextInputState` — the live state. Held in `Entity<TextInputState>`
// so the IME / action pipeline can update it.
//
// Composes [`TextInputCore`] (the shared caret/selection/IME
// state machine) and `Deref`s to it. The consumer-specific bits
// (the text content, placeholder, callbacks, `max_length`,
// `paste_newlines`) live directly on `TextInputState`.
// =====================================================================

/// The text state, caret, selection, scroll, and IME bookkeeping.
pub struct TextInputState {
    /// Text content (UTF-8 byte string). `String` not `SharedString`
    /// because every keystroke rewrites it; interning wouldn't help.
    pub value: String,
    /// Placeholder text shown when `value` is empty. The renderer
    /// paints it in the hint color.
    pub placeholder: SharedString,
    /// Hard cap on `value.len()`. `None` = unlimited. The
    /// `replace_text_in_range_bytes` method (used by every
    /// action method and the platform's `EntityInputHandler`)
    /// truncates to this cap so the value can never exceed it.
    pub max_length: Option<usize>,
    /// Fired whenever the value changes. The renderer sets this
    /// once in `default_render`; the action methods invoke it
    /// after a successful mutation. Closure receives the new
    /// value (UTF-8) plus a `Window` + `App` (the renderer's
    /// `on_change` signature).
    pub on_change: Option<TextChangeCallback>,
    /// Fired on Enter. The renderer sets this once and the
    /// `Enter` action handler invokes it.
    pub on_submit: Option<TextChangeCallback>,
    /// When `true`, `paste` keeps `'\n'` in the pasted text.
    /// `false` (the default) replaces `'\n'` with `' '` — the
    /// single-line input convention. `text_area`'s renderer
    /// sets this to `true` so multi-line paste works.
    pub paste_newlines: bool,
    /// The shared caret / selection / IME / cursor-blink state.
    /// Public so callers (renderers) can read/mutate via the
    /// `Deref` impl or directly via `state.core.X`. See
    /// [`TextInputCore`] for the field-by-field docs.
    pub core: TextInputCore,
}

impl Deref for TextInputState {
    type Target = TextInputCore;
    fn deref(&self) -> &TextInputCore {
        &self.core
    }
}

impl DerefMut for TextInputState {
    fn deref_mut(&mut self) -> &mut TextInputCore {
        &mut self.core
    }
}

impl TextInputState {
    /// Mint a new state. Called by the renderer via
    /// `window.use_keyed_state(id, cx, |_, cx| TextInputState::new(cx))`.
    pub fn new(cx: &mut App) -> Self {
        Self {
            value: String::new(),
            placeholder: SharedString::new_static(""),
            max_length: None,
            on_change: None,
            on_submit: None,
            paste_newlines: false,
            core: TextInputCore::new(cx),
        }
    }

    /// Public focus handle accessor (the `focus_handle` field is
    /// private because we implement the `Focusable` trait which
    /// exposes the same name).
    pub fn focus_handle(&self) -> FocusHandle {
        self.core.focus_handle()
    }

    // -- Pure data accessors -----------------------------------------

    /// Currently selected byte range. Empty when there's no
    /// selection (just a caret).
    pub fn selected_range(&self) -> Range<usize> {
        self.core.selected_range()
    }

    /// `true` if a non-empty selection is active.
    pub fn has_selection(&self) -> bool {
        self.core.has_selection()
    }

    /// Set `value` programmatically (e.g. clear button, initial
    /// value, load from disk). Resets the caret and selection.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.core.move_to(&self.value, self.value.len());
        self.core.selection_start = 0;
        self.core.selection_end = 0;
        self.core.scroll_x = Pixels::ZERO;
    }

    /// Convenience for external callers (e.g. on_submit) — clones
    /// the current value.
    pub fn content(&self) -> String {
        self.value.clone()
    }

    // -- UTF-8 ↔ UTF-16 helpers (the IME pipeline talks UTF-16) -----

    /// `byte_offset` (UTF-8) → `usize` UTF-16 code units.
    pub fn offset_to_utf16(&self, byte_offset: usize) -> usize {
        TextInputCore::offset_to_utf16(&self.value, byte_offset)
    }

    /// `utf16_offset` → UTF-8 byte offset.
    pub fn utf16_to_offset(&self, utf16_offset: usize) -> usize {
        TextInputCore::utf16_to_offset(&self.value, utf16_offset)
    }

    /// Convert a UTF-16 range to a UTF-8 byte range. `None` if
    /// either end fails to align to a char boundary.
    pub fn range_to_utf16(&self, byte_range: &Range<usize>) -> Range<usize> {
        TextInputCore::range_to_utf16(&self.value, byte_range)
    }

    /// Inverse.
    pub fn range_from_utf16(&self, utf16_range: &Range<usize>) -> Range<usize> {
        TextInputCore::range_from_utf16(&self.value, utf16_range)
    }

    /// `text_for_range` body — return the substring for a UTF-16
    /// range, plus the *adjusted* UTF-8 range the platform may
    /// have requested (the platform passes `range_utf16` and we
    /// answer with what we actually returned).
    pub fn text_for_range_utf16(&self, range_utf16: Range<usize>) -> (String, Range<usize>) {
        TextInputCore::text_for_range_utf16(&self.value, range_utf16)
    }

    // -- Char-boundary walking (UTF-8 safe) --------------------------

    /// `byte_offset` → previous char boundary (UTF-8 safe).
    pub fn prev_boundary(&self, byte_offset: usize) -> usize {
        TextInputCore::prev_boundary(&self.value, byte_offset)
    }

    /// `byte_offset` → next char boundary (UTF-8 safe).
    pub fn next_boundary(&self, byte_offset: usize) -> usize {
        TextInputCore::next_boundary(&self.value, byte_offset)
    }

    // -- Selection / caret mutation ----------------------------------

    /// Insert `text` at the current caret and advance the caret.
    /// Used by Enter-on-text-area (inserts '\n') and any
    /// caller that wants to type a single chunk without going
    /// through the IME / action pipeline.
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.replace_text_in_range_bytes(None, text);
    }

    /// Collapse selection to a single caret position.
    pub fn move_to(&mut self, offset: usize) {
        self.core.move_to(&self.value, offset);
    }

    /// Extend the selection to a new offset (keeps `selection_start`
    /// as the anchor and grows `selection_end`).
    pub fn select_to(&mut self, offset: usize) {
        self.core.select_to(&self.value, offset);
    }

    /// Replace the range `[start..end)` (UTF-8 bytes) with
    /// `new_text`. Updates caret, selection, and (if `fire_on_change`
    /// is true) the `on_change` callback is fired by the caller
    /// (the renderer's key handler or the `replace_text_in_range`
    /// platform pipeline).
    pub fn replace_text(&mut self, start: usize, end: usize, new_text: &str) {
        self.core
            .replace_text(&mut self.value, start, end, new_text);
    }

    /// Apply a `Range<usize>` (UTF-8) replacement. Used by the
    /// platform IME / clipboard via `EntityInputHandler`. A
    /// commit (a non-`replace_and_mark_text_in_range` call
    /// that targets the marked range) implicitly clears the
    /// composition.
    pub fn replace_text_in_range_bytes(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
    ) -> bool {
        self.core
            .replace_text_in_range_bytes(&mut self.value, self.max_length, range, new_text)
    }

    /// Compose (IME marked text). Replaces the active selection
    /// with `new_text` and records the marked range as the
    /// **span of the just-inserted text** (not the IME's
    /// `new_selected_range`, which is the cursor selection
    /// *within* the marked text). After a
    /// `setMarkedText("ni", …, replacementRange)` call,
    /// the marked range is `replacementRange.start..start +
    /// new_text.len()`. The eventual commit
    /// (`insertText(text, replacementRange)`) sends
    /// `replacementRange` = the marked range, so the commit
    /// replaces the pinyin with the final text.
    pub fn replace_and_mark_text_in_range_bytes(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
    ) {
        self.core.replace_and_mark_text_in_range_bytes(
            &mut self.value,
            range,
            new_text,
            new_selected_range,
        );
    }

    // -- `EntityInputHandler` body methods (not the trait impl) -----

    /// UTF-8 version of `EntityInputHandler::text_for_range`.
    pub fn text_for_range_inner(&self, range_utf16: Range<usize>) -> (String, Range<usize>) {
        TextInputCore::text_for_range_inner(&self.value, range_utf16)
    }

    /// UTF-8 version of `EntityInputHandler::selected_text_range`.
    pub fn selected_text_range_inner(&self) -> UTF16Selection {
        self.core.selected_text_range_inner(&self.value)
    }

    /// UTF-8 version of `EntityInputHandler::bounds_for_range`.
    /// Returns `None` if we haven't been laid out yet
    /// (`last_layout` is `None`).
    pub fn bounds_for_range_inner(
        &self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
    ) -> Option<Bounds<Pixels>> {
        self.core
            .bounds_for_range_inner(&self.value, range_utf16, element_bounds)
    }

    /// UTF-8 version of `EntityInputHandler::character_index_for_point`.
    pub fn character_index_for_point_inner(&self, point: gpui::Point<Pixels>) -> Option<usize> {
        self.core
            .character_index_for_point_inner(&self.value, point)
    }
}

// =====================================================================
// `EntityInputHandler for TextInputState` — the IME / clipboard
// pipeline. The 8 trait methods are thin wrappers around the
// `*_inner` methods above; the wrappers also call `cx.notify()`
// to trigger a re-render after a mutation.
// =====================================================================

impl EntityInputHandler for TextInputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let (text, adjusted) = self.text_for_range_inner(range_utf16);
        *adjusted_range = Some(adjusted);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(self.selected_text_range_inner())
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        // Return the UTF-16 of the active IME marked range.
        // The platform uses this to know where the composition
        // is when committing (or to query / replace it).
        self.marked_range.as_ref().map(|r| self.range_to_utf16(r))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // The platform is telling us to drop the composition
        // (e.g. focus lost, or the user pressed Escape mid-IME).
        // Clear the marked range and the selection; keep the
        // value as-is so the partially-typed pinyin is
        // preserved (the caller can decide to revert if it
        // wants).
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let before = self.value.clone();
        // On commit, the IME either passes the marked range
        // explicitly or we use our tracked one. If neither
        // is available, fall back to the active selection.
        let range = range_utf16
            .map(|r| self.range_from_utf16(&r))
            .or_else(|| self.marked_range.clone())
            .or_else(|| {
                if !self.selected_range().is_empty() {
                    Some(self.selected_range())
                } else {
                    None
                }
            });
        self.replace_text_in_range_bytes(range, new_text);
        // A commit clears the composition.
        self.marked_range = None;
        if self.value != before
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let before = self.value.clone();
        let range = range_utf16.map(|r| self.range_from_utf16(&r));
        let new_sel = new_selected_range_utf16.map(|r| self.range_from_utf16(&r));
        self.replace_and_mark_text_in_range_bytes(range, new_text, new_sel);
        if self.value != before
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        self.bounds_for_range_inner(range_utf16, element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        self.character_index_for_point_inner(point)
    }
}

// =====================================================================
// `Focusable for TextInputState` — the platform uses this to find
// the focus handle for the entity. Without this, `track_focus` and
// `Window::handle_input(&focus_handle, ...)` can't find the
// handle.
// =====================================================================

impl Focusable for TextInputState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.core.focus_handle()
    }
}

// =====================================================================
// Action handlers. Each takes `(&Action, &mut Window, &mut App)`.
// The `action_handler!` macro wires them into `.on_action` closures
// in the renderer and calls `cx.notify()` after each handler —
// `Entity::update` alone does NOT trigger a re-render in gpui,
// so without that notify the edit waits for the next blink tick.
//
// Each method delegates the actual caret/value mutation to
// `self.core.method(&mut self.value)` (or similar), then fires
// the consumer's `on_change` if the value actually changed.
//
// The same method names are used by `impl TextInputActionHandler`
// below — the trait is what the generic `wire_input_keyboard`
// requires.
// =====================================================================

impl TextInputState {
    pub fn left(&mut self, _: &Left, _window: &mut Window, _cx: &mut App) {
        self.core.left(&self.value);
    }

    pub fn right(&mut self, _: &Right, _window: &mut Window, _cx: &mut App) {
        self.core.right(&self.value);
    }

    pub fn select_left(&mut self, _: &SelectLeft, _window: &mut Window, _cx: &mut App) {
        self.core.select_left(&self.value);
    }

    pub fn select_right(&mut self, _: &SelectRight, _window: &mut Window, _cx: &mut App) {
        self.core.select_right(&self.value);
    }

    pub fn select_all(&mut self, _: &SelectAll, _window: &mut Window, _cx: &mut App) {
        self.core.select_all(&self.value);
    }

    pub fn home(&mut self, _: &Home, _window: &mut Window, _cx: &mut App) {
        self.core.home();
    }

    pub fn end(&mut self, _: &End, _window: &mut Window, _cx: &mut App) {
        self.core.end(&self.value);
    }

    pub fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut App) {
        if self.core.backspace(&mut self.value)
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
    }

    pub fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut App) {
        if self.core.delete(&mut self.value)
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
    }

    pub fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut App) {
        if self.core.paste(&mut self.value, self.paste_newlines, cx)
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
    }

    pub fn copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut App) {
        self.core.copy(&self.value, cx);
    }

    pub fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut App) {
        if self.core.cut(&mut self.value, cx)
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
    }

    pub fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _cx: &mut App,
    ) {
        self.core.show_character_palette(window);
    }

    // -- Mouse handlers (used by the renderer's on_mouse_*) -------

    pub fn on_mouse_down(
        &mut self,
        position: gpui::Point<Pixels>,
        window: &mut Window,
        _cx: &mut App,
    ) {
        self.core.on_mouse_down(&self.value, position, window);
    }

    pub fn on_mouse_up(
        &mut self,
        _event: &gpui::MouseUpEvent,
        _window: &mut Window,
        _cx: &mut App,
    ) {
        self.core.on_mouse_up();
    }

    pub fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        _cx: &mut App,
    ) -> bool {
        self.core.on_mouse_move(&self.value, event)
    }
}

// =====================================================================
// `TextInputActionHandler for TextInputState` — wires
// `TextInputState`'s `value()` getter to the trait (so the Enter
// handler in `wire_input_keyboard` can read the current text for
// `on_submit`).
// =====================================================================

impl TextInputActionHandler for TextInputState {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn backspace(&mut self, action: &Backspace, window: &mut Window, cx: &mut App) {
        self.backspace(action, window, cx);
    }
    fn delete(&mut self, action: &Delete, window: &mut Window, cx: &mut App) {
        self.delete(action, window, cx);
    }
    fn left(&mut self, action: &Left, window: &mut Window, cx: &mut App) {
        self.left(action, window, cx);
    }
    fn right(&mut self, action: &Right, window: &mut Window, cx: &mut App) {
        self.right(action, window, cx);
    }
    fn select_left(&mut self, action: &SelectLeft, window: &mut Window, cx: &mut App) {
        self.select_left(action, window, cx);
    }
    fn select_right(&mut self, action: &SelectRight, window: &mut Window, cx: &mut App) {
        self.select_right(action, window, cx);
    }
    fn select_all(&mut self, action: &SelectAll, window: &mut Window, cx: &mut App) {
        self.select_all(action, window, cx);
    }
    fn home(&mut self, action: &Home, window: &mut Window, cx: &mut App) {
        self.home(action, window, cx);
    }
    fn end(&mut self, action: &End, window: &mut Window, cx: &mut App) {
        self.end(action, window, cx);
    }
    fn paste(&mut self, action: &Paste, window: &mut Window, cx: &mut App) {
        self.paste(action, window, cx);
    }
    fn copy(&mut self, action: &Copy, window: &mut Window, cx: &mut App) {
        self.copy(action, window, cx);
    }
    fn cut(&mut self, action: &Cut, window: &mut Window, cx: &mut App) {
        self.cut(action, window, cx);
    }
    fn show_character_palette(
        &mut self,
        action: &ShowCharacterPalette,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.show_character_palette(action, window, cx);
    }

    fn on_mouse_down(
        &mut self,
        position: gpui::Point<gpui::Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.on_mouse_down(position, window, cx);
    }
    fn on_mouse_up(&mut self, event: &gpui::MouseUpEvent, window: &mut Window, cx: &mut App) {
        self.on_mouse_up(event, window, cx);
    }
    fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        self.on_mouse_move(event, window, cx)
    }
}

// =====================================================================
// `TextInputProps` — pure data carrier. No `state` (the renderer
// mints it via `use_keyed_state`), no `focus_handle` (lives in the
// state), no `apply` (the renderer builds the full element tree).
// =====================================================================

#[derive(Clone)]
pub struct TextInputProps {
    /// Stable element id used by `window.use_keyed_state` to
    /// persist the `TextInputState` across re-renders.
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<TextChangeCallback>,
    pub on_submit: Option<TextChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
    /// Optional initial value seeded by the caller. The
    /// renderer reads this when minting the
    /// `TextInputState` via `use_keyed_state` so the input
    /// starts with the supplied text. Distinct from the
    /// renderer's internal `value` field — `initial_value`
    /// is a one-shot seed, the live value lives in the
    /// state entity and is updated by every keystroke.
    pub initial_value: Option<String>,
}

/// Build a fresh `TextInputProps`. The factory takes only the
/// `id` — the focus handle and state are minted by the
/// renderer in `default_render`.
///
/// (The headless crate is now purely declarative; visual side
/// effects like `cx.focus_handle()` belong in the renderer.)
pub fn text_input(id: impl Into<gpui::ElementId>) -> TextInputProps {
    TextInputProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
        initial_value: None,
    }
}

impl TextInputProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    /// One-shot initial value. The renderer reads this when
    /// minting the `TextInputState` so the input starts
    /// with the supplied text. Subsequent edits live in
    /// the state entity and are not affected by changing
    /// `initial_value` on a later render (to reset the
    /// input at runtime, bump the `id` so a fresh state
    /// is minted).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.initial_value = Some(v.into());
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn max_length(mut self, v: usize) -> Self {
        self.max_length = Some(v);
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_submit = Some(Arc::new(f));
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_focus_border(mut self, v: bool) -> Self {
        self.has_custom_focus_border = v;
        self
    }
    pub fn custom_bg(mut self, c: Hsla) -> Self {
        self.custom_bg = Some(c);
        self.has_custom_bg = true;
        self
    }
    pub fn custom_border(mut self, c: Hsla) -> Self {
        self.custom_border = Some(c);
        self.has_custom_border = true;
        self
    }
    pub fn custom_focus_border(mut self, c: Hsla) -> Self {
        self.custom_focus_border = Some(c);
        self.has_custom_focus_border = true;
        self
    }
    pub fn custom_text_color(mut self, c: Hsla) -> Self {
        self.custom_text_color = Some(c);
        self
    }

    /// Render the text input using the registered `TextInputRenderer`.
    ///
    /// Data flow is one-way: hand the full `TextInputProps` to
    /// the registered renderer's `compose` and return the
    /// resulting `AnyElement`. No token values are pulled from
    /// the renderer here — every visual decision (bg, border,
    /// padding, radius, hover/active, text colour, the inner
    /// `TextInputElement` placement, the keymap wiring, focus
    /// tracking) lives in the renderer's `compose`.
    pub fn render(self, cx: &mut App, window: &mut Window) -> gpui::AnyElement {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::TextInput as TextInputMarker;
        use crate::renderer::text_input::TextInputRenderer;

        let r: Arc<dyn TextInputRenderer> = cx
            .renderer_arc::<TextInputMarker, dyn TextInputRenderer>()
            .expect("TextInputRenderer registered")
            .clone();

        r.compose(&self, cx, window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a test state without minting a real focus handle.
    /// The tests only exercise data helpers (UTF-16, boundaries,
    /// range ops) and never touch the focus handle. We leak
    /// each test state via `Box::leak` so the `FocusHandle`
    /// field's `Drop` never runs on a null-pointer `FocusMap`.
    /// Memory leak is fine for tests.
    fn test_state(value: &str) -> &'static mut TextInputState {
        let mut state = std::mem::MaybeUninit::<TextInputState>::zeroed();
        let ptr = state.as_mut_ptr();
        unsafe {
            (*ptr).value = value.into();
            // Fields on `core` (via DerefMut):
            (*ptr).core.caret = 0;
            (*ptr).core.selection_start = 0;
            (*ptr).core.selection_end = 0;
            (*ptr).core.scroll_x = Pixels::ZERO;
            (*ptr).core.is_selecting = false;
            (*ptr).core.cursor_visible = true;
            (*ptr).core.cursor_blink_running = false;
            (*ptr).core.marked_range = None;
            // Fields on TextInputState directly:
            (*ptr).placeholder = SharedString::new_static("");
            (*ptr).max_length = None;
            (*ptr).on_change = None;
            (*ptr).on_submit = None;
            // focus_handle left zeroed — tests never read it.
            // Leak so the `FocusHandle` field's `Drop` never
            // runs (which would walk into a null FocusMap Arc).
            Box::leak(Box::new(state.assume_init()))
        }
    }

    #[test]
    fn prev_next_boundary_walks_chars() {
        let s = test_state("héllo");
        // h é l l o  (é = 2 bytes)
        // 0 1    3 4 5
        assert_eq!(s.prev_boundary(5), 4); // before 'o' = start of last 'l'
        assert_eq!(s.prev_boundary(4), 3); // before second 'l' = start of first 'l'
        assert_eq!(s.prev_boundary(3), 1); // before first 'l' = start of 'é'
        assert_eq!(s.next_boundary(1), 3); // after 'é' = start of first 'l'
    }

    #[test]
    fn offset_to_utf16_handles_surrogates() {
        // 𝄞 (musical G clef) = U+1D11E = 4 UTF-8 bytes, 2 UTF-16 code units
        let s = test_state("a𝄞b");
        // 'a' = byte 0, '𝄞' = bytes 1..5, 'b' = byte 5
        assert_eq!(s.offset_to_utf16(0), 0);
        assert_eq!(s.offset_to_utf16(1), 1); // before 𝄞
        assert_eq!(s.offset_to_utf16(5), 3); // after 𝄞 (a=1, 𝄞=2)
        assert_eq!(s.offset_to_utf16(6), 4); // after b
    }

    #[test]
    fn text_for_range_returns_substring() {
        let s = test_state("hello");
        let (text, adj) = s.text_for_range_utf16(1..4);
        assert_eq!(text, "ell");
        assert_eq!(adj, 1..4);
    }

    #[test]
    fn replace_text_collapses_selection() {
        let s = &mut *test_state("hello");
        s.core.caret = 5;
        s.core.selection_start = 1;
        s.core.selection_end = 4;
        s.replace_text(1, 4, "XY");
        assert_eq!(s.value, "hXYo");
        assert_eq!(s.core.caret, 3);
        assert_eq!(s.core.selection_start, 3);
        assert_eq!(s.core.selection_end, 3);
    }

    #[test]
    fn backspace_at_zero_is_noop() {
        let s = &mut *test_state("hi");
        s.move_to(0);
        // No selection, caret=0 → replace_text(0,0,"") is a no-op
        s.replace_text(0, 0, "");
        assert_eq!(s.value, "hi");
        assert_eq!(s.core.caret, 0);
    }

    #[test]
    fn move_to_clamps() {
        let s = &mut *test_state("hi");
        s.move_to(100);
        assert_eq!(s.core.caret, 2);
    }

    #[test]
    fn selected_range_is_normalised() {
        let s = &mut *test_state("hello");
        s.core.selection_start = 4;
        s.core.selection_end = 1;
        assert_eq!(s.selected_range(), 1..4);
    }

    #[test]
    fn set_value_resets_state() {
        let s = &mut *test_state("old");
        s.core.caret = 2;
        s.core.selection_start = 0;
        s.core.selection_end = 2;
        s.set_value("fresh");
        assert_eq!(s.value, "fresh");
        assert_eq!(s.core.caret, 5);
        assert_eq!(s.core.selection_start, 0);
        assert_eq!(s.core.selection_end, 0);
    }

    #[test]
    fn ime_marked_range_round_trip() {
        // Simulate the IME pipeline:
        //   1. `replace_and_mark_text_in_range` marks "ni" as
        //      composition. The marked range is the byte span
        //      of the just-inserted text — `replacementRange`
        //      start..start + new_text.len() (NOT the IME's
        //      `new_selected_range`, which is the cursor
        //      selection *within* the marked text).
        //   2. `marked_text_range` returns the UTF-16 span so
        //      the platform can drive the commit.
        //   3. Commit via `replace_text_in_range` with the
        //      marked range — the pinyin is replaced.
        let s = &mut *test_state("");
        s.core.caret = 0;
        // IME: insert "ni" at position 0, mark 0..2 as composition.
        s.replace_and_mark_text_in_range_bytes(Some(0..0), "ni", None);
        assert_eq!(s.value, "ni");
        assert_eq!(s.core.marked_range.as_ref().unwrap().clone(), 0..2);
        // The IME's commit: `insertText(0..2, "你")` →
        // `replace_text_in_range(Some(0..2), "你")`.
        s.replace_text_in_range_bytes(Some(0..2), "你");
        assert_eq!(s.value, "你");
        assert!(s.core.marked_range.is_none());
    }

    #[test]
    fn ime_marked_range_uses_insertion_span_not_selection() {
        // Regression test: the IME sends a `new_selected_range`
        // that is the cursor position WITHIN the marked text
        // (typically `0..0` when the user just inserted pinyin).
        // The marked range itself must be the byte span of
        // the inserted text — using the IME's selection as
        // the marked range would result in a zero-width mark
        // and the commit would fail to replace the pinyin.
        let s = &mut *test_state("");
        // IME inserts "你好" at 0..0 and tells us the
        // selection within the new text is also 0..0
        // (cursor at the start of the marked text).
        s.replace_and_mark_text_in_range_bytes(Some(0..0), "你好", Some(0..0));
        assert_eq!(s.value, "你好");
        assert_eq!(s.core.marked_range.as_ref().unwrap().clone(), 0..6);
        assert_eq!(s.core.selection_start, 0);
        assert_eq!(s.core.selection_end, 0);
        // Commit replaces the marked span (0..6 bytes).
        s.replace_text_in_range_bytes(Some(0..6), "X");
        assert_eq!(s.value, "X");
    }

    #[test]
    fn ime_extending_composition_replaces_marked_not_appends() {
        // Regression test: when the IME updates an existing
        // composition (e.g. user types "s" then "a" while the
        // IME has "s" marked), it sends `setMarkedText` with
        // `replacementRange = NSNotFound` (None) to mean
        // "update the existing composition". Falling back to
        // the active selection would *append* the new pinyin
        // after the existing one (giving "ssa" or "sas"),
        // because the caret sits at the end of the marked
        // text. The fix: prefer `marked_range` over the
        // active selection when `replacementRange` is None.
        let s = &mut *test_state("");
        // Step 1: user types 's'. IME: setMarkedText("s", …,
        // replacementRange: 0..0). value = "s", marked = 0..1.
        s.replace_and_mark_text_in_range_bytes(Some(0..0), "s", Some(0..0));
        assert_eq!(s.value, "s");
        assert_eq!(s.core.marked_range.as_ref().unwrap().clone(), 0..1);
        // Step 2: user types 'a'. IME: setMarkedText("sa", …,
        // replacementRange: NSNotFound). The IME expects the
        // existing marked "s" to be REPLACED by "sa", not
        // appended after.
        s.replace_and_mark_text_in_range_bytes(None, "sa", Some(0..0));
        assert_eq!(s.value, "sa");
        assert_eq!(s.core.marked_range.as_ref().unwrap().clone(), 0..2);
        // Commit.
        s.replace_text_in_range_bytes(Some(0..2), "X");
        assert_eq!(s.value, "X");
    }
}
