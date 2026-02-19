//! Text area layout module.
//!
//! Contains layout structures for text area rendering.

use gpui::{Pixels, ShapedLine, SharedString, TextRun, UnderlineStyle, px};
use std::ops::Range;

pub struct LineLayout {
    pub range: Range<usize>,
    pub shaped: ShapedLine,
    pub y: Pixels,
}

pub struct TextAreaLayout {
    pub lines: Vec<LineLayout>,
    pub line_height: Pixels,
    pub content_height: Pixels,
    pub content_width: Pixels,
}

impl TextAreaLayout {
    pub fn row_for_y(&self, y: Pixels) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }
        let row = (y / self.line_height) as usize;
        Some(row.min(self.lines.len().saturating_sub(1)))
    }

    pub fn position_for_index(&self, index: usize) -> Option<(usize, Pixels)> {
        for (row, line) in self.lines.iter().enumerate() {
            if index < line.range.start {
                continue;
            }
            if index > line.range.end {
                continue;
            }
            let idx_in_line = (index - line.range.start).min(line.shaped.len());
            return Some((row, line.shaped.x_for_index(idx_in_line)));
        }
        self.lines
            .last()
            .map(|line| (self.lines.len().saturating_sub(1), line.shaped.width))
    }
}

/// Helper function to layout text lines based on wrap mode.
pub fn layout_lines(
    display_text: &str,
    marked_range: Option<&Range<usize>>,
    base_run: &TextRun,
    font_size: Pixels,
    line_height: Pixels,
    window: &mut gpui::Window,
) -> (Vec<LineLayout>, Pixels) {
    let mut lines = Vec::new();
    let mut y = Pixels::ZERO;
    let mut max_width = Pixels::ZERO;

    let mut start = 0;
    for (i, ch) in display_text.char_indices() {
        if ch == '\n' {
            let line_text = SharedString::new(display_text[start..i].to_string());
            let runs = runs_for_line(start..i, marked_range, base_run);
            let shaped = window
                .text_system()
                .shape_line(line_text, font_size, &runs, None);
            max_width = max_width.max(shaped.width);
            lines.push(LineLayout {
                range: start..i,
                shaped,
                y,
            });
            y += line_height;
            start = i + '\n'.len_utf8();
        }
    }

    let end = display_text.len();
    if start < end || lines.is_empty() {
        let line_text = SharedString::new(display_text[start..end].to_string());
        let runs = runs_for_line(start..end, marked_range, base_run);
        let shaped = window
            .text_system()
            .shape_line(line_text, font_size, &runs, None);
        max_width = max_width.max(shaped.width);
        lines.push(LineLayout {
            range: start..end,
            shaped,
            y,
        });
    }

    (lines, max_width)
}

fn runs_for_line(
    line_range: Range<usize>,
    marked_range: Option<&Range<usize>>,
    base_run: &TextRun,
) -> Vec<TextRun> {
    let line_len = line_range.end.saturating_sub(line_range.start);
    let base = TextRun {
        len: line_len,
        ..base_run.clone()
    };

    let Some(marked_range) = marked_range else {
        return vec![base];
    };
    let marked_start = marked_range.start.clamp(line_range.start, line_range.end);
    let marked_end = marked_range.end.clamp(line_range.start, line_range.end);
    if marked_start >= marked_end {
        return vec![base];
    }

    let before_len = marked_start - line_range.start;
    let marked_len = marked_end - marked_start;
    let after_len = line_range.end - marked_end;

    [
        TextRun {
            len: before_len,
            ..base.clone()
        },
        TextRun {
            len: marked_len,
            underline: Some(UnderlineStyle {
                color: Some(base.color),
                thickness: px(1.0),
                wavy: false,
            }),
            ..base.clone()
        },
        TextRun {
            len: after_len,
            ..base
        },
    ]
    .into_iter()
    .filter(|run| run.len > 0)
    .collect()
}
