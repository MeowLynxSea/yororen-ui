use std::ops::Range;

use super::TextEditState;
use crate::theme::ActiveTheme;
use gpui::{
    AnyElement, App, Bounds, Context, CursorStyle, Div, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, Hsla, InteractiveElement,
    IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    ParentElement, Pixels, Point, RenderOnce, ShapedLine, SharedString, StatefulInteractiveElement,
    Style, Styled, TextRun, UTF16Selection, UnderlineStyle, actions, div, fill, point, prelude::*,
    px, relative, size,
};

actions!(
    ui_text_input,
    [
        Backspace,
        Delete,
        Enter,
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

type TextInputHandler = Box<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

/// Creates a new text input.
/// Use `.id()` to set a stable element ID for state management.
pub fn text_input() -> TextInput {
    TextInput::new()
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        gpui::KeyBinding::new("backspace", Backspace, Some("UITextInput")),
        gpui::KeyBinding::new("delete", Delete, Some("UITextInput")),
        gpui::KeyBinding::new("enter", Enter, Some("UITextInput")),
        gpui::KeyBinding::new("left", Left, Some("UITextInput")),
        gpui::KeyBinding::new("right", Right, Some("UITextInput")),
        gpui::KeyBinding::new("shift-left", SelectLeft, Some("UITextInput")),
        gpui::KeyBinding::new("shift-right", SelectRight, Some("UITextInput")),
        gpui::KeyBinding::new("secondary-a", SelectAll, Some("UITextInput")),
        gpui::KeyBinding::new("secondary-v", Paste, Some("UITextInput")),
        gpui::KeyBinding::new("secondary-c", Copy, Some("UITextInput")),
        gpui::KeyBinding::new("secondary-x", Cut, Some("UITextInput")),
        gpui::KeyBinding::new("home", Home, Some("UITextInput")),
        gpui::KeyBinding::new("end", End, Some("UITextInput")),
        gpui::KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UITextInput"),
        ),
    ]);
}

pub struct TextInputState {
    focus_handle: FocusHandle,
    edit: TextEditState,
    placeholder: SharedString,
    scroll_x: Pixels,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,

    cursor_visible: bool,
    cursor_blink_epoch: usize,

    focus_subscription: Option<gpui::Subscription>,
}

