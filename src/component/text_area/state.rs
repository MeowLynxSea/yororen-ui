//! Text area state module.

use std::ops::Range;
use std::sync::Arc;

use gpui::{App, Context, FocusHandle, ParentElement, SharedString, UTF16Selection};

use crate::component::TextEditState;
use crate::constants::CURSOR_BLINK_INTERVAL;

pub type TextAreaHandler = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

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
    pub focus_handle: FocusHandle,
    pub edit: TextEditState,
    pub placeholder: SharedString,
    pub scroll_x: gpui::Pixels,
    pub scroll_y: gpui::Pixels,
    pub last_layout: Option<super::layout::TextAreaLayout>,
    pub last_bounds: Option<gpui::Bounds<gpui::Pixels>>,
    pub is_selecting: bool,
    pub cursor_visible: bool,
    pub cursor_blink_epoch: usize,
    pub focus_subscription: Option<gpui::Subscription>,
    pub preferred_x: Option<gpui::Pixels>,
    pub wrap: WrapMode,
    pub enter: EnterBehavior,
}

impl TextAreaState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            edit: TextEditState::new(),
            placeholder: "".into(),
            scroll_x: gpui::Pixels::ZERO,
            scroll_y: gpui::Pixels::ZERO,
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
        self.scroll_x = gpui::Pixels::ZERO;
        self.scroll_y = gpui::Pixels::ZERO;
        self.preferred_x = None;
    }

    pub fn scroll_x(&self) -> gpui::Pixels {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> gpui::Pixels {
        self.scroll_y
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
                    cx.background_executor().timer(CURSOR_BLINK_INTERVAL).await;

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

    pub fn move_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.edit.move_to(offset);
        self.reset_cursor_blink(window, cx);
        cx.notify();
    }

    pub fn select_to(&mut self, offset: usize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.reset_cursor_blink(window, cx);
        self.edit.select_to(offset);
        cx.notify();
    }

    pub fn left(
        &mut self,
        _: &super::actions::Left,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn right(
        &mut self,
        _: &super::actions::Right,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn up(
        &mut self,
        _: &super::actions::Up,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.move_vertically(-1, false, window, cx);
    }

    pub fn down(
        &mut self,
        _: &super::actions::Down,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.move_vertically(1, false, window, cx);
    }

    pub fn select_left(
        &mut self,
        _: &super::actions::SelectLeft,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.preferred_x = None;
        self.select_to(
            self.edit.previous_boundary(self.edit.cursor_offset()),
            window,
            cx,
        );
    }

    pub fn select_right(
        &mut self,
        _: &super::actions::SelectRight,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.preferred_x = None;
        self.select_to(
            self.edit.next_boundary(self.edit.cursor_offset()),
            window,
            cx,
        );
    }

    pub fn select_up(
        &mut self,
        _: &super::actions::SelectUp,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.move_vertically(-1, true, window, cx);
    }

    pub fn select_down(
        &mut self,
        _: &super::actions::SelectDown,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.move_vertically(1, true, window, cx);
    }

    pub fn select_all(
        &mut self,
        _: &super::actions::SelectAll,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.preferred_x = None;
        self.move_to(0, window, cx);
        self.select_to(self.edit.content().len(), window, cx)
    }

    pub fn home(
        &mut self,
        _: &super::actions::Home,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn end(
        &mut self,
        _: &super::actions::End,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn enter(
        &mut self,
        _: &super::actions::Enter,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn backspace(
        &mut self,
        _: &super::actions::Backspace,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn delete(
        &mut self,
        _: &super::actions::Delete,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn show_character_palette(
        &mut self,
        _: &super::actions::ShowCharacterPalette,
        window: &mut gpui::Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    pub fn paste(
        &mut self,
        _: &super::actions::Paste,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.preferred_x = None;
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.reset_cursor_blink(window, cx);
            self.edit.replace_text_in_range(None, &text);
            cx.notify();
        }
    }

    pub fn copy(&mut self, _: &super::actions::Copy, _: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.edit.selected_range().is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                self.edit.content()[self.edit.selected_range().clone()].to_string(),
            ));
        }
    }

    pub fn cut(
        &mut self,
        _: &super::actions::Cut,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    pub fn on_mouse_down(
        &mut self,
        event: &gpui::MouseDownEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        self.preferred_x = None;
        self.reset_cursor_blink(window, cx);
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    pub fn on_mouse_up(
        &mut self,
        _: &gpui::MouseUpEvent,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) {
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

    pub fn move_vertically(
        &mut self,
        row_delta: isize,
        selecting: bool,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.last_layout.as_ref() else {
            return;
        };
        let Some((row, x)) = layout.position_for_index(self.edit.cursor_offset()) else {
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

    pub fn index_for_mouse_position(&self, position: gpui::Point<gpui::Pixels>) -> usize {
        if self.edit.content().is_empty() {
            return 0;
        }
        let (Some(bounds), Some(layout)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        let mut local_x = position.x - bounds.left() + self.scroll_x;
        let mut local_y = position.y - bounds.top() + self.scroll_y;
        if local_y < gpui::Pixels::ZERO {
            local_y = gpui::Pixels::ZERO;
        }
        if local_x < gpui::Pixels::ZERO {
            local_x = gpui::Pixels::ZERO;
        }
        let row = layout
            .row_for_y(local_y)
            .unwrap_or_else(|| layout.lines.len().saturating_sub(1));
        let line = &layout.lines[row];
        let idx_in_line = line.shaped.closest_index_for_x(local_x);
        line.range.start + idx_in_line
    }
}

impl gpui::RenderOnce for TextAreaState {
    fn render(self, _: &mut gpui::Window, _: &mut App) -> impl gpui::IntoElement {
        gpui::div().child(self.edit.content().clone())
    }
}

impl gpui::EntityInputHandler for TextAreaState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let (text, adjusted) = self.edit.text_for_range_utf16(range_utf16);
        actual_range.replace(adjusted);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(self.edit.selected_text_range())
    }

    fn marked_text_range(
        &self,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.edit.marked_text_range_utf16()
    }

    fn unmark_text(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) {
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
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
    ) -> Option<gpui::Bounds<gpui::Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.edit.range_from_utf16(&range_utf16);
        let (row, x) = layout.position_for_index(range.start)?;
        let y = layout.lines[row].y;
        Some(gpui::Bounds::new(
            gpui::point(
                bounds.left() + x - self.scroll_x,
                bounds.top() + y - self.scroll_y,
            ),
            gpui::size(gpui::px(2.), layout.line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<gpui::Pixels>,
        _: &mut gpui::Window,
        _: &mut Context<Self>,
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

impl gpui::Focusable for TextAreaState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
