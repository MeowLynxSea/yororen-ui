use std::{panic::Location, sync::Arc};

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::{ArrowDirection, IconName, icon},
    theme::ActiveTheme,
};

#[derive(Clone, Debug)]
pub struct SelectOption {
    pub value: String,
    pub label: SharedString,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<SharedString>) -> Self {
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

#[track_caller]
pub fn select() -> Select {
    Select::new().id(ElementId::from(Location::caller()))
}

type ChangeFn = Arc<dyn Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Select {
    element_id: Option<ElementId>,
    base: Div,
    options: Vec<SelectOption>,

    value: Option<String>,
    placeholder: SharedString,
    disabled: bool,

    bg_color: Option<Hsla>,
    border_color: Option<Hsla>,
    focus_border_color: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    menu_width: Option<gpui::Pixels>,
    on_change: Option<ChangeFn>,
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
            disabled: false,
            bg_color: None,
            border_color: None,
            focus_border_color: None,
            text_color: None,
            height: None,
            menu_width: None,
            on_change: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
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

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(handler));
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
        let placeholder = self.placeholder;
        let on_change = self.on_change;

        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let menu_open = window.use_keyed_state((id.clone(), "ui:select:open"), cx, |_, _| false);
        let is_open = *menu_open.read(cx);

        let use_internal_value = on_change.is_none() && self.value.is_none();
        let internal_value = use_internal_value.then(|| {
            window.use_keyed_state((id.clone(), "ui:select:value"), cx, |_, _| {
                options
                    .first()
                    .map(|opt| opt.value.clone())
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
                .or_else(|| options.first().map(|opt| opt.value.clone()))
                .unwrap_or_default()
        };

        let selected_label = options
            .iter()
            .find(|opt| opt.value == value)
            .map(|opt| opt.label.clone());

        let theme = cx.theme().clone();

        let bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg_color.unwrap_or(theme.surface.base)
        };

        let border_color = if disabled {
            theme.border.muted
        } else {
            self.border_color.unwrap_or(theme.border.default)
        };

        let focus_border_color = self.focus_border_color.unwrap_or(theme.border.focus);

        let text_color = if disabled {
            theme.content.disabled
        } else {
            self.text_color.unwrap_or(theme.content.primary)
        };

        let hint = theme.content.tertiary;

        let menu_open_for_button = menu_open.clone();
        let menu_open_for_outside = menu_open.clone();
        let menu_open_for_select = menu_open.clone();

        let internal_value_for_select = internal_value.clone();
        let on_change_for_select = on_change.clone();

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
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .text_color(text_color)
            .focusable()
            .focus_visible(|style| style.border_2().border_color(focus_border_color))
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
                    .text_color(selected_label.as_ref().map(|_| text_color).unwrap_or(hint))
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
                let internal_value = internal_value_for_select.clone();
                let text_color = text_color;

                this.child(
                    div()
                        .id("select-menu")
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
                            let is_selected = opt.value == value;
                            let is_disabled = disabled || opt.disabled;
                            let option_value = opt.value.clone();
                            let menu_open_for_select = menu_open_for_select.clone();
                            let on_change = on_change.clone();
                            let internal_value = internal_value.clone();

                            let row_fg = if is_disabled {
                                theme.content.disabled
                            } else {
                                text_color
                            };

                            div()
                                .id((
                                    ElementId::from("ui:select:option"),
                                    option_value.clone(),
                                ))
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
                                .child(opt.label)
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

                                    if let Some(handler) = &on_change {
                                        handler(option_value.clone(), ev, window, cx);
                                    }

                                    menu_open_for_select.update(cx, |open, _| *open = false);
                                })
                        })),
                )
            })
    }
}
