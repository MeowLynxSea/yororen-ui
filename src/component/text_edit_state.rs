use std::ops::Range;

use gpui::{SharedString, UTF16Selection};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
pub struct TextEditState {
    content: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
}

impl TextEditState {
    pub fn clamp_offset(&self, offset: usize) -> usize {
        offset.min(self.content.len())
    }
}

impl Default for TextEditState {
    fn default() -> Self {
        Self {
            content: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
        }
    }
}

impl TextEditState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(&self) -> &SharedString {
        &self.content
    }

    pub fn selected_range(&self) -> &Range<usize> {
        &self.selected_range
    }

    pub fn selection_reversed(&self) -> bool {
        self.selection_reversed
    }

    pub fn marked_range(&self) -> Option<&Range<usize>> {
        self.marked_range.as_ref()
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn set_content(&mut self, content: impl Into<SharedString>) {
        let content = content.into();
        let end = content.len();
        self.content = content;
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
    }

    pub fn move_to(&mut self, offset: usize) {
        let offset = offset.clamp(0, self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
    }

    pub fn select_to(&mut self, offset: usize) {
        let offset = offset.clamp(0, self.content.len());
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
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

    pub fn selected_text_range(&self) -> UTF16Selection {
        UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        }
    }

    pub fn marked_text_range_utf16(&self) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    pub fn unmark_text(&mut self) {
        self.marked_range = None;
    }

    pub fn text_for_range_utf16(&self, range_utf16: Range<usize>) -> (String, Range<usize>) {
        let range_utf8 = self.range_from_utf16(&range_utf16);
        let clamped = clamp_range(&range_utf8, self.content.len());
        (
            self.content[clamped.clone()].to_string(),
            self.range_to_utf16(&clamped),
        )
    }

    pub fn replace_text_in_range(&mut self, range_utf16: Option<Range<usize>>, new_text: &str) {
        let range = self
            .range_for_replacement_utf8(range_utf16.as_ref())
            .unwrap_or_else(|| self.selected_range.clone());
        let (range_start, range_end) = clamp_range_bounds(&range, self.content.len());

        self.content =
            (self.content[0..range_start].to_owned() + new_text + &self.content[range_end..])
                .into();
        self.selected_range = range_start + new_text.len()..range_start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
    }

    pub fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
    ) {
        let range = self
            .range_for_replacement_utf8(range_utf16.as_ref())
            .unwrap_or_else(|| self.selected_range.clone());
        let (range_start, range_end) = clamp_range_bounds(&range, self.content.len());

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

    fn range_for_replacement_utf8(
        &self,
        range_utf16: Option<&Range<usize>>,
    ) -> Option<Range<usize>> {
        range_utf16
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or_else(|| self.marked_range.clone())
    }
}

fn clamp_range(range: &Range<usize>, len: usize) -> Range<usize> {
    let start = range.start.min(len);
    let end = range.end.min(len).max(start);
    start..end
}

fn clamp_range_bounds(range: &Range<usize>, len: usize) -> (usize, usize) {
    let start = range.start.min(len);
    let end = range.end.min(len).max(start);
    (start, end)
}
