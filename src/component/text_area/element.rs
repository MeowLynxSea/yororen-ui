//! Text area element module.
//!
//! Contains the element implementation for text area rendering.

use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, Entity, GlobalElementId, IntoElement,
    LayoutId, PaintQuad, Pixels, Style, TextRun, fill, point, px, relative, size,
};

use super::layout::{LineLayout, TextAreaLayout};
use super::state::{TextAreaState, WrapMode};
use crate::theme::ActiveTheme;

pub struct TextAreaElement {
    pub input: Entity<TextAreaState>,
    pub disabled: bool,
}

pub struct PrepaintState {
    layout: TextAreaLayout,
    cursor: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
    scroll_x: Pixels,
    scroll_y: Pixels,
}

impl IntoElement for TextAreaElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextAreaElement {
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
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.edit.content().clone();
        let placeholder = input.placeholder.clone();
        let selected_range = input.edit.selected_range().clone();
        let cursor = input.edit.cursor_offset();
        let marked_range = input.edit.marked_range().cloned();
        let mut scroll_x = input.scroll_x;
        let mut scroll_y = input.scroll_y;
        let wrap = input.wrap;
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (placeholder, cx.theme().content.tertiary)
        } else {
            (content, style.color)
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();

        let base_run = TextRun {
            len: 0,
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let marked_range = if display_text.is_empty() {
            None
        } else {
            marked_range
        };

        let (lines_result, max_width) = super::layout::layout_lines(
            display_text.as_str(),
            marked_range.as_ref(),
            &base_run,
            font_size,
            line_height,
            window,
        );
        let lines = lines_result;
        let y = lines
            .last()
            .map(|l| l.y + line_height)
            .unwrap_or(line_height);

        let content_height = y.max(line_height);
        let layout = TextAreaLayout {
            lines,
            line_height,
            content_height,
            content_width: max_width,
        };

        let max_scroll_y = (layout.content_height - bounds.size.height).max(Pixels::ZERO);
        scroll_y = scroll_y.clamp(Pixels::ZERO, max_scroll_y);

        let max_scroll_x = match wrap {
            WrapMode::None => (layout.content_width - bounds.size.width).max(Pixels::ZERO),
            WrapMode::Soft => Pixels::ZERO,
        };
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        let mut selection = Vec::new();
        let cursor_width = px(2.);
        let mut cursor_quad = None;
        let mut cursor_row = None;
        let mut cursor_x = Pixels::ZERO;
        let mut cursor_y = Pixels::ZERO;

        if selected_range.is_empty() {
            if let Some((row, x)) = layout.position_for_index(cursor) {
                let line = &layout.lines[row];
                cursor_row = Some(row);
                cursor_x = x;
                cursor_y = line.y;
                cursor_quad = input.cursor_visible.then(|| {
                    fill(
                        Bounds::new(
                            point(
                                bounds.left() + x - scroll_x,
                                bounds.top() + line.y - scroll_y,
                            ),
                            size(cursor_width, line_height),
                        ),
                        cx.theme().border.focus,
                    )
                });
            }
        } else {
            for (row, line) in layout.lines.iter().enumerate() {
                let start = selected_range.start.max(line.range.start);
                let end = selected_range.end.min(line.range.end);
                if start >= end {
                    continue;
                }
                let start_x = line.shaped.x_for_index(start - line.range.start);
                let end_x = line.shaped.x_for_index(end - line.range.start);
                selection.push(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + start_x - scroll_x,
                            bounds.top() + layout.lines[row].y - scroll_y,
                        ),
                        point(
                            bounds.left() + end_x - scroll_x,
                            bounds.top() + layout.lines[row].y + line_height - scroll_y,
                        ),
                    ),
                    cx.theme().border.focus.alpha(0.25),
                ));
            }
        }

        // Keep the cursor within view.
        if cursor_row.is_some() {
            let max_cursor_x = (bounds.size.width - cursor_width).max(Pixels::ZERO);
            if cursor_x < scroll_x {
                scroll_x = cursor_x;
            } else if cursor_x > scroll_x + max_cursor_x {
                scroll_x = cursor_x - max_cursor_x;
            }
            scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

            let cursor_bottom = cursor_y + line_height;
            if cursor_y < scroll_y {
                scroll_y = cursor_y;
            } else if cursor_bottom > scroll_y + bounds.size.height {
                scroll_y = (cursor_bottom - bounds.size.height).max(Pixels::ZERO);
            }
            scroll_y = scroll_y.clamp(Pixels::ZERO, max_scroll_y);
        }

        PrepaintState {
            layout,
            cursor: cursor_quad,
            selection,
            scroll_x,
            scroll_y,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        if !self.disabled {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }

        for quad in prepaint.selection.drain(..) {
            window.paint_quad(quad)
        }

        let line_height = window.line_height();
        for line in &prepaint.layout.lines {
            let y_top = bounds.top() + line.y - prepaint.scroll_y;
            let y_bottom = y_top + line_height;
            if y_bottom < bounds.top() || y_top > bounds.bottom() {
                continue;
            }

            line.shaped
                .paint(
                    point(bounds.left() - prepaint.scroll_x, y_top),
                    line_height,
                    gpui::TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .expect("paint should succeed");
        }

        if !self.disabled
            && focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        let layout = TextAreaLayout {
            lines: prepaint
                .layout
                .lines
                .iter()
                .map(|line| LineLayout {
                    range: line.range.clone(),
                    shaped: line.shaped.clone(),
                    y: line.y,
                })
                .collect(),
            line_height: prepaint.layout.line_height,
            content_height: prepaint.layout.content_height,
            content_width: prepaint.layout.content_width,
        };

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(layout);
            input.last_bounds = Some(bounds);
            input.scroll_x = prepaint.scroll_x;
            input.scroll_y = prepaint.scroll_y;
        });
    }
}