impl TextInputState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            edit: TextEditState::new(),
            placeholder: "".into(),
            scroll_x: Pixels::ZERO,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,

            cursor_visible: true,
            cursor_blink_epoch: 0,

            focus_subscription: None,
        }
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

    pub fn content(&self) -> &SharedString {
        self.edit.content()
    }

    pub fn set_content(&mut self, content: impl Into<SharedString>) {
        self.edit.set_content(content);
        self.scroll_x = Pixels::ZERO;
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

    fn select_left(&mut self, _: &SelectLeft, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.select_to(
            self.edit.previous_boundary(self.edit.cursor_offset()),
            window,
            cx,
        );
    }

    fn select_right(&mut self, _: &SelectRight, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.select_to(
            self.edit.next_boundary(self.edit.cursor_offset()),
            window,
            cx,
        );
    }

    fn select_all(&mut self, _: &SelectAll, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.edit.content().len(), window, cx)
    }

    fn home(&mut self, _: &Home, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    fn end(&mut self, _: &End, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.move_to(self.edit.content().len(), window, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.edit.selected_range().is_empty() {
            self.select_to(
                self.edit.previous_boundary(self.edit.cursor_offset()),
                window,
                cx,
            )
        }
        self.reset_cursor_blink(window, cx);
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if self.edit.selected_range().is_empty() {
            self.select_to(
                self.edit.next_boundary(self.edit.cursor_offset()),
                window,
                cx,
            )
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
        if !self.edit.selected_range().is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.edit.content()[self.edit.selected_range().clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.edit.selected_range().is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.edit.content()[self.edit.selected_range().clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.edit.move_to(offset);
        self.reset_cursor_blink(window, cx);
        cx.notify();
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.edit.content().is_empty() {
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
            return self.edit.content().len();
        }
        line.closest_index_for_x(position.x - bounds.left() + self.scroll_x)
    }

    fn select_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.reset_cursor_blink(window, cx);
        self.edit.select_to(offset);
        cx.notify();
    }
}

impl RenderOnce for TextInputState {
    fn render(self, _window: &mut gpui::Window, _cx: &mut App) -> impl IntoElement {
        div().child(self.edit.content().clone())
    }
}

impl EntityInputHandler for TextInputState {
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
        let last_layout = self.last_layout.as_ref()?;
        let range = self.edit.range_from_utf16(&range_utf16);
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
        if self.edit.content().is_empty() {
            return Some(0);
        }

        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        debug_assert!(last_layout.text == *self.edit.content());
        let utf8_index = last_layout
            .index_for_x(line_point.x + self.scroll_x)
            .unwrap_or_else(|| last_layout.len());
        Some(self.edit.offset_to_utf16(utf8_index))
    }
}

impl Focusable for TextInputState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct TextLineElement {
    input: Entity<TextInputState>,
    disabled: bool,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_x: Pixels,
}

impl IntoElement for TextLineElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextLineElement {
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
        let content = input.edit.content().clone();
        let placeholder = input.placeholder.clone();
        let selected_range = input.edit.selected_range().clone();
        let cursor = input.edit.cursor_offset();
        let marked_range = input.edit.marked_range().cloned();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (placeholder, cx.theme().content.tertiary)
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = marked_range.as_ref() {
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

        let cursor_pos = line.x_for_index(cursor);

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
                            size(px(2.), bounds.bottom() - bounds.top()),
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
                            bounds.left() + line.x_for_index(selected_range.start) - scroll_x,
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end) - scroll_x,
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
pub struct TextInput {
    element_id: Option<ElementId>,
    base: Div,
    placeholder: SharedString,

    disabled: bool,

    bg_color: Option<Hsla>,
    border_color: Option<Hsla>,
    focus_border_color: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    content: Option<SharedString>,

    on_change: Option<TextInputHandler>,

    on_submit: Option<TextInputHandler>,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div().h(px(36.)).px_3(),
            placeholder: "".into(),

            disabled: false,
            bg_color: None,
            border_color: None,
            focus_border_color: None,
            text_color: None,
            height: None,
            content: None,
            on_change: None,
            on_submit: None,
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

    pub fn content(mut self, content: impl Into<SharedString>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_submit = Some(Box::new(handler));
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

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TextInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for TextInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for TextInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TextInput {}

impl RenderOnce for TextInput {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        // TextInput requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID.
        let id = self.element_id.unwrap_or_else(|| {
            panic!("TextInput requires an element ID. Use `.id()` to set a stable ID.")
        });

        let disabled = self.disabled;

        let state = window.use_keyed_state(id.clone(), cx, |_, cx| TextInputState::new(cx));
        let focus_handle = state.read(cx).focus_handle.clone();
        let placeholder = self.placeholder;

        state.update(cx, |state, _cx| {
            state.placeholder = placeholder;
        });

        let content = self.content;
        let last_prop_content = window.use_keyed_state(
            (id.clone(), "ui:text-input:last-prop-content"),
            cx,
            |_, _cx| None::<SharedString>,
        );

        if let Some(prop_content) = content.clone() {
            let should_sync = last_prop_content.read(cx).as_ref() != Some(&prop_content);
            if should_sync {
                last_prop_content.update(cx, |state, _cx| {
                    *state = Some(prop_content.clone());
                });

                if state.read(cx).edit.content() != &prop_content {
                    state.update(cx, |state, _cx| {
                        state.set_content(prop_content);
                    });
                }
            }
        } else if last_prop_content.read(cx).is_some() {
            last_prop_content.update(cx, |state, _cx| {
                *state = None;
            });
        }

        let on_change = self.on_change;
        let last_content =
            window.use_keyed_state((id.clone(), "ui:text-input:last-content"), cx, |_, _cx| {
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
        let height = self.height.unwrap_or_else(|| px(36.).into());
        let inset = if disabled { px(6.) } else { px(5.) };

        let on_submit = self.on_submit;
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
            .key_context("UITextInput")
            .on_action({
                let state = state.clone();
                let on_submit = on_submit;
                move |_: &Enter, window, cx| {
                    if disabled {
                        return;
                    }

                    let content = state.read(cx).edit.content().clone();
                    if let Some(on_submit) = &on_submit {
                        on_submit(content.clone(), window, cx);
                    }
                }
            })
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
                    if disabled {
                        return;
                    }
                    state.update(cx, |state, cx| state.cut(action, window, cx));
                }
            })
            .on_action({
                let state = state.clone();
                move |action: &Copy, window, cx| {
                    if disabled {
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

        base =
            base.text_color(text_color)
                .child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .px(inset)
                        .child(div().w_full().rounded_sm().overflow_hidden().child(
                            TextLineElement {
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
