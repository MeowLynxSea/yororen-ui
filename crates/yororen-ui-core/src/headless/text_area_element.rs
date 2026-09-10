//! `TextAreaElement` — the inner painter for the multi-line
//! `TextArea` headless component.
//!
//! Lives in headless (not the renderer) for the same reason as
//! `TextInputElement`: it carries no theme knowledge. Splits the
//! value by `'\n'`, shapes each line independently (gpui's
//! `shape_line` rejects strings containing `'\n'`), paints each
//! row at its own Y, renders one selection quad per row the
//! selection spans, and registers the IME pipeline against the
//! `TextInputState`.

use std::ops::Range;

use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, FocusHandle, GlobalElementId, Hsla,
    IntoElement, LayoutId, PaintQuad, Pixels, ShapedLine, SharedString, Style, TextRun, Window,
    fill, point, px, relative, size,
};

use crate::headless::text_input::TextInputState;

/// The inner multi-line painter. Embedded by `headless::TextAreaProps::render`.
pub struct TextAreaElement {
    pub state: gpui::Entity<TextInputState>,
    pub focus_handle: FocusHandle,
    pub disabled: bool,
    pub text_color: Hsla,
    pub hint_color: Hsla,
    pub cursor_color: Hsla,
    pub selection_color: Hsla,
    pub placeholder: SharedString,
    /// Minimum visible height (the wrapper sets this so the text
    /// area is tappable even when empty). The actual height is
    /// `max(min_h, line_count * line_height)`.
    pub min_h: Pixels,
}

pub struct TextAreaPrepaintState {
    pub lines: Vec<ShapedLine>,
    /// Byte range in `value` for each row in `lines`. Length
    /// matches `lines`. The i-th range is
    /// `line_i_start..line_i_end`; non-final rows include the
    /// trailing `'\n'` in the range (so the next row's start is
    /// `range.end`).
    pub line_byte_ranges: Vec<Range<usize>>,
    pub selection_quads: Vec<PaintQuad>,
    pub cursor: Option<PaintQuad>,
    pub scroll_x: Pixels,
    pub cursor_visible: bool,
}

