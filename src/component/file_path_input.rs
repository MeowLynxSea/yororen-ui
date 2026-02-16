use std::{path::PathBuf, sync::Arc};

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    component::{button, label, text_input},
    i18n::{defaults::DefaultPlaceholders, I18nContext},
    theme::{ActionVariantKind, ActiveTheme},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilePathStatus {
    Ok,
    Warning,
    Error,
}

/// Creates a new file path input element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn file_path_input(id: impl Into<ElementId>) -> FilePathInput {
    FilePathInput::new().id(id)
}

type ChangeFn = Arc<dyn Fn(PathBuf, &mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct FilePathInput {
    element_id: ElementId,
    base: Div,

    value: Option<PathBuf>,
    placeholder: SharedString,
    button_label: SharedString,
    dialog_prompt: SharedString,
    /// Whether to use localized placeholders from i18n
    localized: bool,
    disabled: bool,

    status: Option<FilePathStatus>,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
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
            element_id: "ui:file-path-input".into(),
            base: div(),
            value: None,
            placeholder: "Select a path…".into(),
            button_label: "Select…".into(),
            dialog_prompt: "Select a path".into(),
            localized: false,
            disabled: false,
            status: None,
            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,
            on_change: None,
        }
    }

    /// Use localized placeholders from i18n.
    /// The placeholder text will be determined by the current locale.
    pub fn localized(mut self) -> Self {
        self.localized = true;
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
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

    pub fn button_label(mut self, label: impl Into<SharedString>) -> Self {
        self.button_label = label.into();
        self
    }

    pub fn dialog_prompt(mut self, prompt: impl Into<SharedString>) -> Self {
        self.dialog_prompt = prompt.into();
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
        // Extract all values from self
        let id = self.element_id.clone();
        let localized = self.localized;
        let placeholder = if localized {
            DefaultPlaceholders::file_path_placeholder(cx.i18n().locale()).into()
        } else {
            self.placeholder
        };

        // FilePathInput requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.

        let disabled = self.disabled;
        let theme = cx.theme().clone();
        let height = self.height.unwrap_or_else(|| px(36.).into());
        let border = self.border;
        let focus_border = self.focus_border;
        let bg = self.bg;
        let text_color = self.text_color;
        let status = self.status;

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
            border.unwrap_or(theme.border.default)
        };

        let derived_status = if status.is_some() {
            status
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
        let focus_border_color = focus_border.unwrap_or(theme.border.focus);

        let bg_color = if disabled {
            theme.surface.sunken
        } else {
            bg.unwrap_or(theme.surface.base)
        };

        let text_color_value = if disabled {
            theme.content.disabled
        } else {
            text_color.unwrap_or(theme.content.primary)
        };

        let on_change = self.on_change;

        let input_id: ElementId = (id.clone(), "ui:file-path:input").into();
        let button_id: ElementId = (id.clone(), "ui:file-path:button").into();

        self.base
            .id(id.clone())
            .flex()
            .items_center()
            .gap_2()
            .child(
                div().flex_1().min_w(px(0.)).child(
                    text_input(input_id)
                        .placeholder(placeholder)
                        .disabled(true)
                        .height(height)
                        .bg(bg_color)
                        .border(border_color)
                        .focus_border(focus_border_color)
                        .text_color(text_color_value)
                        .content(text)
                        .on_change(|_, _window, _cx| {}),
                ),
            )
            .child(
                button(button_id)
                    .h(px(36.))
                    .px_3()
                    .rounded_md()
                    .variant(ActionVariantKind::Neutral)
                    .disabled(disabled)
                    .child(label(self.button_label).inherit_color(true))
                    .on_click({
                        let value_state = value_state.clone();
                        let on_change = on_change.clone();
                        let dialog_prompt = self.dialog_prompt.clone();
                        move |_ev: &ClickEvent, window, cx| {
                            if disabled {
                                return;
                            }

                            let prompt = Some(dialog_prompt.clone());
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
