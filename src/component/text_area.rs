use std::{ops::Range, panic::Location};

use gpui::{
    AnyElement, App, Bounds, Context, CursorStyle, Div, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, Hsla, InteractiveElement,
    IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    ParentElement, Pixels, Point, RenderOnce, ShapedLine, SharedString, StatefulInteractiveElement,
    Style, Styled, TextRun, UTF16Selection, UnderlineStyle, actions, div, fill, point, prelude::*,
    px, relative, size,
};

use crate::{component::TextEditState, theme::ActiveTheme};

actions!(
    ui_text_area,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        Enter,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

#[track_caller]
pub fn text_area() -> TextArea {
    TextArea::new().id(ElementId::from(Location::caller()))
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        gpui::KeyBinding::new("backspace", Backspace, Some("UITextArea")),
        gpui::KeyBinding::new("delete", Delete, Some("UITextArea")),
        gpui::KeyBinding::new("left", Left, Some("UITextArea")),
        gpui::KeyBinding::new("right", Right, Some("UITextArea")),
        gpui::KeyBinding::new("up", Up, Some("UITextArea")),
        gpui::KeyBinding::new("down", Down, Some("UITextArea")),
        gpui::KeyBinding::new("shift-left", SelectLeft, Some("UITextArea")),
        gpui::KeyBinding::new("shift-right", SelectRight, Some("UITextArea")),
        gpui::KeyBinding::new("shift-up", SelectUp, Some("UITextArea")),
        gpui::KeyBinding::new("shift-down", SelectDown, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-a", SelectAll, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-v", Paste, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-c", Copy, Some("UITextArea")),
        gpui::KeyBinding::new("secondary-x", Cut, Some("UITextArea")),
        gpui::KeyBinding::new("home", Home, Some("UITextArea")),
        gpui::KeyBinding::new("end", End, Some("UITextArea")),
        gpui::KeyBinding::new("enter", Enter, Some("UITextArea")),
        gpui::KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UITextArea"),
        ),
    ]);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WrapMode {
    #[default]
    None,
    Soft,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EnterBehavior {
    #[default]
    Newline,
    Submit,
    Disabled,
}

pub struct TextAreaState {
    focus_handle: FocusHandle,
    edit: TextEditState,
    placeholder: SharedString,

    scroll_x: Pixels,
    scroll_y: Pixels,

    last_layout: Option<TextAreaLayout>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,

    cursor_visible: bool,
    cursor_blink_epoch: usize,
    focus_subscription: Option<gpui::Subscription>,

    preferred_x: Option<Pixels>,

    wrap: WrapMode,
    enter: EnterBehavior,
}

impl TextAreaState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            edit: TextEditState::new(),
            placeholder: "".into(),

            scroll_x: Pixels::ZERO,
            scroll_y: Pixels::ZERO,

            last_layout: None,
            last_bounds: None,
            is_selecting: false,

            cursor_visible: true,
            cursor_blink_epoch: 0,
            focus_subscription: None,

            preferred_x: None,

            wrap: WrapMode::None,
            enter: EnterBehavior::Newline,
        }
    }

    pub fn content(&self) -> &SharedString {
        self.edit.content()
    }

    pub fn set_content(&mut self, content: impl Into<SharedString>) {
        self.edit.set_content(content);
        self.scroll_x = Pixels::ZERO;
        self.scroll_y = Pixels::ZERO;
        self.preferred_x = None;
    }

    pub fn scroll_x(&self) -> Pixels {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> Pixels {
        self.scroll_y
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
                use std::time::Duration;

                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(500))
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

    fn move_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.edit.move_to(offset);
        self.reset_cursor_blink(window, cx);
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.reset_cursor_blink(window, cx);
        self.edit.select_to(offset);
        cx.notify();
    }

    fn left(&mut self, _: &Left, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if self.edit.selected_range().is_empty() {
            self.move_to(
                self.edit.previous_boundary(self.edit.cursor_offset()),
                window,
                cx,
            );
        } else {
            self.move_to(self.edit.selected_range().start, window, cx)
        }
    }

    fn right(&mut self, _: &Right, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if self.edit.selected_range().is_empty() {
            self.move_to(
                self.edit.next_boundary(self.edit.selected_range().end),
                window,
                cx,
            );
        } else {
            self.move_to(self.edit.selected_range().end, window, cx)
        }
    }

    fn up(&mut self, _: &Up, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_vertically(-1, false, window, cx);
    }

    fn down(&mut self, _: &Down, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_vertically(1, false, window, cx);
    }

    fn select_left(&mut self, _: &SelectLeft, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(
            self.edit.previous_boundary(self.edit.cursor_offset()),
            window,
            cx,
        );
    }

    fn select_right(&mut self, _: &SelectRight, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(
            self.edit.next_boundary(self.edit.cursor_offset()),
            window,
            cx,
        );
    }

    fn select_up(&mut self, _: &SelectUp, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_vertically(-1, true, window, cx);
    }

    fn select_down(&mut self, _: &SelectDown, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_vertically(1, true, window, cx);
    }

    fn select_all(&mut self, _: &SelectAll, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.move_to(0, window, cx);
        self.select_to(self.edit.content().len(), window, cx)
    }

    fn home(&mut self, _: &Home, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;

        let cursor = self.edit.cursor_offset();
        let Some(layout) = self.last_layout.as_ref() else {
            self.move_to(0, window, cx);
            return;
        };
        let Some((row, _x)) = layout.position_for_index(cursor) else {
            self.move_to(0, window, cx);
            return;
        };
        let line = &layout.lines[row];
        self.move_to(line.range.start, window, cx);
    }

    fn end(&mut self, _: &End, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;

        let cursor = self.edit.cursor_offset();
        let Some(layout) = self.last_layout.as_ref() else {
            self.move_to(self.edit.content().len(), window, cx);
            return;
        };
        let Some((row, _x)) = layout.position_for_index(cursor) else {
            self.move_to(self.edit.content().len(), window, cx);
            return;
        };
        let line = &layout.lines[row];
        self.move_to(line.range.end, window, cx);
    }

    fn enter(&mut self, _: &Enter, window: &mut gpui::Window, cx: &mut Context<Self>) {
        match self.enter {
            EnterBehavior::Newline => {
                self.reset_cursor_blink(window, cx);
                self.edit.replace_text_in_range(None, "\n");
                cx.notify();
            }
            EnterBehavior::Submit => {}
            EnterBehavior::Disabled => {}
        }
    }

    fn backspace(&mut self, _: &Backspace, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if self.edit.selected_range().is_empty() {
            self.select_to(
                self.edit.previous_boundary(self.edit.cursor_offset()),
                window,
                cx,
            )
        }
        self.reset_cursor_blink(window, cx);
        self.edit.replace_text_in_range(None, "");
        cx.notify();
    }

    fn delete(&mut self, _: &Delete, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if self.edit.selected_range().is_empty() {
            self.select_to(
                self.edit.next_boundary(self.edit.cursor_offset()),
                window,
                cx,
            )
        }
        self.reset_cursor_blink(window, cx);
        self.edit.replace_text_in_range(None, "");
        cx.notify();
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
        self.preferred_x = None;
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.reset_cursor_blink(window, cx);
            self.edit.replace_text_in_range(None, &text);
            cx.notify();
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.edit.selected_range().is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.edit.content()[self.edit.selected_range().clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if !self.edit.selected_range().is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.edit.content()[self.edit.selected_range().clone()].to_string(),
            ));
            self.reset_cursor_blink(window, cx);
            self.edit.replace_text_in_range(None, "");
            cx.notify();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        self.preferred_x = None;
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

    fn move_vertically(
        &mut self,
        row_delta: isize,
        selecting: bool,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.last_layout.as_ref() else {
            return;
        };

        let cursor = self.edit.cursor_offset();
        let Some((row, x)) = layout.position_for_index(cursor) else {
            return;
        };

        let target_x = self.preferred_x.get_or_insert(x);
        let target_row = (row as isize + row_delta)
            .clamp(0, layout.lines.len().saturating_sub(1) as isize)
            as usize;
        let line = &layout.lines[target_row];
        let idx_in_line = line.shaped.closest_index_for_x(*target_x);
        let target = line.range.start + idx_in_line;

        if selecting {
            self.select_to(target, window, cx);
        } else {
            self.move_to(target, window, cx);
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.edit.content().is_empty() {
            return 0;
        }

        let (Some(bounds), Some(layout)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };

        let mut local_x = position.x - bounds.left() + self.scroll_x;
        let mut local_y = position.y - bounds.top() + self.scroll_y;

        if local_y < Pixels::ZERO {
            local_y = Pixels::ZERO;
        }
        if local_x < Pixels::ZERO {
            local_x = Pixels::ZERO;
        }

        let row = layout
            .row_for_y(local_y)
            .unwrap_or_else(|| layout.lines.len().saturating_sub(1));
        let line = &layout.lines[row];
        let idx_in_line = line.shaped.closest_index_for_x(local_x);
        line.range.start + idx_in_line
    }
}

