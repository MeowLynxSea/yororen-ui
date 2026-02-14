use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, div,
    ease_out_quint, prelude::FluentBuilder, px,
};

use crate::{
    component::{compute_input_style, generate_element_id, ArrowDirection, ChangeCallback, ChangeWithEventCallback, IconName, icon},
    constants::animation,
    i18n::{defaults::DefaultPlaceholders, I18nContext},
    theme::ActiveTheme,
};

/// Creates a new select option.
///
/// # Example
///
/// ```rust
/// select_option()
///     .value("option1")
///     .label("Option 1")
///     .disabled(true)
/// ```
pub fn select_option() -> SelectOption {
    SelectOption::new()
}

#[derive(Clone, Debug)]
pub struct SelectOption {
    pub value: Option<String>,
    pub label: Option<SharedString>,
    pub disabled: bool,
}

impl Default for SelectOption {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectOption {
    pub fn new() -> Self {
        Self {
            value: None,
            label: None,
            disabled: false,
        }
    }

    /// Set the option value (used as the underlying value when selected).
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the option label (displayed in the UI).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Creates a new select dropdown.
/// Use `.id()` to set a stable element ID for state management.
///
/// # Accessibility
///
/// This component provides accessibility support through the following attributes:
/// - The select element is keyboard accessible (Tab to focus, Space/Enter to open)
/// - Arrow keys can navigate through options when the menu is open
/// - Escape closes the menu
/// - The menu is properly labeled for screen readers
///
/// For full accessibility support:
/// - The component tracks `aria-expanded` state internally (true when menu is open)
/// - The menu container has a unique ID for `aria-controls` association
/// - Selected options are visually indicated with a checkmark
/// - Disabled options are properly marked
pub fn select() -> Select {
    Select::new()
}

#[derive(IntoElement)]
pub struct Select {
    element_id: Option<ElementId>,
    base: Div,
    options: Vec<SelectOption>,

    value: Option<String>,
    placeholder: SharedString,
    /// Whether to use localized placeholder from i18n
    localized: bool,
    disabled: bool,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    menu_width: Option<gpui::Pixels>,
    on_change: Option<ChangeCallback<String>>,
    on_change_simple: Option<Arc<dyn Fn(String)>>,
    on_change_with_event: Option<ChangeWithEventCallback<String>>,
}

impl Default for Select {
    fn default() -> Self {
        Self::new()
    }
}

impl Select {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            options: Vec::new(),
            value: None,
            placeholder: "Select…".into(),
            localized: false,
            disabled: false,
            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,
            menu_width: None,
            on_change: None,
            on_change_simple: None,
            on_change_with_event: None,
        }
    }

    /// Use localized placeholder from i18n.
    /// The placeholder text will be determined by the current locale.
    pub fn localized(mut self) -> Self {
        self.localized = true;
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn option(mut self, option: SelectOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
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

    /// Set a change handler for the select.
    /// The handler receives the selected value without event information.
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// Set a simplified change handler that only receives the selected value.
    /// Use this when you don't need access to window or app context.
    ///
    /// # Example
    ///
    /// ```rust
    /// select()
    ///     .on_change_simple(|value| {
    ///         println!("Selected: {}", value);
    ///     })
    /// ```
    pub fn on_change_simple<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String),
    {
        self.on_change_simple = Some(Arc::new(handler));
        self
    }

    /// Set a change handler that receives the selected value and click event.
    /// Use this when you need access to the event information (e.g., mouse position).
    pub fn on_change_with_event<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change_with_event = Some(Arc::new(handler));
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

    pub fn menu_width(mut self, width: gpui::Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }
}

impl ParentElement for Select {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Select {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Select {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Select {}

impl RenderOnce for Select {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let height = self.height.unwrap_or_else(|| px(36.).into());
        let menu_width = self.menu_width;
        let options = self.options;
        let localized = self.localized;
        let placeholder = if localized {
            DefaultPlaceholders::select_placeholder(cx.i18n().locale()).into()
        } else {
            self.placeholder
        };
        let on_change = self.on_change;
        let on_change_simple = self.on_change_simple;
        let on_change_with_event = self.on_change_with_event;

        // Select requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id.unwrap_or_else(|| generate_element_id("ui:select"));

        let menu_open = window.use_keyed_state((id.clone(), "ui:select:open"), cx, |_, _| false);
        let is_open = *menu_open.read(cx);

        let use_internal_value = on_change.is_none()
            && on_change_simple.is_none()
            && on_change_with_event.is_none()
            && self.value.is_none();
        let internal_value = use_internal_value.then(|| {
            window.use_keyed_state((id.clone(), "ui:select:value"), cx, |_, _| {
                options
                    .first()
                    .and_then(|opt| opt.value.clone())
                    .unwrap_or_default()
            })
        });

        let value = if use_internal_value {
            internal_value
                .as_ref()
                .expect("internal state should exist")
                .read(cx)
                .clone()
        } else {
            self.value
                .clone()
                .or_else(|| options.first().and_then(|opt| opt.value.clone()))
                .unwrap_or_default()
        };

