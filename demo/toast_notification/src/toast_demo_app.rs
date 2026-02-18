use gpui::{Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::component::{toast, ToastKind};
use yororen_ui::theme::ActiveTheme;

pub struct ToastDemoApp;

impl ToastDemoApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ToastDemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        let title = div()
            .text_xl()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.content.primary)
            .child("Toast Notification Demo");

        let description = div()
            .text_sm()
            .text_color(theme.content.secondary)
            .child("Toast variants and options (static display)");

        // Toast variants section
        let variants_title = div()
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.content.primary)
            .mt_6()
            .child("Toast Variants");

        let variants = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                toast()
                    .message("Success: Operation completed successfully!")
                    .kind(ToastKind::Success),
            )
            .child(
                toast()
                    .message("Warning: This action cannot be undone!")
                    .kind(ToastKind::Warning),
            )
            .child(
                toast()
                    .message("Error: Failed to connect to server")
                    .kind(ToastKind::Error),
            )
            .child(
                toast()
                    .message("Info: New version available!")
                    .kind(ToastKind::Info),
            )
            .child(
                toast()
                    .message("Neutral: Operation in progress...")
                    .kind(ToastKind::Neutral),
            );

        // Additional options section
        let options_title = div()
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.content.primary)
            .mt_6()
            .child("Additional Toast Options");

        let options = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                toast()
                    .message("Long message that wraps to multiple lines when it exceeds the maximum width allowed for the toast notification.")
                    .kind(ToastKind::Info)
                    .wrap(true)
                    .max_width(px(300.)),
            )
            .child(
                toast()
                    .message("Toast without icon")
                    .kind(ToastKind::Success)
                    .icon(false),
            )
            .child(
                toast()
                    .message("Custom width toast")
                    .kind(ToastKind::Warning)
                    .width(px(200.)),
            );

        div()
            .size_full()
            .bg(theme.surface.base)
            .p(px(24.))
            .flex()
            .flex_col()
            .gap_4()
            .child(title)
            .child(description)
            .child(variants_title)
            .child(variants)
            .child(options_title)
            .child(options)
    }
}
