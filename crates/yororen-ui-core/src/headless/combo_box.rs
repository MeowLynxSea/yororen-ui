//! Headless `combo_box` — text input + option list + keyboard
//! navigation. The renderer shows a text field + a dropdown list.
//!
//! ## Editable vs selection-only
//!
//! `editable` (default `true`) mirrors WinUI's `IsEditable`:
//!
//! - `true`  — the user may type free-form text. Typing filters
//!   the option list, and pressing <kbd>Enter</kbd> **commits**
//!   the current text as the value (firing `on_change`) when no
//!   option is highlighted, so values outside the option list
//!   are legal.
//! - `false` — selection-only combo. Typing no longer mutates the
//!   text; the value can only change by picking an option.

use std::ops::Range;
use std::sync::Arc;

use gpui::{
    App, AppContext, Bounds, Context, Div, ElementId, Entity, EntityInputHandler, FocusHandle,
    Focusable, InteractiveElement, Pixels, ShapedLine, SharedString, Stateful, UTF16Selection,
    Window,
};

use crate::animation::{AnimatedPresenceState, AnimatedVisibility};
use crate::headless::list_navigable::{ListNavigable, highlight_next, highlight_prev};
use crate::headless::text_input::{
    Backspace, Copy, Cut, Delete, End, Enter, Escape, Home, Left, Paste, Right, SelectAll,
    SelectLeft, SelectRight, ShowCharacterPalette, TextInputActionHandler,
};
use crate::headless::text_input_core::TextInputCore;
use crate::headless::text_input_element::TextInputPainterHost;

#[derive(Clone, Debug)]
pub struct ComboBoxOption {
    pub value: SharedString,
    pub label: SharedString,
}

impl ComboBoxOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

pub type ComboBoxChangeCallback = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct ComboBoxState {
    pub open: bool,
    pub animation: AnimatedVisibility,
    pub text: String,
    pub value: Option<SharedString>,
    pub options: Vec<ComboBoxOption>,
    pub highlighted_index: Option<usize>,
    pub placeholder: SharedString,
    pub dismiss_on_escape: bool,
    /// Editable mode (WinUI `IsEditable`, default `true`). When
    /// `false` the embedded text input ignores edit actions —
    /// the value can only change via `pick`. When `true`,
    /// <kbd>Enter</kbd> also commits free-form text as the value.
    pub editable: bool,
    pub core: TextInputCore,
    on_change: Option<ComboBoxChangeCallback>,
}

impl ComboBoxState {
    pub fn new(app: &mut App) -> Entity<Self> {
        let core = TextInputCore::new(app);
        app.new(|_| Self {
            open: false,
            animation: AnimatedVisibility::new(),
            text: String::new(),
            value: None,
            options: Vec::new(),
            highlighted_index: None,
            placeholder: "Search…".into(),
            dismiss_on_escape: true,
            editable: true,
            core,
            on_change: None,
        })
    }

    pub fn open(&mut self) {
        self.open = true;
        self.animation.show();
    }
    pub fn close(&mut self) {
        self.open = false;
        self.animation.hide();
    }
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.animation.toggle();
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
    pub fn is_visible(&self) -> bool {
        self.animation.is_visible()
    }
    pub fn set_text(&mut self, t: impl Into<String>) {
        self.text = t.into();
    }
    pub fn set_options(&mut self, opts: Vec<ComboBoxOption>) {
        self.options = opts;
    }
    pub fn set_value(&mut self, v: impl Into<SharedString>) {
        let v = v.into();
        self.value = Some(v.clone());
        if let Some(opt) = self.options.iter().find(|o| o.value == v) {
            self.text = opt.label.to_string();
        }
        // Keep the embedded text-input caret/selection valid.
        self.core.move_to(&self.text, self.text.len());
        self.core.selection_start = 0;
        self.core.selection_end = 0;
        self.core.scroll_x = Pixels::ZERO;
    }
    pub fn set_placeholder(&mut self, p: impl Into<SharedString>) {
        self.placeholder = p.into();
    }
    /// Switch editable / selection-only mode (WinUI
    /// `IsEditable`). See the module docs for the behavioural
    /// difference.
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }
    pub fn is_editable(&self) -> bool {
        self.editable
    }
    pub fn highlight(&mut self, i: usize) {
        if i < self.options.len() {
            self.highlighted_index = Some(i);
        }
    }
    pub fn highlight_next(&mut self) {
        highlight_next(self);
    }
    pub fn highlight_prev(&mut self) {
        highlight_prev(self);
    }
    pub fn set_on_change<F>(&mut self, f: F)
    where
        F: 'static + Send + Sync + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
    }
    pub fn invoke_change(&self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        if let Some(f) = &self.on_change {
            f(value, window, cx);
        }
    }
    /// Pick an option. Headless data-layer action — does
    /// NOT belong in the renderer. Writes the new value
    /// (which also resyncs `text` to the label), closes
    /// the dropdown, and fires `on_change`. Renderer
    /// composes visuals + wires this as the item's click.
    pub fn pick(&mut self, value: SharedString, window: &mut gpui::Window, cx: &mut App) {
        self.set_value(value.clone());
        self.open = false;
        self.animation.hide();
        self.invoke_change(value, window, cx);
    }
    /// Borrow the on_change callback (if any). Used by
    /// renderers to fire the user's pick handler after
    /// they mutate `value` themselves (since `set_value`
    /// alone doesn't fire `on_change`).
    pub fn on_change(&self) -> Option<&ComboBoxChangeCallback> {
        self.on_change.as_ref()
    }
    pub fn select_highlighted(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if let Some(i) = self.highlighted_index
            && let Some(opt) = self.options.get(i)
        {
            let value = opt.value.clone();
            self.text = opt.label.to_string();
            self.value = Some(value.clone());
            self.open = false;
            self.animation.hide();
            self.invoke_change(value, window, cx);
        }
    }
    /// Commit the current free-form text as the value (editable
    /// mode's <kbd>Enter</kbd> action when no option is
    /// highlighted). Fires `on_change` with the raw text —
    /// legal values are not restricted to the option list.
    pub fn commit_text(&mut self, window: &mut gpui::Window, cx: &mut App) {
        if self.text.is_empty() {
            return;
        }
        // If the text happens to exactly match an option's label,
        // commit that option's value so callers receive the
        // canonical value string.
        let value = match self.options.iter().find(|o| o.label == self.text) {
            Some(opt) => opt.value.clone(),
            None => SharedString::new(self.text.clone()),
        };
        self.value = Some(value.clone());
        self.open = false;
        self.animation.hide();
        self.invoke_change(value, window, cx);
    }
}