impl RenderOnce for TextAreaState {
    fn render(self, _window: &mut gpui::Window, _cx: &mut App) -> impl IntoElement {
        div().child(self.edit.content().clone())
    }
}

impl EntityInputHandler for TextAreaState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let (text, adjusted) = self.edit.text_for_range_utf16(range_utf16);
        actual_range.replace(adjusted);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(self.edit.selected_text_range())
    }

    fn marked_text_range(
        &self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.edit.marked_text_range_utf16()
    }

    fn unmark_text(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) {
        self.edit.unmark_text();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.preferred_x = None;
        self.reset_cursor_blink(window, cx);
        self.edit.replace_text_in_range(range_utf16, new_text);
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
        self.preferred_x = None;
        self.reset_cursor_blink(window, cx);
        self.edit
            .replace_and_mark_text_in_range(range_utf16, new_text, new_selected_range_utf16);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.edit.range_from_utf16(&range_utf16);
        let (row, x) = layout.position_for_index(range.start)?;
        let y = layout.lines[row].y;

        Some(Bounds::new(
            point(
                bounds.left() + x - self.scroll_x,
                bounds.top() + y - self.scroll_y,
            ),
            size(px(2.), layout.line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        if self.edit.content().is_empty() {
            return Some(0);
        }
        let layout = self.last_layout.as_ref()?;
        let bounds = self.last_bounds?;
        let local = bounds.localize(&point)?;

        let local_x = local.x + self.scroll_x;
        let local_y = local.y + self.scroll_y;

        let row = layout
            .row_for_y(local_y)
            .unwrap_or_else(|| layout.lines.len().saturating_sub(1));
        let line = &layout.lines[row];
        let idx_in_line = line
            .shaped
            .index_for_x(local_x)
            .unwrap_or_else(|| line.shaped.len());
        Some(self.edit.offset_to_utf16(line.range.start + idx_in_line))
    }
}

impl Focusable for TextAreaState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct LineLayout {
    range: Range<usize>,
    shaped: ShapedLine,
    y: Pixels,
}

struct TextAreaLayout {
    lines: Vec<LineLayout>,
    line_height: Pixels,
    content_height: Pixels,
    content_width: Pixels,
}

impl TextAreaLayout {
    fn row_for_y(&self, y: Pixels) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }
        let row = (y / self.line_height) as usize;
        Some(row.min(self.lines.len().saturating_sub(1)))
    }

    fn position_for_index(&self, index: usize) -> Option<(usize, Pixels)> {
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

struct TextAreaElement {
    input: Entity<TextAreaState>,
    disabled: bool,
}

struct PrepaintState {
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

        let mut lines = Vec::new();
        let mut y = Pixels::ZERO;
        let mut max_width = Pixels::ZERO;

        let marked_range = if display_text.is_empty() {
            None
        } else {
            marked_range
        };

        match wrap {
            WrapMode::None => {
                let mut start = 0;
                for (i, ch) in display_text.char_indices() {
                    if ch == '\n' {
                        let line_text = SharedString::new(display_text[start..i].to_string());
                        let runs = runs_for_line(&base_run, start..i, marked_range.as_ref());
                        let shaped = window.text_system().shape_line(
                            line_text.clone(),
                            font_size,
                            &runs,
                            None,
                        );
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
                let line_text = SharedString::new(display_text[start..end].to_string());
                let runs = runs_for_line(&base_run, start..end, marked_range.as_ref());
                let shaped =
                    window
                        .text_system()
                        .shape_line(line_text.clone(), font_size, &runs, None);
                max_width = max_width.max(shaped.width);
                lines.push(LineLayout {
                    range: start..end,
                    shaped,
                    y,
                });
                y += line_height;
            }
            WrapMode::Soft => {
                // Minimal implementation: only respect explicit newlines.
                // Soft wrapping by width can be layered on later.
                let mut start = 0;
                for (i, ch) in display_text.char_indices() {
                    if ch == '\n' {
                        let line_text = SharedString::new(display_text[start..i].to_string());
                        let runs = runs_for_line(&base_run, start..i, marked_range.as_ref());
                        let shaped = window.text_system().shape_line(
                            line_text.clone(),
                            font_size,
                            &runs,
                            None,
                        );
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
                let line_text = SharedString::new(display_text[start..end].to_string());
                let runs = runs_for_line(&base_run, start..end, marked_range.as_ref());
                let shaped =
                    window
                        .text_system()
                        .shape_line(line_text.clone(), font_size, &runs, None);
                max_width = max_width.max(shaped.width);
                lines.push(LineLayout {
                    range: start..end,
                    shaped,
                    y,
                });
                y += line_height;
            }
        }

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
            // Clip by bounds to avoid painting offscreen lines.
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

fn runs_for_line(
    base_run: &TextRun,
    line_range: Range<usize>,
    marked_range: Option<&Range<usize>>,
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

#[derive(IntoElement)]
pub struct TextArea {
    element_id: Option<ElementId>,
    base: Div,
    placeholder: SharedString,

    disabled: bool,
    wrap: WrapMode,
    enter: EnterBehavior,

    bg_color: Option<Hsla>,
    border_color: Option<Hsla>,
    focus_border_color: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<Box<dyn Fn(SharedString, &mut gpui::Window, &mut App)>>,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div().h(px(120.)).px_3(),
            placeholder: "".into(),

            disabled: false,
            wrap: WrapMode::None,
            enter: EnterBehavior::Newline,

            bg_color: None,
            border_color: None,
            focus_border_color: None,
            text_color: None,
            height: None,
            on_change: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn wrap(mut self, wrap: WrapMode) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn enter_behavior(mut self, enter: EnterBehavior) -> Self {
        self.enter = enter;
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
        self.bg_color = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border_color = Some(color.into());
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

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TextArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for TextArea {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for TextArea {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TextArea {}

impl RenderOnce for TextArea {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let disabled = self.disabled;
        let state = window.use_keyed_state(id.clone(), cx, |_, cx| TextAreaState::new(cx));
        let focus_handle = state.read(cx).focus_handle.clone();

        let placeholder = self.placeholder;
        let wrap = self.wrap;
        let enter = self.enter;
        state.update(cx, |state, _cx| {
            state.placeholder = placeholder;
            state.wrap = wrap;
            state.enter = enter;
        });

        let on_change = self.on_change;
        let last_content =
            window.use_keyed_state((id.clone(), "ui:text-area:last-content"), cx, |_, _cx| {
                SharedString::new_static("")
            });

        let theme = cx.theme();
        let bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg_color.unwrap_or_else(|| theme.surface.base)
        };

        let border_color = if disabled {
            theme.border.muted
        } else {
            self.border_color.unwrap_or_else(|| theme.border.default)
        };
        let focus_border_color = self
            .focus_border_color
            .unwrap_or_else(|| theme.border.focus);
        let text_color = if disabled {
            theme.content.disabled
        } else {
            self.text_color.unwrap_or_else(|| theme.content.primary)
        };
        let height = self.height.unwrap_or_else(|| px(120.).into());
        let inset = if disabled { px(6.) } else { px(5.) };

        let mut base = self
            .base
            .id(id.clone())
            .flex()
            .items_start()
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
            .key_context("UITextArea")
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Backspace, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.backspace(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Delete, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.delete(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Left, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.left(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Right, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.right(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Up, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.up(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Down, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.down(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &SelectLeft, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_left(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &SelectRight, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_right(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &SelectUp, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_up(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &SelectDown, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_down(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &SelectAll, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.select_all(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Home, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.home(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &End, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.end(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Enter, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.enter(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
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
                let disabled = disabled;
                move |action: &Paste, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.paste(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Cut, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.cut(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled;
                move |action: &Copy, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.copy(action, window, cx));
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                let disabled = disabled;
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
                let disabled = disabled;
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let state = state.clone();
                let disabled = disabled;
                move |event, window, cx| {
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_move({
                let state = state.clone();
                let disabled = disabled;
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
                div().w_full().h_full().flex().px(inset).child(
                    div()
                        .w_full()
                        .h_full()
                        .rounded_sm()
                        .id((id.clone(), "ui:text-area:scroll"))
                        .overflow_scroll()
                        .child(TextAreaElement {
                            input: state.clone(),
                            disabled,
                        }),
                ),
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
            let current = state.read(cx).edit.content().clone();
            let prev = last_content.read(cx).clone();
            if current != prev {
                last_content.update(cx, |value, _cx| *value = current.clone());
                on_change(current, window, cx);
            }
            this
        })
    }
}
