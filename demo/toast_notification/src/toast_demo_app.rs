use std::sync::Arc;

use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use serde_json::json;
use yororen_ui::component::{button, label, toast, ToastKind};
use yororen_ui::notification::{DismissStrategy, Notification, NotificationCenter};
use yororen_ui::notification::notification_host;
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

        // Ensure notification center exists.
        if cx.try_global::<NotificationCenter>().is_none() {
            cx.set_global(NotificationCenter::new());
        }
        let center = cx.global::<NotificationCenter>().clone();

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
            )
            .child(
                toast()
                    .kind(ToastKind::Info)
                    .wrap(true)
                    .max_width(px(300.))
                    .content(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(label("Custom content").strong(true).inherit_color(true))
                            .child(
                                label("Use Toast::content(...) to render any layout inside the toast box (e.g., title + multi-line text).")
                                    .inherit_color(true)
                                    .wrap(),
                            ),
                    ),
            );

        let actions_title = div()
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.content.primary)
            .mt_6()
            .child("Notification Center (queued)");

        let actions = div()
            .flex()
            .gap_2()
            .child(
                button("demo:notify:success")
                    .child("Notify success")
                    .on_click({
                        let center = center.clone();
                        move |_ev, _window, cx| {
                            center.notify(
                                Notification::new("Saved!").kind(ToastKind::Success),
                                cx,
                            );
                        }
                    }),
            )
            .child(
                button("demo:notify:sticky")
                    .child("Notify sticky")
                    .on_click({
                        let center = center.clone();
                        move |_ev, _window, cx| {
                            center.notify(
                                Notification::new("This persists (sticky)")
                                    .kind(ToastKind::Info)
                                    .sticky(true)
                                    .dismiss(DismissStrategy::Manual),
                                cx,
                            );
                        }
                    }),
            )
            .child(
                button("demo:notify:payload")
                    .child("Notify payload")
                    .on_click({
                        let center = center.clone();
                        move |_ev, _window, cx| {
                            let center_for_cb = center.clone();
                            center.notify_with_callbacks(
                                Notification::new("Click this toast to read payload")
                                    .kind(ToastKind::Info)
                                    .action_label("Click to try!")
                                    .payload(json!({
                                        "kind": "demo",
                                        "id": 42,
                                        "message": "hello from payload"
                                    }))
                                    .dismiss(DismissStrategy::Manual),
                                Some(Arc::new(move |n, _ev, window, cx| {
                                    let payload = n
                                        .payload
                                        .as_ref()
                                        .and_then(|v| v.get("message"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("<missing>");

                                    center_for_cb.notify(
                                        Notification::new(format!("payload.message = {payload}"))
                                            .kind(ToastKind::Success),
                                        cx,
                                    );
                                    window.refresh();
                                })),
                                None,
                                cx,
                            );
                        }
                    }),
            );

        let content = div()
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
            .child(actions_title)
            .child(actions);

        div()
            .size_full()
            .relative()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .min_h_0()
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .id("demo:scroll")
                    .overflow_scroll()
                    .child(content),
            )
            // Render overlay host last so it paints above.
            .child(notification_host())
    }
}