impl AnimatedPresenceState for ComboBoxState {
    fn visibility(&self) -> &AnimatedVisibility {
        &self.animation
    }
    fn visibility_mut(&mut self) -> &mut AnimatedVisibility {
        &mut self.animation
    }
}

impl ListNavigable for ComboBoxState {
    fn options_len(&self) -> usize {
        self.options.len()
    }
    fn highlighted_index(&self) -> Option<usize> {
        self.highlighted_index
    }
    fn set_highlighted(&mut self, i: usize) {
        self.highlighted_index = Some(i);
    }
}

// =====================================================================
// `TextInputPainterHost` — lets the generic `TextInputElement` paint
// the combo's embedded text input using `combo_state.text` as the
// display value and `combo_state.core` for caret / selection / IME.
// =====================================================================

impl TextInputPainterHost for ComboBoxState {
    fn display_value(&self) -> String {
        self.text.clone()
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

// =====================================================================
// `TextInputActionHandler` — keyboard actions for the embedded text
// input. Text mutations automatically open the dropdown so the user
// sees filtered options while typing. In selection-only mode
// (`editable == false`) every mutating action is a no-op — the
// value only changes via `pick`.
// =====================================================================

impl TextInputActionHandler for ComboBoxState {
    fn value(&self) -> String {
        self.text.clone()
    }

    fn left(&mut self, _: &Left, _w: &mut Window, _cx: &mut App) {
        self.core.left(&self.text);
    }
    fn right(&mut self, _: &Right, _w: &mut Window, _cx: &mut App) {
        self.core.right(&self.text);
    }
    fn select_left(&mut self, _: &SelectLeft, _w: &mut Window, _cx: &mut App) {
        self.core.select_left(&self.text);
    }
    fn select_right(&mut self, _: &SelectRight, _w: &mut Window, _cx: &mut App) {
        self.core.select_right(&self.text);
    }
    fn select_all(&mut self, _: &SelectAll, _w: &mut Window, _cx: &mut App) {
        self.core.select_all(&self.text);
    }
    fn home(&mut self, _: &Home, _w: &mut Window, _cx: &mut App) {
        self.core.home();
    }
    fn end(&mut self, _: &End, _w: &mut Window, _cx: &mut App) {
        self.core.end(&self.text);
    }
    fn backspace(&mut self, _: &Backspace, _w: &mut Window, _cx: &mut App) {
        if !self.editable {
            return;
        }
        if self.core.backspace(&mut self.text) && !self.is_open() {
            self.open();
        }
    }
    fn delete(&mut self, _: &Delete, _w: &mut Window, _cx: &mut App) {
        if !self.editable {
            return;
        }
        if self.core.delete(&mut self.text) && !self.is_open() {
            self.open();
        }
    }
    fn paste(&mut self, _: &Paste, _w: &mut Window, cx: &mut App) {
        if !self.editable {
            return;
        }
        if self.core.paste(&mut self.text, false, cx) && !self.is_open() {
            self.open();
        }
    }
    fn copy(&mut self, _: &Copy, _w: &mut Window, cx: &mut App) {
        self.core.copy(&self.text, cx);
    }
    fn cut(&mut self, _: &Cut, _w: &mut Window, cx: &mut App) {
        if !self.editable {
            return;
        }
        if self.core.cut(&mut self.text, cx) && !self.is_open() {
            self.open();
        }
    }
    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _cx: &mut App,
    ) {
        self.core.show_character_palette(window);
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut App) {
        if self.highlighted_index.is_some() {
            self.select_highlighted(window, cx);
        } else if self.editable {
            // Editable mode: Enter commits the typed text as the
            // value, so free-form entries outside the option list
            // are legal.
            self.commit_text(window, cx);
        }
    }
    fn escape(&mut self, _: &Escape, _w: &mut Window, _cx: &mut App) {
        if self.dismiss_on_escape {
            self.close();
        }
    }

    fn on_mouse_down(
        &mut self,
        position: gpui::Point<gpui::Pixels>,
        window: &mut Window,
        _cx: &mut App,
    ) {
        self.core.on_mouse_down(&self.text, position, window, _cx);
    }
    fn on_mouse_up(&mut self, _event: &gpui::MouseUpEvent, _w: &mut Window, _cx: &mut App) {
        self.core.on_mouse_up();
    }
    fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _w: &mut Window,
        _cx: &mut App,
    ) -> bool {
        self.core.on_mouse_move(&self.text, event)
    }
}

