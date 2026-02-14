use std::ops::Range;

use gpui::{
    AnyElement, App, Bounds, Context, CursorStyle, Div, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, Hsla, InteractiveElement,
    IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    ParentElement, Pixels, Point, RenderOnce, ShapedLine, SharedString, StatefulInteractiveElement,
    Style, Styled, TextRun, UTF16Selection, UnderlineStyle, actions, div, fill, point,
    prelude::FluentBuilder, px, relative, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{component::generate_element_id, constants::CURSOR_BLINK_INTERVAL, theme::ActiveTheme};

type PasswordInputHandler = Box<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

const MASK_CHAR: char = '•';

actions!(
    ui_password_input,
    [
        Backspace,
        Delete,
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

/// Creates a new password input element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn password_input() -> PasswordInput {
    PasswordInput::new()
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        gpui::KeyBinding::new("backspace", Backspace, Some("UIPasswordInput")),
        gpui::KeyBinding::new("delete", Delete, Some("UIPasswordInput")),
        gpui::KeyBinding::new("left", Left, Some("UIPasswordInput")),
        gpui::KeyBinding::new("right", Right, Some("UIPasswordInput")),
        gpui::KeyBinding::new("shift-left", SelectLeft, Some("UIPasswordInput")),
        gpui::KeyBinding::new("shift-right", SelectRight, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-a", SelectAll, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-v", Paste, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-c", Copy, Some("UIPasswordInput")),
        gpui::KeyBinding::new("secondary-x", Cut, Some("UIPasswordInput")),
        gpui::KeyBinding::new("home", Home, Some("UIPasswordInput")),
        gpui::KeyBinding::new("end", End, Some("UIPasswordInput")),
        gpui::KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UIPasswordInput"),
        ),
    ]);
}

pub struct PasswordInputState {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,

    cursor_visible: bool,
    cursor_blink_epoch: usize,

