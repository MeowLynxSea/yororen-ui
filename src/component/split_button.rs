use std::{panic::Location, sync::Arc};

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, Styled, div, ease_out_quint,
    prelude::FluentBuilder, px,
};

use crate::component::{ArrowDirection, Icon, IconName, button};
use crate::theme::ActiveTheme;

#[track_caller]
pub fn split_button() -> SplitButton {
    SplitButton::new().id(ElementId::from(Location::caller()))
}

type ClickFn = Box<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App)>;
type ActionFn = Arc<dyn Fn(SplitButtonAction, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone, Debug)]
pub struct SplitButtonOption {
    id: String,
    label: String,
}

impl SplitButtonOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SplitButtonAction {
    Primary,
    Option(String),
}

#[derive(IntoElement)]
pub struct SplitButton {
    element_id: Option<ElementId>,
    base: Div,
    label: String,
    on_primary: Option<ClickFn>,
    on_action: Option<ActionFn>,
    options: Vec<SplitButtonOption>,
    disabled: bool,
    bg_color: Option<Hsla>,
    hover_bg: Option<Hsla>,
    menu_width: Option<Pixels>,
}

impl Default for SplitButton {
    fn default() -> Self {
        Self::new()
    }
}

impl SplitButton {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            label: "Action".to_string(),
            on_primary: None,
            on_action: None,
            options: Vec::new(),
            disabled: false,
            bg_color: None,
            hover_bg: None,
            menu_width: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = Some(id.into());
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(mut self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_primary<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_primary = Some(Box::new(handler));
        self
    }

    pub fn on_action<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SplitButtonAction, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_action = Some(Arc::new(handler));
        self
    }

    pub fn option(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(SplitButtonOption::new(id, label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SplitButtonOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.bg_color = Some(fill.into());
        self
    }

    pub fn hover_bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.hover_bg = Some(fill.into());
        self
    }

    pub fn menu_width(mut self, width: Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }
}

impl ParentElement for SplitButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SplitButton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for SplitButton {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for SplitButton {}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let on_primary = self.on_primary;
        let on_action = self.on_action;
        let options = self.options;
        let action_variant = cx.theme().action.neutral.clone();
        let bg = self.bg_color.unwrap_or(action_variant.bg);
        let hover_bg = self.hover_bg.unwrap_or(action_variant.hover_bg);
        let menu_width = self.menu_width;
        let neutral_bg = cx.theme().action.neutral.bg.alpha(0.0);
        let neutral_hover_bg = cx.theme().action.neutral.hover_bg;
        let neutral_fg = cx.theme().action.neutral.fg;
        let border_default = cx.theme().border.default;
        let border_divider = cx.theme().border.divider;
        let surface_raised = cx.theme().surface.raised;

        let id = if let Some(id) = self.element_id.clone() {
            id
        } else {
            "split-button".into()
        };

        let menu_open = window.use_keyed_state(id.clone(), cx, |_window, _cx| false);
        let is_open = *menu_open.read(cx);

        let menu_open_for_button = menu_open.clone();
        let menu_open_for_outside = menu_open.clone();
        let menu_open_for_options = menu_open.clone();
        let menu_open_for_primary = menu_open.clone();
        let on_action_for_menu = on_action.clone();

        let has_options = !options.is_empty();

        let text_color = action_variant.fg;

        self.base
            .id(id)
            .relative()
            .flex()
            .items_center()
            .rounded_lg()
            .border_1()
            .border_color(border_default)
            .bg(bg)
            .text_color(text_color)
            .when(is_open, |this| this.bg(hover_bg))
            .when(is_open && has_options, |this| {
                let on_action = on_action_for_menu.clone();
                let menu = div()
                    .id("split-button-menu")
                    .absolute()
                    .top_full()
                    .right_0()
                    .mt(px(10.))
                    .rounded_md()
                    .border_1()
                    .border_color(border_default)
                    .bg(surface_raised)
                    .shadow_md()
                    .py_1()
                    .when_some(menu_width, |this, width| this.w(width))
                    .occlude()
                    .on_mouse_down_out(move |_ev, _window, cx| {
                        menu_open_for_outside.update(cx, |open, _cx| *open = false);
                    })
                    .children(options.into_iter().map(move |option| {
                        let option_id = option.id.clone();
                        let option_label = option.label.clone();
                        let on_action = on_action.clone();
                        let menu_open_for_option = menu_open_for_options.clone();
                        button()
                            .w_full()
                            .px_3()
                            .py_2()
                            .bg(neutral_bg)
                            .hover_bg(neutral_hover_bg)
                            .text_color(neutral_fg)
                            .on_click(move |ev, window, cx| {
                                menu_open_for_option.update(cx, |open, _cx| *open = false);
                                if let Some(handler) = &on_action {
                                    handler(
                                        SplitButtonAction::Option(option_id.clone()),
                                        ev,
                                        window,
                                        cx,
                                    );
                                }
                            })
                            .child(option_label)
                    }));

                let animated_menu = menu.with_animation(
                    format!("split-button-menu-{}", is_open),
                    Animation::new(std::time::Duration::from_millis(160))
                        .with_easing(ease_out_quint()),
                    |this, value| this.opacity(value).mt(px(10.0 - 6.0 * value)),
                );

                this.child(gpui::deferred(animated_menu).with_priority(100))
            })
            .child(
                button()
                    .h(px(36.))
                    .px_4()
                    .py_2()
                    .rounded_lg()
                    .rounded_r_none()
                    .bg(neutral_bg)
                    .hover_bg(hover_bg)
                    .when(disabled, |this| this.cursor_not_allowed())
                    .on_click(move |ev, window, cx| {
                        if disabled {
                            return;
                        }
                        if let Some(handler) = &on_primary {
                            handler(ev, window, cx);
                        }
                        if let Some(handler) = &on_action {
                            handler(SplitButtonAction::Primary, ev, window, cx);
                        }
                        if has_options {
                            menu_open_for_primary.update(cx, |open, _cx| *open = false);
                        }
                    })
                    .child(self.label),
            )
            .when(has_options, |this| {
                this.child(div().w(px(1.)).h_full().bg(border_divider))
                    .child(
                        button()
                            .w(px(40.))
                            .h(px(36.))
                            .rounded_lg()
                            .rounded_l_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(neutral_bg)
                            .hover_bg(hover_bg)
                            .when(disabled, |this| this.cursor_not_allowed())
                            .on_click(move |_ev, _window, cx| {
                                if disabled {
                                    return;
                                }

                                menu_open_for_button.update(cx, |open, _cx| *open = !*open);
                                cx.stop_propagation();
                            })
                            .child(
                                Icon::new(IconName::Arrow(ArrowDirection::Down))
                                    .size(px(12.))
                                    .color(text_color),
                            ),
                    )
            })
    }
}
