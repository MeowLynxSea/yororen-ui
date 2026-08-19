//! `WinUIFormFieldRenderer` — default `FormFieldRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, Div, Hsla, InteractiveElement, ParentElement, Pixels, SharedString, Stateful, Styled, div,
};

use yororen_ui_core::headless::form_field::FormFieldProps;
use yororen_ui_core::theme::{ActiveTheme, Theme};

pub use yororen_ui_core::renderer::form_field::{FormFieldRenderState, FormFieldRenderer};

pub struct WinUIFormFieldRenderer;

impl WinUIFormFieldRenderer {
    pub fn label_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or_default()
    }
    pub fn error_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("status.error.fg")
            .or_else(|| theme.get_color("status.danger.fg"))
            .unwrap_or_default()
    }
    pub fn helper_color(&self, _state: &FormFieldRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or_default()
    }
    pub fn gap(&self, _state: &FormFieldRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32)
    }
    pub fn font_size(&self, _state: &FormFieldRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_sm")
                .unwrap_or(12.0) as f32,
        )
    }
}

impl FormFieldRenderer for WinUIFormFieldRenderer {
    fn compose(&self, props: &mut FormFieldProps, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let state = FormFieldRenderState {
            has_error: props.error.is_some(),
            required: props.required,
        };
        let label_color = self.label_color(&state, theme);
        let error_color = self.error_color(&state, theme);
        let helper_color = self.helper_color(&state, theme);
        let gap = self.gap(&state, theme);
        let font_size = self.font_size(&state, theme);

        let mut wrapper = div().id(props.id.clone()).flex().flex_col().gap(gap);

        // 1. Label row (with required indicator).
        if let Some(label) = &props.label {
            let label_text: SharedString = if props.required {
                SharedString::from(format!("{} *", label))
            } else {
                label.clone()
            };
            wrapper = wrapper.child(
                div()
                    .text_size(font_size)
                    .text_color(label_color)
                    .child(label_text),
            );
        }

        // 2. Input — taken out of the props so the renderer owns it.
        if let Some(input) = props.input.take() {
            wrapper = wrapper.child(input);
        }

        // 3. Error text (above help text — errors are more urgent).
        if let Some(error) = &props.error {
            wrapper = wrapper.child(
                div()
                    .text_size(font_size)
                    .text_color(error_color)
                    .child(error.clone()),
            );
        }

        // 4. Help text.
        if let Some(help) = &props.help {
            wrapper = wrapper.child(
                div()
                    .text_size(font_size)
                    .text_color(helper_color)
                    .child(help.clone()),
            );
        }

        wrapper
    }
}

pub fn arc_form_field<T: FormFieldRenderer + 'static>(r: T) -> Arc<dyn FormFieldRenderer> {
    Arc::new(r)
}