    focus_subscription: Option<gpui::Subscription>,
    scroll_x: Pixels,
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
            scroll_x: Pixels::ZERO,
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
        self.scroll_x = Pixels::ZERO;
    }

    fn show_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.cursor_visible {
            self.cursor_visible = true;
            cx.notify();
        }
    }

    fn reset_cursor_blink(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
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

    fn focus_in(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
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

    fn left(&mut self, _: &Left, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), window, cx);
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }

    fn right(&mut self, _: &Right, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), window, cx);
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), window, cx);
    }

    fn select_right(&mut self, _: &SelectRight, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), window, cx);
    }

    fn select_all(&mut self, _: &SelectAll, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }

    fn home(&mut self, _: &Home, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    fn end(&mut self, _: &End, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), window, cx)
        }
        self.reset_cursor_blink(window, cx);
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), window, cx)
        }
        self.reset_cursor_blink(window, cx);
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
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

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut gpui::Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.reset_cursor_blink(window, cx);
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut gpui::Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &Paste, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.reset_cursor_blink(window, cx);
            self.replace_text_in_range(None, &text.replace("\n", " "), window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.reset_cursor_blink(window, cx);
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
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

    fn select_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
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

    fn offset_from_utf16(&self, offset: usize) -> usize {
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

    fn offset_to_utf16(&self, offset: usize) -> usize {
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

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn grapheme_index_for_content_offset(&self, offset: usize) -> usize {
        let mut index = 0;
        for (byte_index, _) in self.content.grapheme_indices(true) {
            if byte_index >= offset {
                break;
            }
            index += 1;
        }
        index
    }

    fn content_offset_for_grapheme_index(&self, grapheme_index: usize) -> usize {
        for (current, (byte_index, _)) in self.content.grapheme_indices(true).enumerate() {
            if current == grapheme_index {
                return byte_index;
            }
        }
        self.content.len()
    }

    fn display_index_for_content_offset(&self, offset: usize) -> usize {
        self.grapheme_index_for_content_offset(offset) * MASK_CHAR.len_utf8()
    }

    fn content_offset_for_display_index(&self, display_offset: usize) -> usize {
        let grapheme_index = display_offset / MASK_CHAR.len_utf8();
        self.content_offset_for_grapheme_index(grapheme_index)
    }

    fn display_text(&self) -> SharedString {
        if self.content.is_empty() {
            return self.placeholder.clone();
        }

        let grapheme_count = self.content.graphemes(true).count();
        SharedString::from(MASK_CHAR.to_string().repeat(grapheme_count))
    }
}

impl RenderOnce for PasswordInputState {
    fn render(self, _window: &mut gpui::Window, _cx: &mut App) -> impl IntoElement {
        div().child(self.content)
    }
}

impl EntityInputHandler for PasswordInputState {
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
        bounds: Bounds<Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let range = self.display_index_for_content_offset(range.start)
            ..self.display_index_for_content_offset(range.end);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start) - self.scroll_x,
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end) - self.scroll_x,
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
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

struct PasswordLineElement {
    input: Entity<PasswordInputState>,
    disabled: bool,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_x: Pixels,
}

impl IntoElement for PasswordLineElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for PasswordLineElement {
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
        style.size.height = window.line_height().into();
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
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let marked_range = input.marked_range.clone();
        let style = window.text_style();

        let display_text = input.display_text();
        let text_color = if content.is_empty() {
            cx.theme().content.tertiary
        } else {
            style.color
        };

        let cursor_display_index = input.display_index_for_content_offset(cursor);
        let selection_display_range = input.display_index_for_content_offset(selected_range.start)
            ..input.display_index_for_content_offset(selected_range.end);
        let marked_display_range = marked_range.as_ref().map(|range| {
            input.display_index_for_content_offset(range.start)
                ..input.display_index_for_content_offset(range.end)
        });

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = marked_display_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor_display_index);

        let cursor_width = px(2.);
        let max_cursor_x = (bounds.size.width - cursor_width).max(Pixels::ZERO);
        let max_scroll_x = (line.width - max_cursor_x).max(Pixels::ZERO);
        let mut scroll_x = input.scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        if cursor_pos < scroll_x {
            scroll_x = cursor_pos;
        } else if cursor_pos > scroll_x + max_cursor_x {
            scroll_x = cursor_pos - max_cursor_x;
        }
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                input.cursor_visible.then(|| {
                    fill(
                        Bounds::new(
                            point(bounds.left() + cursor_pos - scroll_x, bounds.top()),
                            size(cursor_width, bounds.bottom() - bounds.top()),
                        ),
                        cx.theme().border.focus,
                    )
                }),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selection_display_range.start)
                                - scroll_x,
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selection_display_range.end)
                                - scroll_x,
                            bounds.bottom(),
                        ),
                    ),
                    cx.theme().border.focus.alpha(0.25),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
            scroll_x,
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

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }
        let line = prepaint.line.take().expect("line should exist");
        line.paint(
            point(bounds.left() - prepaint.scroll_x, bounds.top()),
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        )
        .expect("paint should succeed");

        if !self.disabled
            && focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.scroll_x = prepaint.scroll_x;
        });
    }
}

#[derive(IntoElement)]
pub struct PasswordInput {
    element_id: Option<ElementId>,
    base: Div,
    placeholder: SharedString,

    disabled: bool,

    allow_copy: bool,
    allow_cut: bool,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<PasswordInputHandler>,
}

impl PasswordInput {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div().h(px(36.)).px_3(),
            placeholder: "".into(),

            disabled: false,

            allow_copy: false,
            allow_cut: false,

            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,
            on_change: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Allow copy action to write selected text into clipboard.
    ///
    /// Default: `false`.
    pub fn allow_copy(mut self, allow: bool) -> Self {
        self.allow_copy = allow;
        self
    }

