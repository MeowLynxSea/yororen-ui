use std::sync::Arc;

use gpui::{
    App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::{IconName, TextInputState, icon, icon_button, text_input},
    theme::ActiveTheme,
};

/// Creates a new search input element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn search_input(id: impl Into<ElementId>) -> SearchInput {
    SearchInput::new().id(id)
}

type ChangeFn = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;
type SubmitFn = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(IntoElement)]
pub struct SearchInput {
    element_id: ElementId,
    base: Div,
    placeholder: SharedString,

    disabled: bool,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<ChangeFn>,
    on_submit: Option<SubmitFn>,
}

impl Default for SearchInput {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchInput {
    pub fn new() -> Self {
        Self {
            element_id: "ui:search-input".into(),
            base: div(),
            placeholder: "".into(),

            disabled: false,

            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,

            on_change: None,
            on_submit: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
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

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_submit = Some(Arc::new(handler));
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

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for SearchInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SearchInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for SearchInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for SearchInput {}

impl RenderOnce for SearchInput {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        // SearchInput requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id.clone();
        let placeholder = self.placeholder;
        let disabled = self.disabled;
        let height = self.height.unwrap_or_else(|| px(36.).into());
        let bg = self.bg;
        let border = self.border;
        let focus_border = self.focus_border;
        let text_color = self.text_color;
        let on_change = self.on_change;
        let on_submit = self.on_submit;

        let input_id: ElementId = (id.clone(), "ui:search-input:input").into();
        let clear_id: ElementId = (id.clone(), "ui:search-input:clear").into();

        let theme = cx.theme().clone();
        let hint = theme.content.tertiary;
        let action_variant = theme.action.neutral.clone();

        let input_state =
            window.use_keyed_state(input_id.clone(), cx, |_, cx| TextInputState::new(cx));
        let clear_visible = !input_state.read(cx).content().is_empty();

        let on_change_for_input = on_change.clone();
        let on_change_for_clear = on_change;

        let on_submit_for_input = on_submit.clone();

        self.base
            .id(id.clone())
            .flex()
            .items_center()
            .gap_2()
            .h(height)
            .px_3()
            .bg(bg.unwrap_or(theme.surface.base))
            .border_1()
            .border_color(border.unwrap_or(theme.border.default))
            .rounded_md()
            .when_some(focus_border, |this, focus_border| {
                this.focus_visible(|style| style.border_2().border_color(focus_border))
            })
            .when(disabled, |this| this.opacity(0.6).cursor_not_allowed())
            .child(icon(IconName::Search).size(px(14.)).color(hint))
            .child(
                div().flex_1().child(
                    text_input(input_id)
                        .placeholder(placeholder)
                        .disabled(disabled)
                        .height(height)
                        .bg(theme.surface.base.alpha(0.0))
                        .border(theme.border.default.alpha(0.0))
                        .focus_border(theme.border.default.alpha(0.0))
                        .text_color(text_color.unwrap_or(theme.content.primary))
                        .on_change({
                            let on_change = on_change_for_input;
                            move |value, window, cx| {
                                if let Some(handler) = &on_change {
                                    handler(value, window, cx);
                                }
                            }
                        })
                        .on_submit({
                            let on_submit = on_submit_for_input;
                            move |value, window, cx| {
                                if let Some(handler) = &on_submit {
                                    handler(value, window, cx);
                                }
                            }
                        }),
                ),
            )
            .child(
                div()
                    .w(px(36.))
                    .h(px(36.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(clear_visible && !disabled, |this| {
                        this.child(
                            icon_button(clear_id)
                                .icon(icon(IconName::Close))
                                .icon_size(px(12.))
                                .bg(action_variant.bg.alpha(0.0))
                                .hover_bg(action_variant.hover_bg)
                                .on_click({
                                    let input_state = input_state.clone();
                                    let on_change = on_change_for_clear;
                                    move |_ev, window, cx| {
                                        input_state.update(cx, |state, cx| {
                                            state.set_content(SharedString::new_static(""));
                                            cx.notify();
                                        });

                                        if let Some(handler) = &on_change {
                                            handler(SharedString::new_static(""), window, cx);
                                        }
                                    }
                                }),
                        )
                    }),
            )
    }
}
