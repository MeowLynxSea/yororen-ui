use std::sync::Arc;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, div, px,
};

use crate::{
    component::{button, compute_input_style, text_input},
    theme::{ActionVariantKind, ActiveTheme},
};

/// Creates a new number input element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn number_input(id: impl Into<ElementId>) -> NumberInput {
    NumberInput::new().id(id)
}

type ChangeFn = Arc<dyn Fn(f64, &mut gpui::Window, &mut gpui::App)>;
type ValidateFn = Arc<dyn Fn(&str) -> bool>;

#[derive(IntoElement)]
pub struct NumberInput {
    element_id: ElementId,
    base: Div,

    value: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,

    placeholder: SharedString,
    disabled: bool,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<ChangeFn>,
    validate: Option<ValidateFn>,
}

impl Default for NumberInput {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberInput {
    pub fn new() -> Self {
        Self {
            element_id: "ui:number-input".into(),
            base: div(),
            value: None,
            min: None,
            max: None,
            step: 1.0,
            placeholder: "0".into(),
            disabled: false,
            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,
            on_change: None,
            validate: None,
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

    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        assert!(step != 0.0, "NumberInput step cannot be zero");
        self.step = step;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(f64, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// Sets a custom validation function.
    /// The function receives the raw input string and returns true if valid.
    /// Example: `.validate(|s| !s.contains('-'))` to disallow negative numbers.
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: 'static + Fn(&str) -> bool,
    {
        self.validate = Some(Arc::new(validator));
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

impl ParentElement for NumberInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for NumberInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for NumberInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for NumberInput {}

impl RenderOnce for NumberInput {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self.element_id;

        let disabled = self.disabled;
        let step = self.step;
        let min = self.min;
        let max = self.max;
        let on_change = self.on_change;
        let validate = self.validate;

        let theme = cx.theme().clone();
        let height = self.height.unwrap_or_else(|| px(36.).into());

        let input_style = compute_input_style(
            &theme,
            disabled,
            self.bg,
            self.border,
            self.focus_border,
            self.text_color,
        );

        let use_internal_value = on_change.is_none();
        let initial_value = self.value.unwrap_or(0.0);
        let internal_value = if use_internal_value {
            Some(
                window.use_keyed_state((id.clone(), format!("{}:value", id)), cx, |_, _| {
                    initial_value
                }),
            )
        } else {
            None
        };

        let value_state = if use_internal_value {
            *internal_value
                .as_ref()
                .expect("internal value should exist")
                .read(cx)
        } else {
            self.value.unwrap_or(0.0)
        };

        let value_state = clamp_f64(value_state, min, max);
        let _text = SharedString::from(format_number(value_state));

        let set_value = {
            let internal_value = internal_value.clone();
            let on_change = on_change.clone();
            move |next: f64, window: &mut gpui::Window, cx: &mut gpui::App| {
                let next = clamp_f64(next, min, max);
                if let Some(internal_value) = &internal_value {
                    internal_value.update(cx, |state, cx| {
                        *state = next;
                        cx.notify();
                    });
                }
                if let Some(handler) = &on_change {
                    handler(next, window, cx);
                }
            }
        };

        let sanitize = move |raw: &str| -> Option<f64> {
            // If custom validator is set, check it first
            if let Some(ref validator) = validate && !validator(raw) {
                return None;
            }
            raw.parse::<f64>().ok()
        };

        // Keep the input "controlled": always reflect the current numeric value.
        // This prevents non-numeric characters from staying visible in the text field.
        let controlled_text = SharedString::from(format_number(value_state));

        self.base
            .id(id.clone())
            .h(height)
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div().flex_1().min_w(px(0.)).child(
                    text_input(format!("{}:input", id))
                        .placeholder(self.placeholder)
                        .disabled(disabled)
                        .height(height)
                        .bg(input_style.bg)
                        .border(input_style.border)
                        .focus_border(input_style.focus_border)
                        .text_color(input_style.text_color)
                        .content(controlled_text)
                        .on_change({
                            let set_value = set_value.clone();
                            move |value, window, cx| {
                                if let Some(parsed) = sanitize(value.as_ref()) {
                                    set_value(parsed, window, cx);
                                }
                            }
                        }),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        button(format!("{}:decrement", id))
                            .h(px(36.))
                            .px_3()
                            .rounded_md()
                            .variant(ActionVariantKind::Neutral)
                            .disabled(disabled)
                            .child("-")
                            .on_click({
                                let internal_value = internal_value.clone();
                                let on_change = on_change.clone();
                                move |_ev: &ClickEvent, window, cx| {
                                    let current = if use_internal_value {
                                        internal_value
                                            .as_ref()
                                            .expect("internal value should exist")
                                            .read(cx)
                                            .to_owned()
                                    } else {
                                        value_state
                                    };

                                    let next = clamp_f64(current - step, min, max);
                                    if let Some(internal_value) = &internal_value {
                                        internal_value.update(cx, |state, cx| {
                                            *state = next;
                                            cx.notify();
                                        });
                                    }
                                    if let Some(handler) = &on_change {
                                        handler(next, window, cx);
                                    }
                                }
                            }),
                    )
                    .child(
                        button(format!("{}:increment", id))
                            .h(px(36.))
                            .px_3()
                            .rounded_md()
                            .variant(ActionVariantKind::Neutral)
                            .disabled(disabled)
                            .child("+")
                            .on_click({
                                let internal_value = internal_value.clone();
                                let on_change = on_change.clone();
                                move |_ev: &ClickEvent, window, cx| {
                                    let current = if use_internal_value {
                                        internal_value
                                            .as_ref()
                                            .expect("internal value should exist")
                                            .read(cx)
                                            .to_owned()
                                    } else {
                                        value_state
                                    };

                                    let next = clamp_f64(current + step, min, max);
                                    if let Some(internal_value) = &internal_value {
                                        internal_value.update(cx, |state, cx| {
                                            *state = next;
                                            cx.notify();
                                        });
                                    }
                                    if let Some(handler) = &on_change {
                                        handler(next, window, cx);
                                    }
                                }
                            }),
                    ),
            )
    }
}

fn clamp_f64(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    let value = if let Some(min) = min {
        value.max(min)
    } else {
        value
    };
    if let Some(max) = max {
        value.min(max)
    } else {
        value
    }
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() <= f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}
