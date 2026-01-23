use std::{panic::Location, path::PathBuf, sync::Arc};

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::{button, label, text_input},
    theme::{ActionVariantKind, ActiveTheme},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilePathStatus {
    Ok,
    Warning,
    Error,
}

#[track_caller]
pub fn file_path_input() -> FilePathInput {
    FilePathInput::new().id(ElementId::from(Location::caller()))
}

type ChangeFn = Arc<dyn Fn(PathBuf, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct FilePathInput {
    element_id: Option<ElementId>,
    base: Div,

    value: Option<PathBuf>,
    placeholder: SharedString,
    disabled: bool,

    status: Option<FilePathStatus>,

    bg_color: Option<Hsla>,
    border_color: Option<Hsla>,
    focus_border_color: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<ChangeFn>,
}

impl Default for FilePathInput {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePathInput {
    pub fn new() -> Self {
        Self {
            element_id: None,
            base: div(),
            value: None,
            placeholder: "选择路径…".into(),
            disabled: false,
            status: None,
            bg_color: None,
            border_color: None,
            focus_border_color: None,
            text_color: None,
            height: None,
            on_change: None,
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

    pub fn value(mut self, value: impl Into<PathBuf>) -> Self {
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

    pub fn status(mut self, status: FilePathStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(PathBuf, &mut gpui::Window, &mut gpui::App),
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
}

impl ParentElement for FilePathInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for FilePathInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for FilePathInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for FilePathInput {}

impl RenderOnce for FilePathInput {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self
            .element_id
            .unwrap_or_else(|| ElementId::from(Location::caller()));

        let disabled = self.disabled;
        let theme = cx.theme().clone();
        let height = self.height.unwrap_or_else(|| px(36.).into());

        // Always use internal state so selecting a file updates the UI without requiring external wiring.
        let initial_value = self.value.clone().unwrap_or_default();
        let value_state =
            window.use_keyed_state((id.clone(), "ui:file-path:value"), cx, |_, _| initial_value);

        let value = value_state.read(cx).clone();

        let text = SharedString::from(value.to_string_lossy().to_string());
        let showing_placeholder = value.as_os_str().is_empty();

        let base_border = if disabled {
            theme.border.muted
        } else {
            self.border_color.unwrap_or(theme.border.default)
        };

        let derived_status = if self.status.is_some() {
            self.status
        } else if showing_placeholder {
            None
        } else {
            Some(FilePathStatus::Ok)
        };

        let status_color = match derived_status {
            Some(FilePathStatus::Ok) => Some(theme.status.success.bg),
            Some(FilePathStatus::Warning) => Some(theme.status.warning.bg),
            Some(FilePathStatus::Error) => Some(theme.status.error.bg),
            None => None,
        };

        let border_color = status_color.unwrap_or(base_border);
        let focus_border_color = self.focus_border_color.unwrap_or(theme.border.focus);

        let bg = if disabled {
            theme.surface.sunken
        } else {
            self.bg_color.unwrap_or(theme.surface.base)
        };

        let text_color = if disabled {
            theme.content.disabled
        } else {
            self.text_color.unwrap_or(theme.content.primary)
        };

        let on_change = self.on_change;

        self.base
            .id(id.clone())
            .flex()
            .items_center()
            .gap_2()
            .child(
                div().flex_1().min_w(px(0.)).child(
                    text_input()
                        .id((id.clone(), "ui:file-path:input"))
                        .placeholder(self.placeholder)
                        .disabled(true)
                        .height(height)
                        .bg(bg)
                        .border(border_color)
                        .focus_border(focus_border_color)
                        .text_color(text_color)
                        .content(text)
                        .on_change(|_, _window, _cx| {}),
                ),
            )
            .child(
                button()
                    .h(px(36.))
                    .px_3()
                    .rounded_md()
                    .variant(ActionVariantKind::Neutral)
                    .disabled(disabled)
                    .child(label("选择…").inherit_color(true))
                    .on_click({
                        let value_state = value_state.clone();
                        let on_change = on_change.clone();
                        move |_ev: &ClickEvent, window, cx| {
                            if disabled {
                                return;
                            }

                            let prompt = Some(SharedString::new_static("选择路径"));
                            let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
                                files: true,
                                directories: true,
                                multiple: false,
                                prompt,
                            });

                            let value_state = value_state.clone();
                            let on_change = on_change.clone();

                            window
                                .spawn(cx, async move |cx| {
                                    let result = receiver.await;
                                    cx.update(move |window, cx| {
                                        let selected = match result {
                                            Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                            _ => None,
                                        };

                                        if let Some(path) = selected {
                                            value_state.update(cx, |state, cx| {
                                                *state = path.clone();
                                                cx.notify();
                                            });

                                            if let Some(handler) = &on_change {
                                                handler(path, window, cx);
                                            }

                                            window.refresh();
                                        }
                                    })
                                    .ok();
                                })
                                .detach();
                        }
                    }),
            )
            .when(showing_placeholder, |this| {
                this.text_color(theme.content.tertiary)
            })
    }
}