impl IntoElement for TextAreaElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = TextAreaPrepaintState;

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
        let value = self.state.read(cx).value.clone();
        let line_count = if value.is_empty() {
            1
        } else {
            value.split('\n').count()
        };
        let line_height_px = window.line_height();
        let total_height = line_height_px * line_count as f32;
        let height = total_height.max(self.min_h);

        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = height.into();
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
        let value = input.value.clone();
        let caret = input.caret;
        let selection = input.selected_range();
        let scroll_x_input = input.scroll_x;
        let cursor_visible = input.cursor_visible;

        let is_empty = value.is_empty();
        let placeholder = input.placeholder.clone();
        let display_text: SharedString = if is_empty {
            placeholder
        } else {
            SharedString::from(value.clone())
        };

        let line_strs: Vec<String> = if is_empty {
            vec![String::new()]
        } else {
            display_text.split('\n').map(String::from).collect()
        };

        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let run_color = if is_empty {
            self.hint_color
        } else {
            self.text_color
        };

        let mut shaped: Vec<ShapedLine> = Vec::with_capacity(line_strs.len());
        for line_str in &line_strs {
            let run = TextRun {
                len: line_str.len(),
                font: style.font(),
                color: run_color,
                background_color: None,
                underline: None,
                strikethrough: None,
                letter_spacing: None,
            };
            let shaped_line =
                window
                    .text_system()
                    .shape_line(line_str.clone().into(), font_size, &[run], None);
            shaped.push(shaped_line);
        }

        let line_byte_ranges: Vec<Range<usize>> = {
            let mut ranges = Vec::with_capacity(line_strs.len());
            let mut offset = 0usize;
            for (i, line_str) in line_strs.iter().enumerate() {
                let end = if i + 1 < line_strs.len() {
                    offset + line_str.len() + 1
                } else {
                    offset + line_str.len()
                };
                ranges.push(offset..end);
                offset = end;
            }
            ranges
        };

        let caret_row = line_byte_ranges
            .iter()
            .position(|r| r.contains(&caret))
            .unwrap_or(line_byte_ranges.len().saturating_sub(1))
            .min(shaped.len().saturating_sub(1));
        let caret_col = caret.saturating_sub(line_byte_ranges[caret_row].start);

        let line_height_px = window.line_height();
        let caret_x_line = shaped[caret_row].x_for_index(caret_col);
        let cursor_thickness = px(2.0);
        let max_cursor_x = (bounds.size.width - cursor_thickness).max(Pixels::ZERO);
        let max_line_width = shaped
            .iter()
            .map(|l| l.width)
            .fold(Pixels::ZERO, |a, b| a.max(b));
        let max_scroll_x = (max_line_width - max_cursor_x).max(Pixels::ZERO);
        let mut scroll_x = scroll_x_input.clamp(Pixels::ZERO, max_scroll_x);
        if caret_x_line < scroll_x {
            scroll_x = caret_x_line;
        } else if caret_x_line > scroll_x + max_cursor_x {
            scroll_x = caret_x_line - max_cursor_x;
        }
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        let mut selection_quads = Vec::new();
        if !selection.is_empty() && !is_empty {
            for (i, line) in shaped.iter().enumerate() {
                let range = &line_byte_ranges[i];
                let sel_start = selection.start.max(range.start).min(range.end);
                let sel_end = selection.end.max(range.start).min(range.end);
                if sel_start < sel_end {
                    let col_start = sel_start - range.start;
                    let col_end = sel_end - range.start;
                    let start_x = line.x_for_index(col_start);
                    let end_x = line.x_for_index(col_end);
                    let y_top = bounds.top() + (i as f32) * line_height_px;
                    let y_bottom = y_top + line_height_px;
                    let quad = fill(
                        Bounds::from_corners(
                            point(bounds.left() + start_x.min(end_x) - scroll_x, y_top),
                            point(bounds.left() + start_x.max(end_x) - scroll_x, y_bottom),
                        ),
                        self.selection_color,
                    );
                    selection_quads.push(quad);
                }
            }
        }

        let cursor = if selection.is_empty() && !is_empty {
            let y_top = bounds.top() + (caret_row as f32) * line_height_px;
            let y_bottom = y_top + line_height_px;
            let cursor_paint_x = bounds.left() + caret_x_line - scroll_x;
            Some(fill(
                Bounds::new(
                    point(cursor_paint_x, y_top),
                    size(cursor_thickness, y_bottom - y_top),
                ),
                self.cursor_color,
            ))
        } else {
            None
        };

        TextAreaPrepaintState {
            lines: shaped,
            line_byte_ranges,
            selection_quads,
            cursor,
            scroll_x,
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

        let selection_quads = std::mem::take(&mut prepaint.selection_quads);
        for quad in selection_quads {
            window.paint_quad(quad);
        }

        let line_height_px = window.line_height();
        let lines: Vec<ShapedLine> = std::mem::take(&mut prepaint.lines);
        let line_byte_ranges: Vec<Range<usize>> = std::mem::take(&mut prepaint.line_byte_ranges);
        for (i, line) in lines.iter().enumerate() {
            let y_offset = bounds.top() + (i as f32) * line_height_px;
            let origin_x = bounds.left() - prepaint.scroll_x;
            let _ = line.paint(
                point(origin_x, y_offset),
                line_height_px,
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            );
        }

        let is_focused = self.focus_handle.is_focused(window);
        if is_focused
            && prepaint.cursor_visible
            && let Some(cur) = prepaint.cursor.take()
        {
            window.paint_quad(cur);
        }

        self.state.update(cx, |input, _cx| {
            input.last_layout = lines.first().cloned();
            input.last_line_layouts = lines;
            input.last_line_byte_ranges = line_byte_ranges;
            input.last_line_height = Some(line_height_px);
            input.last_bounds = Some(bounds);
            input.scroll_x = prepaint.scroll_x;
        });
    }
}
