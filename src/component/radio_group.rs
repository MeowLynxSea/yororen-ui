use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::component::{Radio, radio};

#[derive(Clone, Debug)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl RadioOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Creates a new radio group.
/// Use `.id()` to set a stable element ID for state management.
pub fn radio_group(id: impl Into<ElementId>) -> RadioGroup {
    RadioGroup::new().id(id)
}

type ChangeFn = Arc<dyn Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

type RenderOptionFn = Box<dyn Fn(&RadioOption, Radio) -> AnyElement>;

#[derive(IntoElement)]
pub struct RadioGroup {
    element_id: ElementId,
    base: Div,
    options: Vec<RadioOption>,
    value: Option<String>,
    disabled: bool,
    tone: Option<Hsla>,
    on_change: Option<ChangeFn>,
    render_option: Option<RenderOptionFn>,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RadioGroup {
    /// Creates a new radio group.
    /// Use `.id()` to set a stable element ID for state management.
    pub fn new() -> Self {
        Self {
            element_id: "ui:radio-group".into(),
            base: div(),
            options: Vec::new(),
            value: None,
            disabled: false,
            tone: None,
            on_change: None,
            render_option: None,
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

    pub fn option(mut self, option: RadioOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = RadioOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn render_option<F>(mut self, render: F) -> Self
    where
        F: 'static + Fn(&RadioOption, Radio) -> AnyElement,
    {
        self.render_option = Some(Box::new(render));
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for RadioGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for RadioGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for RadioGroup {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for RadioGroup {}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let tone = self.tone;
        let on_change = self.on_change;

        let id = self.element_id;

        let use_internal_state = on_change.is_none() && self.value.is_none();
        let internal_value = use_internal_state.then(|| {
            window.use_keyed_state(id.clone(), cx, |_window, _cx| {
                self.options
                    .first()
                    .map(|opt| opt.value.clone())
                    .unwrap_or_default()
            })
        });

        let selected = if use_internal_state {
            internal_value
                .as_ref()
                .expect("internal state should exist")
                .read(cx)
                .clone()
        } else {
            self.value
                .clone()
                .or_else(|| self.options.first().map(|opt| opt.value.clone()))
                .unwrap_or_default()
        };

        let render_option = self.render_option;
        let options = self.options;
        let group_id = id.clone();

        self.base
            .id(group_id.clone())
            .flex()
            .flex_col()
            .gap_2()
            .children(options.into_iter().map(move |option| {
                let option_disabled = disabled || option.disabled;
                let is_selected = option.value == selected;
                let radio_id = (group_id.clone(), format!("radio:{}", option.value));
                let radio = radio(radio_id)
                    .checked(is_selected)
                    .disabled(option_disabled)
                    .when_some(tone, |this, tone| this.tone(tone));

                let value = option.value.clone();
                let value_for_id = value.clone();
                let option_label = option.label.clone();
                let internal_value = internal_value.clone();
                let on_change = on_change.clone();

                let select = Rc::new(
                    move |ev: &ClickEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
                        if option_disabled {
                            return;
                        }

                        if let Some(internal_value) = &internal_value {
                            internal_value.update(cx, |state, _cx| {
                                *state = value.clone();
                            });
                        }

                        if let Some(handler) = &on_change {
                            handler(value.clone(), ev, window, cx);
                        }
                    },
                );

                let radio = radio.on_toggle({
                    let select = select.clone();
                    move |_checked, ev, window, cx| {
                        if let Some(ev) = ev {
                            select(ev, window, cx);
                        }
                    }
                });

                if let Some(render_option) = &render_option {
                    render_option(&option, radio)
                } else {
                    div()
                        .id((group_id.clone(), format!("option:{}", value_for_id)))
                        .flex()
                        .items_center()
                        .gap_2()
                        .when(!option_disabled, |this| this.cursor_pointer())
                        .when(option_disabled, |this| {
                            this.cursor_not_allowed().opacity(0.6)
                        })
                        .on_click({
                            let select = select.clone();
                            move |ev, window, cx| select(ev, window, cx)
                        })
                        .child(radio)
                        .child(option_label)
                        .into_any_element()
                }
            }))
    }
}
