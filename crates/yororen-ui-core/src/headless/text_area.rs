//! Headless `text_area` — multi-line text input.
//!
//! Reuses `TextInputState` (renderer-minted); the only
//! behavioural difference is that `Enter` inserts a newline
//! instead of firing `on_submit`.

use std::sync::Arc;

use gpui::{App, Hsla};

use super::scroll_view::ScrollViewController;

/// `TextArea` is a multi-line text input. It reuses
/// `TextInputState` for value + caret.
#[derive(Clone)]
pub struct TextAreaProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<super::text_input::TextChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
    /// Optional one-shot initial value seeded by the caller. The
    /// renderer reads this when minting the underlying
    /// `TextInputState` via `use_keyed_state`, mirroring
    /// `TextInputProps::initial_value`. The live value lives in
    /// the state entity and is updated by every keystroke — to
    /// reset at runtime, bump the `id` so a fresh state is minted.
    pub initial_value: Option<String>,
    /// External scroll controller for the vertical scroll
    /// container the renderer wraps the text area in. `None`
    /// (default) lets the renderer mint and persist one keyed by
    /// the element id — supply your own when buttons outside the
    /// area need to drive its scroll position.
    pub scroll: Option<ScrollViewController>,
}

pub fn text_area(id: impl Into<gpui::ElementId>) -> TextAreaProps {
    TextAreaProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
        initial_value: None,
        scroll: None,
    }
}

impl TextAreaProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn max_length(mut self, v: usize) -> Self {
        self.max_length = Some(v);
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_focus_border(mut self, v: bool) -> Self {
        self.has_custom_focus_border = v;
        self
    }
    pub fn custom_bg(mut self, c: Hsla) -> Self {
        self.custom_bg = Some(c);
        self.has_custom_bg = true;
        self
    }
    pub fn custom_border(mut self, c: Hsla) -> Self {
        self.custom_border = Some(c);
        self.has_custom_border = true;
        self
    }
    pub fn custom_focus_border(mut self, c: Hsla) -> Self {
        self.custom_focus_border = Some(c);
        self.has_custom_focus_border = true;
        self
    }
    pub fn custom_text_color(mut self, c: Hsla) -> Self {
        self.custom_text_color = Some(c);
        self
    }
    /// One-shot initial value, mirroring `TextInputProps::value`.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.initial_value = Some(v.into());
        self
    }

    /// Drive the vertical scroll position with an externally owned
    /// controller instead of the renderer's auto-persisted one.
    pub fn scroll(mut self, controller: &ScrollViewController) -> Self {
        self.scroll = Some(controller.clone());
        self
    }

    /// Render the text area using the registered `TextAreaRenderer`.
    ///
    /// One-way data flow: renderer's `compose` owns everything.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::TextArea as TextAreaMarker;
        use crate::renderer::text_area::TextAreaRenderer;

        let r: Arc<dyn TextAreaRenderer> = cx
            .renderer_arc::<TextAreaMarker, dyn TextAreaRenderer>()
            .expect("TextAreaRenderer registered")
            .clone();
        r.compose(&self, cx, window)
    }
}
