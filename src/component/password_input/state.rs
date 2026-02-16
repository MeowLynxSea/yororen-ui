//! Password input state module.

use std::ops::Range;
use std::sync::Arc;

use gpui::{App, Context, EntityInputHandler, FocusHandle, Focusable, ParentElement, ShapedLine, SharedString, UTF16Selection};
use unicode_segmentation::UnicodeSegmentation;

use crate::constants::CURSOR_BLINK_INTERVAL;

pub type PasswordInputHandler = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

const MASK_CHAR: char = '•';

pub struct PasswordInputState {
    pub focus_handle: FocusHandle,
    pub content: SharedString,
    pub placeholder: SharedString,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<ShapedLine>,
    pub last_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    pub is_selecting: bool,

    pub cursor_visible: bool,
    pub cursor_blink_epoch: usize,

    pub focus_subscription: Option<gpui::Subscription>,
    pub scroll_x: gpui::Pixels,
}

impl PasswordInputState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,

            cursor_visible: true,
            cursor_blink_epoch: 0,

            focus_subscription: None,
            scroll_x: gpui::Pixels::ZERO,
        }
    }

    pub fn content(&self) -> &SharedString {
        &self.content
    }

    pub fn set_content(&mut self, content: impl Into<SharedString>) {
        let content = content.into();
        let end = content.len();
        self.content = content;
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        self.scroll_x = gpui::Pixels::ZERO;
    }

    pub fn show_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.cursor_visible {
            self.cursor_visible = true;
            cx.notify();
        }
    }

    pub fn reset_cursor_blink(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.show_cursor(cx);

        self.cursor_blink_epoch = self.cursor_blink_epoch.wrapping_add(1);
        let epoch = self.cursor_blink_epoch;

        let this = cx.entity().downgrade();
        window
            .spawn(cx, async move |cx| {
                loop {
                    cx.background_executor()
                        .timer(CURSOR_BLINK_INTERVAL)
                        .await;

                    let Ok(should_continue) = cx.update(|window, cx| {
                        this.update(cx, |this, cx| {
                            if this.cursor_blink_epoch != epoch {
                                return false;
                            }

                            if !this.focus_handle.is_focused(window) {
                                this.cursor_visible = true;
                                cx.notify();
                                return false;
                            }

                            this.cursor_visible = !this.cursor_visible;
                            cx.notify();
                            true
                        })
                        .unwrap_or(false)
                    }) else {
                        return;
                    };

                    if !should_continue {
                        return;
                    }
                }
            })
            .detach();
    }

    pub fn focus_in(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.focus_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            let this = cx.entity().downgrade();
            let subscription = window.on_focus_in(&focus_handle, cx, move |window, cx| {
                this.update(cx, |this, cx| this.reset_cursor_blink(window, cx))
                    .ok();
            });
            self.focus_subscription = Some(subscription);
        }

        window.focus(&self.focus_handle, cx);
        self.reset_cursor_blink(window, cx);
    }

    pub fn left(&mut self, _: &super::actions::Left, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), window, cx);
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }

    pub fn right(&mut self, _: &super::actions::Right, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), window, cx);
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }

    pub fn select_left(&mut self, _: &super::actions::SelectLeft, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), window, cx);
    }

    pub fn select_right(&mut self, _: &super::actions::SelectRight, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), window, cx);
    }

    pub fn select_all(&mut self, _: &super::actions::SelectAll, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }

    pub fn home(&mut self, _: &super::actions::Home, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    pub fn end(&mut self, _: &super::actions::End, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    pub fn backspace(&mut self, _: &super::actions::Backspace, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), window, cx)
        }
        self.reset_cursor_blink(window, cx);
        self.replace_text_in_range(None, "", window, cx)
    }

    pub fn delete(&mut self, _: &super::actions::Delete, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), window, cx)
        }
        self.reset_cursor_blink(window, cx);
        self.replace_text_in_range(None, "", window, cx)
    }

    pub fn on_mouse_down(
        &mut self,
        event: &gpui::MouseDownEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        self.reset_cursor_blink(window, cx);

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), window, cx)
        }
    }

    pub fn on_mouse_up(&mut self, _: &gpui::MouseUpEvent, _window: &mut gpui::Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    pub fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.reset_cursor_blink(window, cx);
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    pub fn show_character_palette(
        &mut self,
        _: &super::actions::ShowCharacterPalette,
        window: &mut gpui::Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    pub fn paste(&mut self, _: &super::actions::Paste, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.reset_cursor_blink(window, cx);
            self.replace_text_in_range(None, &text.replace("\n", " "), window, cx);
        }
    }

    pub fn copy(&mut self, _: &super::actions::Copy, _: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    pub fn cut(&mut self, _: &super::actions::Cut, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    pub fn move_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.reset_cursor_blink(window, cx);
        cx.notify();
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn index_for_mouse_position(&self, position: gpui::Point<gpui::Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };

        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        self.content_offset_for_display_index(
            line.closest_index_for_x(position.x - bounds.left() + self.scroll_x),
        )
    }

    pub fn select_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.reset_cursor_blink(window, cx);
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    pub fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    pub fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    pub fn grapheme_index_for_content_offset(&self, offset: usize) -> usize {
        let mut index = 0;
        for (byte_index, _) in self.content.grapheme_indices(true) {
            if byte_index >= offset {
                break;
            }
            index += 1;
        }
        index
    }

    pub fn content_offset_for_grapheme_index(&self, grapheme_index: usize) -> usize {
        for (current, (byte_index, _)) in self.content.grapheme_indices(true).enumerate() {
            if current == grapheme_index {
                return byte_index;
            }
        }
        self.content.len()
    }

    pub fn display_index_for_content_offset(&self, offset: usize) -> usize {
        self.grapheme_index_for_content_offset(offset) * MASK_CHAR.len_utf8()
    }

    pub fn content_offset_for_display_index(&self, display_offset: usize) -> usize {
        let grapheme_index = display_offset / MASK_CHAR.len_utf8();
        self.content_offset_for_grapheme_index(grapheme_index)
    }

    pub fn display_text(&self) -> SharedString {
        if self.content.is_empty() {
            return self.placeholder.clone();
        }

        let grapheme_count = self.content.graphemes(true).count();
        SharedString::from(MASK_CHAR.to_string().repeat(grapheme_count))
    }
}

impl gpui::RenderOnce for PasswordInputState {
    fn render(self, _window: &mut gpui::Window, _cx: &mut App) -> impl gpui::IntoElement {
        gpui::div().child(self.content)
    }
}

impl gpui::EntityInputHandler for PasswordInputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        if range.start > range.end || range.end > self.content.len() {
            return None;
        }
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.reset_cursor_blink(window, cx);
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let range_start = range.start.min(self.content.len());
        let range_end = range.end.min(self.content.len()).max(range_start);
        self.content =
            (self.content[0..range_start].to_owned() + new_text + &self.content[range_end..])
                .into();
        self.selected_range = range_start + new_text.len()..range_start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.reset_cursor_blink(window, cx);
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let range_start = range.start.min(self.content.len());
        let range_end = range.end.min(self.content.len()).max(range_start);
        self.content =
            (self.content[0..range_start].to_owned() + new_text + &self.content[range_end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range_start..range_start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range_start..new_range.end + range_end)
            .unwrap_or_else(|| range_start + new_text.len()..range_start + new_text.len());
        self.selection_reversed = false;

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::Bounds<gpui::Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let range = self.display_index_for_content_offset(range.start)
            ..self.display_index_for_content_offset(range.end);
        Some(gpui::Bounds::from_corners(
            gpui::point(
                bounds.left() + last_layout.x_for_index(range.start) - self.scroll_x,
                bounds.top(),
            ),
            gpui::point(
                bounds.left() + last_layout.x_for_index(range.end) - self.scroll_x,
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<gpui::Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        if self.content.is_empty() {
            return Some(0);
        }

        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        let utf8_index = last_layout
            .index_for_x(line_point.x + self.scroll_x)
            .unwrap_or_else(|| last_layout.len());
        Some(self.offset_to_utf16(self.content_offset_for_display_index(utf8_index)))
    }
}

impl Focusable for PasswordInputState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
