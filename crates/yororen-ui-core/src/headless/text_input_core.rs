//! `TextInputCore` — the shared caret/selection/cursor-blink/IME
//! state machine for any single-line text-input-style component.
//!
//! Lives in headless (not the renderer) because it carries no
//! theme knowledge: it owns the caret, selection, UTF-8 ↔ UTF-16
//! conversion, the IME marked-range bookkeeping, the cursor-blink
//! epoch, the mouse-select state, and the last shaped-line /
//! bounds cache that the IME pipeline reads back.
//!
//! ## Why a separate type?
//!
//! `TextInputState` (used by 7 standalone input renderers) and
//! `ComboBoxState` (which embeds a real text input as part of its
//! trigger) both need this exact state machine. By extracting it,
//! the combo_box can compose the same `TextInputElement` painter
//! plus the same 14 keyboard actions plus the same IME pipeline,
//! without minting a separate `Entity<TextInputState>` and
//! manually syncing its `value` with `combo_state.text`.
//!
//! ## `value` is the caller's
//!
//! Methods that work with the text content take `value: &str` /
//! `&mut String` as a parameter rather than reading a field on
//! `self`. The reason: `TextInputState` calls its text field
//! `value`; `ComboBoxState` calls it `text`; both want to use
//! this exact state machine. Keeping the text as a parameter
//! avoids forcing both consumers to rename or alias their
//! text field.
//!
//! ## Action methods don't fire `on_change`
//!
//! `backspace`, `delete`, `paste`, `cut` return `bool` (whether
//! the value actually changed) and leave firing `on_change` to
//! the caller — because the callback type varies between
//! consumers (`TextInputState` takes `&str`, `ComboBoxState`
//! takes `SharedString` for the *picked* value, not the typed
//! text).

use std::ops::Range;

use gpui::{App, Bounds, FocusHandle, Pixels, Point, ShapedLine, UTF16Selection, Window, point};

/// The shared text-input state machine. Composed into
/// `TextInputState` and `ComboBoxState`. Not a `Component` /
/// `Render` — it's a plain data carrier + pure logic.
#[derive(Clone)]
pub struct TextInputCore {
    /// Caret position, in UTF-8 bytes. `0..=value.len()`.
    pub caret: usize,
    /// Selection start, in UTF-8 bytes.
    /// `selection_start <= selection_end`.
    pub selection_start: usize,
    pub selection_end: usize,
    /// Horizontal scroll offset (pixels) when the text is wider
    /// than the visible width. Updated by the painter's
    /// `prepaint`; read by `bounds_for_range_inner` and
    /// `character_index_for_point_inner`.
    pub scroll_x: Pixels,
    /// The `ShapedLine` produced by the painter's `prepaint`.
    /// Cached so the IME's `bounds_for_range` /
    /// `character_index_for_point` can answer without
    /// re-shaping.
    pub last_layout: Option<ShapedLine>,
    /// The bounds of the text area, in window coordinates.
    /// Cached by the painter's `paint` for the same reason.
    pub last_bounds: Option<Bounds<Pixels>>,
    /// Per-line shaped layouts. **Empty** for single-line
    /// inputs (text_input, search_input, password_input, etc.).
    /// Populated by `TextAreaElement::paint` for multi-line.
    pub last_line_layouts: Vec<ShapedLine>,
    /// Byte range in the value for each row in
    /// `last_line_layouts`. See `TextInputState` for semantics.
    pub last_line_byte_ranges: Vec<Range<usize>>,
    /// Line height in pixels, captured at paint time.
    pub last_line_height: Option<Pixels>,
    /// `true` while the user is drag-selecting with the mouse.
    /// Toggled by the painter's mouse-down / mouse-up handlers.
    pub is_selecting: bool,
    /// Whether the caret quad is currently painted. Toggled by
    /// the cursor-blink task; every caret / text edit forces it
    /// back to `true` so the caret stays solid while typing.
    pub cursor_visible: bool,
    /// Whether the cursor-blink task spawned by
    /// `start_cursor_blink` is currently running. The flag makes
    /// that function idempotent — `compose` calls it on every
    /// frame while focused, but the task must only be spawned
    /// once per focus-in. It exits (and clears the flag) as soon
    /// as focus moves elsewhere.
    pub cursor_blink_running: bool,
    /// Active IME composition range, in **UTF-8 bytes**.
    pub marked_range: Option<Range<usize>>,
    /// Focus handle. Minted in `new`; private to keep the
    /// `Focusable` trait impl on the consumer in control of
    /// the public name.
    focus_handle: FocusHandle,
}