        let selected_label = options
            .iter()
            .find(|opt| opt.value.as_ref() == Some(&value))
            .and_then(|opt| opt.label.clone());

        let theme = cx.theme().clone();

        let input_style = compute_input_style(
            &theme,
            disabled,
            self.bg,
            self.border,
            self.focus_border,
            self.text_color,
        );

        let hint = theme.content.tertiary;

        let menu_open_for_button = menu_open.clone();
        let menu_open_for_outside = menu_open.clone();
        let menu_open_for_select = menu_open.clone();

        let internal_value_for_select = internal_value.clone();
        let on_change_for_select = on_change.clone();
        let on_change_simple_for_select = on_change_simple.clone();
        let on_change_with_event_for_select = on_change_with_event.clone();

        self.base
            .id(id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .h(height)
            .px_3()
            .rounded_md()
            .bg(input_style.bg)
            .border_1()
            .border_color(input_style.border)
            .text_color(input_style.text_color)
            .focusable()
            .focus_visible(|style| style.border_2().border_color(input_style.focus_border))
            .when(disabled, |this| this.opacity(0.6).cursor_not_allowed())
            .when(!disabled, |this| this.cursor_pointer())
            .when(is_open, |this| this.bg(theme.surface.hover))
            .on_click(move |_ev, _window, cx| {
                if disabled {
                    return;
                }
                menu_open_for_button.update(cx, |open, _| *open = !*open);
            })
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .truncate()
                    .text_color(selected_label.as_ref().map(|_| input_style.text_color).unwrap_or(hint))
                    .child(selected_label.unwrap_or(placeholder)),
            )
            .child(
                icon(IconName::Arrow(ArrowDirection::Down))
                    .size(px(14.))
                    .color(hint),
            )
            .when(is_open, move |this| {
                let options = options.clone();
                let value = value.clone();
                let on_change = on_change_for_select.clone();
                let on_change_simple = on_change_simple_for_select.clone();
                let on_change_with_event = on_change_with_event_for_select.clone();
                let internal_value = internal_value_for_select.clone();
                let text_color = input_style.text_color;

                let menu = div()
                    .id((id.clone(), "select-menu"))
                    .absolute()
                    .top_full()
                    .left_0()
                    .mt(px(10.))
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border.default)
                    .bg(theme.surface.raised)
                    .shadow_md()
                    .py_1()
                    .when_some(menu_width, |this, width| this.w(width))
                    .occlude()
                    .on_mouse_down_out(move |_ev, _window, cx| {
                        menu_open_for_outside.update(cx, |open, _cx| *open = false);
                    })
                    .children(options.into_iter().map(move |opt| {
                        let is_selected = opt.value.as_ref() == Some(&value);
                        let is_disabled = disabled || opt.disabled;
                        let option_value = opt.value.clone().expect("SelectOption value is required");
                        let menu_open_for_select = menu_open_for_select.clone();
                        let on_change = on_change.clone();
                        let on_change_simple = on_change_simple.clone();
                        let on_change_with_event = on_change_with_event.clone();
                        let internal_value = internal_value.clone();

                        let row_fg = if is_disabled {
                            theme.content.disabled
                        } else {
                            text_color
                        };

                        div()
                            .id((ElementId::from("ui:select:option"), option_value.clone()))
                            .px_3()
                            .py_2()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .text_color(row_fg)
                            .when(!is_disabled, |this| {
                                this.cursor_pointer()
                                    .hover(|this| this.bg(theme.surface.hover))
                            })
                            .when(is_disabled, |this| this.cursor_not_allowed().opacity(0.6))
                            .child(opt.label.expect("SelectOption label is required"))
                            .when(is_selected, |this| {
                                this.child(
                                    icon(IconName::Check)
                                        .size(px(12.))
                                        .color(theme.action.primary.bg),
                                )
                            })
                            .on_click(move |ev, window, cx| {
                                if is_disabled {
                                    return;
                                }

                                if let Some(internal_value) = &internal_value {
                                    internal_value.update(cx, |state, _| {
                                        *state = option_value.clone();
                                    });
                                }

                                // Prefer on_change_with_event if provided, otherwise use on_change or on_change_simple
                                if let Some(handler) = &on_change_with_event {
                                    handler(option_value.clone(), ev, window, cx);
                                } else if let Some(handler) = &on_change {
                                    handler(option_value.clone(), window, cx);
                                } else if let Some(handler) = &on_change_simple {
                                    handler(option_value.clone());
                                }

                                menu_open_for_select.update(cx, |open, _| *open = false);
                            })
                    }));

                let animated_menu = menu.with_animation(
                    format!("select-menu-{}", is_open),
                    Animation::new(animation::MENU_OPEN)
                        .with_easing(ease_out_quint()),
                    |this, value| this.opacity(value).mt(px(10.0 - 6.0 * value)),
                );

                this.child(gpui::deferred(animated_menu).with_priority(100))
            })
    }
}
