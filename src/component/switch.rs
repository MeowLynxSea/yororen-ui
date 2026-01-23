use std::panic::Location;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::theme::ActiveTheme;

#[track_caller]
pub fn switch() -> Switch {
    Switch::new().id(ElementId::from(Location::caller()))
}

type ToggleFn = Box<dyn Fn(bool, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Switch {
    element_id: Option<ElementId>,
    base: Div,
    checked: bool,
    disabled: bool,
    on_toggle: Option<ToggleFn>,
    tone: Option<Hsla>,
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Switch {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            element_id: Some(ElementId::from(Location::caller())),
            base: div().w(px(34.)).h(px(18.)),
            checked: false,
            disabled: false,
            on_toggle: None,
            tone: None,
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

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
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

    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl ParentElement for Switch {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Switch {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Switch {}

impl RenderOnce for Switch {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let explicit_checked = self.checked;
        let on_toggle = self.on_toggle;
        let tone = self.tone;

        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let use_internal_state = on_toggle.is_none();
        let internal_checked = use_internal_state
            .then(|| window.use_keyed_state(id.clone(), cx, |_window, _cx| explicit_checked));

        let checked = if use_internal_state {
            *internal_checked
                .as_ref()
                .expect("internal state should exist")
                .read(cx)
        } else {
            explicit_checked
        };

        let theme = cx.theme();
        let accent = tone.unwrap_or_else(|| theme.action.primary.bg);

        let track_bg = if disabled {
            theme.surface.sunken
        } else if checked {
            accent
        } else {
            theme.surface.base
        };

        let track_border = if disabled {
            theme.border.muted
        } else if checked {
            accent
        } else {
            theme.border.default
        };

        let hover_bg = if checked {
            theme.action.primary.hover_bg
        } else {
            theme.surface.hover
        };

        let knob_bg = if disabled {
            theme.content.disabled
        } else if checked {
            theme.action.primary.fg
        } else {
            theme.content.primary
        };

        let mut base = self
            .base
            .id(id.clone())
            .rounded_full()
            .border_1()
            .border_color(track_border)
            .bg(track_bg)
            .p(px(2.))
            .flex()
            .items_center()
            .when(checked, |this| this.justify_end())
            .when(!checked, |this| this.justify_start())
            .focusable()
            .focus_visible(|style| style.border_2().border_color(theme.border.focus));

        if disabled {
            base = base.opacity(0.6).cursor_not_allowed();
        } else {
            base = base.cursor_pointer().hover(move |this| this.bg(hover_bg));
        }

        base.child(div().w(px(14.)).h(px(14.)).rounded_full().bg(knob_bg))
            .on_click(move |ev, window, cx| {
                if disabled {
                    return;
                }

                if use_internal_state {
                    if let Some(internal_checked) = &internal_checked {
                        internal_checked.update(cx, |value, _cx| *value = !*value);
                    }
                } else if let Some(handler) = &on_toggle {
                    handler(!explicit_checked, ev, window, cx);
                }
            })
    }
}