// =====================================================================
// `EntityInputHandler` — the platform IME / clipboard pipeline for the
// embedded text input. Combo box fires `on_change` only on pick, not
// while typing, so these handlers do NOT notify or invoke the callback.
// =====================================================================

impl EntityInputHandler for ComboBoxState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let (text, adjusted) = TextInputCore::text_for_range_utf16(&self.text, range_utf16);
        *adjusted_range = Some(adjusted);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(self.core.selected_text_range_inner(&self.text))
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.core
            .marked_range
            .as_ref()
            .map(|r| TextInputCore::range_to_utf16(&self.text, r))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.core.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.editable {
            return;
        }
        let before = self.text.clone();
        let range = range_utf16
            .map(|r| TextInputCore::range_from_utf16(&self.text, &r))
            .or_else(|| self.core.marked_range.clone())
            .or_else(|| {
                if !self.core.selected_range().is_empty() {
                    Some(self.core.selected_range())
                } else {
                    None
                }
            });
        self.core
            .replace_text_in_range_bytes(&mut self.text, None, range, new_text);
        self.core.marked_range = None;
        if self.text != before && !self.is_open() {
            // Typing opens the dropdown so the filtered options
            // are visible — same behaviour as the keymap-driven
            // mutations in `TextInputActionHandler`.
            self.open();
        }
        // The IME pipeline bypasses `wire_input_keyboard`'s
        // notify-on-action, so without this the edit would only
        // reach the screen on the next cursor-blink tick (typed
        // letters would appear "after a delay").
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.editable {
            return;
        }
        let before = self.text.clone();
        let range = range_utf16.map(|r| TextInputCore::range_from_utf16(&self.text, &r));
        let new_sel =
            new_selected_range_utf16.map(|r| TextInputCore::range_from_utf16(&self.text, &r));
        self.core
            .replace_and_mark_text_in_range_bytes(&mut self.text, range, new_text, new_sel);
        if self.text != before && !self.is_open() {
            self.open();
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
        self.core
            .bounds_for_range_inner(&self.text, range_utf16, element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        self.core.character_index_for_point_inner(&self.text, point)
    }
}

// =====================================================================
// `Focusable for ComboBoxState` — the platform uses this to find the
// focus handle for the entity.
// =====================================================================

impl Focusable for ComboBoxState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.core.focus_handle()
    }
}

#[derive(Clone)]
pub struct ComboBoxProps {
    pub id: ElementId,
    pub state: Entity<ComboBoxState>,
}

pub fn combo_box(id: impl Into<ElementId>, state: Entity<ComboBoxState>) -> ComboBoxProps {
    ComboBoxProps {
        id: id.into(),
        state,
    }
}

impl ComboBoxProps {
    /// Apply the headless contract to the renderer-built `el`.
    /// Sets the element id only. The renderer is responsible
    /// for visuals AND for wiring the click handler that
    /// toggles the dropdown — the renderer composes the
    /// full UI (text input + dropdown) and decides how to
    /// route the click to avoid bubbling from option items
    /// back to the trigger.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Render the combo box using the registered
    /// `ComboBoxRenderer`. The renderer composes the full
    /// UI (a real text input trigger + a dropdown of filtered
    /// options) and returns an `AnyElement`. We then layer the
    /// headless `id` on top via `apply`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::renderer::RendererContext;
        use crate::renderer::combo_box::ComboBoxRenderer;
        use crate::renderer::markers::ComboBox as ComboBoxMarker;

        // `renderer_arc` returns `&Arc<dyn …>` which holds an
        // immutable borrow on `cx`. We need to pass `&mut cx`
        // to `compose` next. Clone the Arc to release the
        // immutable borrow before the call.
        let r: Arc<dyn ComboBoxRenderer> = cx
            .renderer_arc::<ComboBoxMarker, dyn ComboBoxRenderer>()
            .expect("ComboBoxRenderer registered")
            .clone();
        r.compose(&self, cx, window)
    }
}