    /// Allow cut action to write selected text into clipboard and delete it.
    ///
    /// Default: `false`.
    pub fn allow_cut(mut self, allow: bool) -> Self {
        self.allow_cut = allow;
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn height(mut self, height: gpui::AbsoluteLength) -> Self {
        self.height = Some(height);
        self
    }
}

impl Default for PasswordInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for PasswordInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for PasswordInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for PasswordInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for PasswordInput {}

impl RenderOnce for PasswordInput {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        let element_id = self.element_id;

        // PasswordInput requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = element_id.unwrap_or_else(|| generate_element_id("ui:password-input"));

        let disabled = self.disabled;
        let allow_copy = self.allow_copy;
        let allow_cut = self.allow_cut;

        let state = window.use_keyed_state(id.clone(), cx, |_, cx| PasswordInputState::new(cx));
        let focus_handle = state.read(cx).focus_handle.clone();
        let placeholder = self.placeholder;
        state.update(cx, |state, _cx| {
            state.placeholder = placeholder;
        });

        let on_change = self.on_change;
        let last_content = window.use_keyed_state(
            (id.clone(), "ui:password-input:last-content"),
            cx,
            |_, _cx| SharedString::new_static(""),
        );

        let theme = cx.theme();

        let bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg.unwrap_or_else(|| theme.surface.base)
        };

        let border_color = if disabled {
            theme.border.muted
        } else {
            self.border.unwrap_or_else(|| theme.border.default)
        };
        let focus_border_color = self
            .focus_border
            .unwrap_or_else(|| theme.border.focus);
        let text_color = if disabled {
            theme.content.disabled
        } else {
            self.text_color.unwrap_or_else(|| theme.content.primary)
        };
        let height = self.height.unwrap_or_else(|| px(36.).into());
        let inset = if disabled { px(6.) } else { px(5.) };

        let mut base = self
            .base
            .id(id.clone())
            .flex()
            .items_center()
            .w_full()
            .h(height)
            .rounded_md()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .when(!disabled && focus_handle.is_focused(window), |this| {
                this.border_2().border_color(focus_border_color)
            })
            .when(!disabled, |this| this.track_focus(&focus_handle))
            .when(!disabled, |this| this.cursor(CursorStyle::IBeam))
            .when(disabled, |this| this.cursor_not_allowed().opacity(0.6))
            .key_context("UIPasswordInput")
            .on_action({
                let state = state.clone();
                move |action: &Backspace, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.backspace(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Delete, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.delete(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Left, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.left(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Right, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.right(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &SelectLeft, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_left(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &SelectRight, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_right(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &SelectAll, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_all(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Home, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.home(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &End, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.end(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &ShowCharacterPalette, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| {
                        state.show_character_palette(action, window, cx)
                    });
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Paste, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.paste(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Cut, window, cx| {
                    if disabled || !allow_cut {
                        return;
                    }
                    state.update(cx, |state, cx| state.cut(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Copy, window, cx| {
                    if disabled || !allow_copy {
                        return;
                    }
                    state.update(cx, |state, cx| state.copy(action, window, cx));
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| {
                        state.focus_in(window, cx);
                        state.on_mouse_down(event, window, cx);
                    });
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_move({
                let state = state.clone();
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_move(event, window, cx));
                }
            });

        base = base
            .text_color(text_color)
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .px(inset)
                    .child(div().w_full().rounded_sm().overflow_hidden().child(
                        PasswordLineElement {
                            input: state.clone(),
                            disabled,
                        },
                    )),
            )
            .on_mouse_down_out(move |_event, window, _cx| {
                if disabled {
                    return;
                }
                if focus_handle.is_focused(window) {
                    window.blur();
                }
            });

        base.map(move |this| {
            if on_change.is_none() {
                return this;
            }

            let on_change = on_change.expect("checked");
            let current = state.read(cx).content.clone();
            let prev = last_content.read(cx).clone();
            if current != prev {
                last_content.update(cx, |value, _cx| *value = current.clone());
                on_change(current, window, cx);
            }
            this
        })
    }
}