impl TextInputCore {
    /// Mint a fresh core with the focus handle from `cx`.
    pub fn new(cx: &mut App) -> Self {
        Self {
            caret: 0,
            selection_start: 0,
            selection_end: 0,
            scroll_x: Pixels::ZERO,
            last_layout: None,
            last_bounds: None,
            last_line_layouts: Vec::new(),
            last_line_byte_ranges: Vec::new(),
            last_line_height: None,
            is_selecting: false,
            cursor_visible: true,
            cursor_blink_running: false,
            marked_range: None,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Public focus-handle accessor.
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    // -- Pure state accessors (don't need the text value) -----------

    /// Currently selected byte range. Empty when there's no
    /// selection (just a caret).
    pub fn selected_range(&self) -> Range<usize> {
        self.selection_start.min(self.selection_end)..self.selection_start.max(self.selection_end)
    }

    /// `true` if a non-empty selection is active.
    pub fn has_selection(&self) -> bool {
        self.selection_start != self.selection_end
    }

    // -- Lifecycle ---------------------------------------------------

    /// Mouse-up: end the drag-select.
    pub fn on_mouse_up(&mut self) {
        self.is_selecting = false;
    }

    /// Show the platform's character palette (Ctrl+Cmd+Space).
    pub fn show_character_palette(&self, window: &mut Window) {
        window.show_character_palette();
    }

    // -- UTF-8 ↔ UTF-16 helpers (the IME pipeline talks UTF-16) -----

    /// `byte_offset` (UTF-8) → UTF-16 code units.
    pub fn offset_to_utf16(value: &str, byte_offset: usize) -> usize {
        let mut count = 0usize;
        for (i, c) in value.char_indices() {
            if i >= byte_offset {
                return count;
            }
            count += c.len_utf16();
        }
        count
    }

    /// UTF-16 offset → UTF-8 byte offset.
    pub fn utf16_to_offset(value: &str, utf16_offset: usize) -> usize {
        let mut count = 0usize;
        for (i, c) in value.char_indices() {
            if count >= utf16_offset {
                return i;
            }
            count += c.len_utf16();
        }
        value.len()
    }

    /// Convert a UTF-16 range to a UTF-8 byte range.
    pub fn range_to_utf16(value: &str, byte_range: &Range<usize>) -> Range<usize> {
        Self::offset_to_utf16(value, byte_range.start)..Self::offset_to_utf16(value, byte_range.end)
    }

    /// Inverse.
    pub fn range_from_utf16(value: &str, utf16_range: &Range<usize>) -> Range<usize> {
        Self::utf16_to_offset(value, utf16_range.start)
            ..Self::utf16_to_offset(value, utf16_range.end)
    }

    /// Return the substring for a UTF-16 range, plus the
    /// adjusted UTF-8 range the platform may have requested.
    pub fn text_for_range_utf16(value: &str, range_utf16: Range<usize>) -> (String, Range<usize>) {
        let start = Self::utf16_to_offset(value, range_utf16.start);
        let end = Self::utf16_to_offset(value, range_utf16.end);
        let text = value.get(start..end).unwrap_or("").to_string();
        (text, start..end)
    }

    // -- Char-boundary walking (UTF-8 safe) --------------------------

    /// `byte_offset` → previous char boundary (UTF-8 safe).
    pub fn prev_boundary(value: &str, byte_offset: usize) -> usize {
        if byte_offset == 0 {
            return 0;
        }
        let bytes = value.as_bytes();
        let mut i = byte_offset - 1;
        while i > 0 && (bytes[i] & 0b1100_0000) == 0b1000_0000 {
            i -= 1;
        }
        i
    }

    /// `byte_offset` → next char boundary (UTF-8 safe).
    pub fn next_boundary(value: &str, byte_offset: usize) -> usize {
        let len = value.len();
        if byte_offset >= len {
            return len;
        }
        let bytes = value.as_bytes();
        let mut i = byte_offset + 1;
        while i < len && (bytes[i] & 0b1100_0000) == 0b1000_0000 {
            i += 1;
        }
        i
    }

    // -- Selection / caret mutation ----------------------------------

    /// Collapse selection to a single caret position. Clamps
    /// `offset` to `value.len()`.
    pub fn move_to(&mut self, value: &str, offset: usize) {
        let clamped = offset.min(value.len());
        self.caret = clamped;
        self.selection_start = clamped;
        self.selection_end = clamped;
        self.cursor_visible = true;
    }

    /// Extend the selection to a new offset (keeps
    /// `selection_start` as the anchor and grows
    /// `selection_end`). Clamps to `value.len()`.
    pub fn select_to(&mut self, value: &str, offset: usize) {
        let clamped = offset.min(value.len());
        self.caret = clamped;
        self.selection_end = clamped;
        self.cursor_visible = true;
    }

    /// Replace the range `[start..end)` (UTF-8 bytes) with
    /// `new_text`. Updates caret and selection to land at the
    /// end of the inserted text.
    pub fn replace_text(&mut self, value: &mut String, start: usize, end: usize, new_text: &str) {
        let start = start.min(value.len());
        let end = end.max(start).min(value.len());
        value.replace_range(start..end, new_text);
        let new_caret = start + new_text.len();
        self.caret = new_caret;
        self.selection_start = new_caret;
        self.selection_end = new_caret;
        self.cursor_visible = true;
    }

    /// Apply a `Range<usize>` (UTF-8) replacement. Used by the
    /// platform IME / clipboard via `EntityInputHandler`. A
    /// commit (a non-`replace_and_mark_text_in_range` call that
    /// targets the marked range) implicitly clears the
    /// composition.
    ///
    /// `max_length`, when `Some`, truncates the inserted text
    /// so `value.len()` never exceeds the cap. Returns `true`
    /// if the value actually changed.
    pub fn replace_text_in_range_bytes(
        &mut self,
        value: &mut String,
        max_length: Option<usize>,
        range: Option<Range<usize>>,
        new_text: &str,
    ) -> bool {
        let before = value.clone();
        // Lookup order: IME-sent range → active marked range
        // → active selection. The marked range wins over the
        // selection because the caret sits at the end of the
        // marked text — falling back to the selection would
        // insert *after* the pinyin.
        let resolved = range.or_else(|| self.marked_range.clone()).or_else(|| {
            if !self.selected_range().is_empty() {
                Some(self.selected_range())
            } else {
                None
            }
        });
        // Decide the effective new text up front (honouring
        // `max_length`). This avoids the "apply, then truncate"
        // path which would leave the caret past the end.
        let effective = if let Some(cap) = max_length {
            let existing_len = match &resolved {
                Some(r) => value.len() - (r.end - r.start),
                None => value.len(),
            };
            let room = cap.saturating_sub(existing_len);
            &new_text[..new_text.len().min(room)]
        } else {
            new_text
        };
        match &resolved {
            Some(r) => self.replace_text(value, r.start, r.end, effective),
            None => self.replace_text(value, self.caret, self.caret, effective),
        }
        // A `replace_text_in_range` outside of an active
        // composition context ends any pending IME mark. The
        // platform's commit call always lands here, so this is
        // what actually clears the pinyin.
        self.marked_range = None;
        *value != before
    }

    /// Compose (IME marked text). Replaces the active selection
    /// with `new_text` and records the marked range as the
    /// span of the just-inserted text.
    pub fn replace_and_mark_text_in_range_bytes(
        &mut self,
        value: &mut String,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
    ) -> bool {
        let before = value.clone();
        // The actual range we replace follows the lookup, with
        // one extra priority: an active IME composition
        // (marked range) wins over the active selection. The
        // IME on macOS sends `replacementRange = NSNotFound`
        // (None) when updating the *existing* composition —
        // in that case the new text must replace the marked
        // range, not be inserted at the caret (the caret is
        // already at the end of the marked text, so falling
        // back to the active selection would *append* the
        // new pinyin after the existing composition and
        // produce "sas" / "sa's" artefacts).
        let range = range
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range());
        let range_start = range.start.min(value.len());
        let range_end = range.end.max(range_start).min(value.len());

        value.replace_range(range_start..range_end, new_text);
        let marked_start = range_start;
        let marked_end = range_start + new_text.len();
        self.caret = marked_end;
        self.cursor_visible = true;
        if !new_text.is_empty() {
            self.marked_range = Some(marked_start..marked_end);
        } else {
            // Empty marked text (e.g. the IME sent an empty
            // composition to clear) — no composition.
            self.marked_range = None;
        }

        // The IME also tells us the cursor selection *within*
        // the new marked text. Translate it from UTF-16 to
        // bytes and offset by the marked start. If the IME
        // didn't provide one, the caret sits at the end of
        // the marked text.
        if let Some(sel_utf16) = new_selected_range {
            let start_in_marked =
                Self::utf16_to_offset(value, sel_utf16.start).saturating_sub(marked_start);
            let end_in_marked =
                Self::utf16_to_offset(value, sel_utf16.end).saturating_sub(marked_start);
            let sel_start = (marked_start + start_in_marked).min(marked_end);
            let sel_end = (marked_start + end_in_marked).min(marked_end);
            self.selection_start = sel_start;
            self.selection_end = sel_end;
        } else {
            self.selection_start = marked_end;
            self.selection_end = marked_end;
        }
        *value != before
    }

    // -- `EntityInputHandler` body methods (not the trait impl) -----

    /// UTF-8 version of `EntityInputHandler::text_for_range`.
    pub fn text_for_range_inner(value: &str, range_utf16: Range<usize>) -> (String, Range<usize>) {
        Self::text_for_range_utf16(value, range_utf16)
    }

    /// UTF-8 version of `EntityInputHandler::selected_text_range`.
    pub fn selected_text_range_inner(&self, value: &str) -> UTF16Selection {
        let byte_range = self.selected_range();
        let start = Self::offset_to_utf16(value, byte_range.start);
        let end = Self::offset_to_utf16(value, byte_range.end);
        UTF16Selection {
            range: start..end,
            reversed: false,
        }
    }

    /// UTF-8 version of `EntityInputHandler::bounds_for_range`.
    /// Returns `None` if we haven't been laid out yet
    /// (`last_layout` is `None`).
    pub fn bounds_for_range_inner(
        &self,
        value: &str,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
    ) -> Option<Bounds<Pixels>> {
        if !self.last_line_layouts.is_empty() {
            // Multi-line path: find every row the range
            // touches and union the pixel bounds.
            let range_bytes = Self::range_from_utf16(value, &range_utf16);
            let line_height = self.last_line_height?;
            let mut min_x = Pixels::from(f32::INFINITY);
            let mut max_x = Pixels::from(f32::NEG_INFINITY);
            let mut min_y = Pixels::from(f32::INFINITY);
            let mut max_y = Pixels::from(f32::NEG_INFINITY);
            let mut any = false;
            for (i, line) in self.last_line_layouts.iter().enumerate() {
                let line_range = &self.last_line_byte_ranges[i];
                let sel_start = range_bytes.start.max(line_range.start).min(line_range.end);
                let sel_end = range_bytes.end.max(line_range.start).min(line_range.end);
                if sel_start < sel_end {
                    any = true;
                    let col_start = sel_start - line_range.start;
                    let col_end = sel_end - line_range.start;
                    let start_x = line.x_for_index(col_start);
                    let end_x = line.x_for_index(col_end);
                    let y_top = Pixels::from(f32::from(line_height) * (i as f32));
                    let y_bottom = y_top + line_height;
                    min_x = min_x.min(start_x.min(end_x));
                    max_x = max_x.max(start_x.max(end_x));
                    min_y = min_y.min(y_top);
                    max_y = max_y.max(y_bottom);
                }
            }
            if !any {
                return None;
            }
            return Some(Bounds::from_corners(
                point(
                    element_bounds.left() + min_x - self.scroll_x,
                    element_bounds.top() + min_y,
                ),
                point(
                    element_bounds.left() + max_x - self.scroll_x,
                    element_bounds.top() + max_y,
                ),
            ));
        }

        let line = self.last_layout.as_ref()?;
        let range_bytes = Self::range_from_utf16(value, &range_utf16);
        let start_x = line.x_for_index(range_bytes.start);
        let end_x = line.x_for_index(range_bytes.end);
        Some(Bounds::from_corners(
            point(
                element_bounds.left() + start_x - self.scroll_x,
                element_bounds.top(),
            ),
            point(
                element_bounds.left() + end_x - self.scroll_x,
                element_bounds.bottom(),
            ),
        ))
    }

    /// UTF-8 version of `EntityInputHandler::character_index_for_point`.
    pub fn character_index_for_point_inner(
        &self,
        value: &str,
        point: Point<Pixels>,
    ) -> Option<usize> {
        if value.is_empty() {
            return Some(0);
        }

        let bounds = self.last_bounds.as_ref()?;
        let local = bounds.localize(&point)?;

        if !self.last_line_layouts.is_empty() {
            // Multi-line: figure out the row from the Y
            // coordinate, then the column within that row.
            let line_height = self.last_line_height?;
            let row_count = self.last_line_layouts.len();
            let row = ((local.y / line_height).floor() as i64)
                .max(0)
                .min((row_count - 1) as i64) as usize;
            let line = &self.last_line_layouts[row];
            let col_bytes = line
                .index_for_x(local.x + self.scroll_x)
                .unwrap_or(line.len());
            let row_start = self.last_line_byte_ranges[row].start;
            let byte_in_value = row_start + col_bytes;
            return Some(Self::offset_to_utf16(value, byte_in_value));
        }

        let line = self.last_layout.as_ref()?;
        let utf8_index = line
            .index_for_x(local.x + self.scroll_x)
            .unwrap_or(line.len());
        Some(Self::offset_to_utf16(value, utf8_index))
    }

    // -- Action mutators (no on_change firing; caller does that) ----

    /// Left arrow. Collapses a selection to its start, otherwise
    /// moves the caret back one char.
    pub fn left(&mut self, value: &str) {
        if self.has_selection() {
            self.move_to(value, self.selected_range().start);
        } else {
            self.move_to(value, Self::prev_boundary(value, self.caret));
        }
    }

    /// Right arrow.
    pub fn right(&mut self, value: &str) {
        if self.has_selection() {
            self.move_to(value, self.selected_range().end);
        } else {
            self.move_to(value, Self::next_boundary(value, self.caret));
        }
    }

    /// Shift+Left.
    pub fn select_left(&mut self, value: &str) {
        let new_end = Self::prev_boundary(value, self.caret);
        self.select_to(value, new_end);
    }

    /// Shift+Right.
    pub fn select_right(&mut self, value: &str) {
        let new_end = Self::next_boundary(value, self.caret);
        self.select_to(value, new_end);
    }

    /// Cmd/Ctrl+A.
    pub fn select_all(&mut self, value: &str) {
        self.move_to(value, 0);
        self.select_to(value, value.len());
    }

    /// Home.
    pub fn home(&mut self) {
        self.caret = 0;
        self.selection_start = 0;
        self.selection_end = 0;
        self.cursor_visible = true;
    }

    /// End.
    pub fn end(&mut self, value: &str) {
        self.move_to(value, value.len());
    }

    /// Backspace. Returns `true` if the value changed.
    pub fn backspace(&mut self, value: &mut String) -> bool {
        let before = value.clone();
        if self.has_selection() {
            let r = self.selected_range();
            self.replace_text(value, r.start, r.end, "");
        } else if self.caret > 0 {
            let prev = Self::prev_boundary(value, self.caret);
            self.replace_text(value, prev, self.caret, "");
        }
        // A direct keyboard backspace (not via the IME
        // pipeline) cancels any active composition.
        self.marked_range = None;
        *value != before
    }

    /// Forward delete. Returns `true` if the value changed.
    pub fn delete(&mut self, value: &mut String) -> bool {
        let before = value.clone();
        if self.has_selection() {
            let r = self.selected_range();
            self.replace_text(value, r.start, r.end, "");
        } else if self.caret < value.len() {
            let next = Self::next_boundary(value, self.caret);
            self.replace_text(value, self.caret, next, "");
        }
        self.marked_range = None;
        *value != before
    }

    /// Cmd/Ctrl+V. `paste_newlines = false` collapses `\n` to
    /// space (single-line input convention). Returns `true` if
    /// the value changed.
    pub fn paste(&mut self, value: &mut String, paste_newlines: bool, cx: &mut App) -> bool {
        let Some(item) = cx.read_from_clipboard() else {
            return false;
        };
        let Some(text) = item.text() else {
            return false;
        };
        let text = if paste_newlines {
            text.to_string()
        } else {
            text.replace('\n', " ")
        };
        let before = value.clone();
        let changed = self.replace_text_in_range_bytes(value, None, None, &text);
        self.marked_range = None;
        let _ = before; // silence unused
        changed
    }

    /// Cmd/Ctrl+C.
    pub fn copy(&self, value: &str, cx: &mut App) {
        if self.has_selection() {
            let r = self.selected_range();
            let text = value[r.clone()].to_string();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
    }

    /// Cmd/Ctrl+X. Returns `true` if the value changed.
    pub fn cut(&mut self, value: &mut String, cx: &mut App) -> bool {
        if self.has_selection() {
            let r = self.selected_range();
            let text = value[r.clone()].to_string();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            let before = value.clone();
            self.replace_text(value, r.start, r.end, "");
            return *value != before;
        }
        false
    }

    /// Mouse-down: start drag-select and place the caret.
    pub fn on_mouse_down(&mut self, value: &str, position: Point<Pixels>, window: &mut Window) {
        self.is_selecting = true;
        if let Some(utf16) = self.character_index_for_point_inner(value, position) {
            let byte = Self::utf16_to_offset(value, utf16);
            self.move_to(value, byte);
        }
        window.focus(&self.focus_handle);
    }

    /// Mouse-move (while drag-selecting). Returns `true` when the
    /// drag actually moved the caret / selection, so the caller
    /// knows a repaint is needed (plain hovers return `false` to
    /// avoid a repaint per mouse-move).
    pub fn on_mouse_move(&mut self, value: &str, event: &gpui::MouseMoveEvent) -> bool {
        if self.is_selecting
            && let Some(utf16) = self.character_index_for_point_inner(value, event.position)
        {
            let byte = Self::utf16_to_offset(value, utf16);
            self.select_to(value, byte);
            return true;
        }
        false
    }
}
