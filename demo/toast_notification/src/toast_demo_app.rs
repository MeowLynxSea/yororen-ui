//! yororen-ui Toast Notification Demo Root Component
//!
//! This file demonstrates the **Toast notification patterns** in yororen-ui.
//!
//! ## Toast vs NotificationCenter
//!
//! yororen-ui provides two notification systems:
//! 1. **Toast Component** - Lightweight, static UI element for immediate display
//!    - Use for simple in-app feedback
//!    - Renders inline within your UI tree
//!    - No persistence or queuing
//!
//! 2. **NotificationCenter** - Managed notification queue with interactions
//!    - Use for important user feedback that may need acknowledgment
//!    - Notifications queued and displayed via overlay
//!    - Supports callbacks, payloads, and manual dismiss
//!
//! ## This Demo Shows
//!
//! - Toast component variants (Success, Warning, Error, Info, Neutral)
//! - Toast customization (wrapping, dimensions, custom content)
//! - NotificationCenter initialization and usage
//! - Interactive notifications with callbacks and payloads

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

/// Root component - displays Toast demo and handles notification interactions
///
/// This is the component passed to `cx.open_window()` in main().
/// It serves as the parent for all other components in the application.
pub struct ToastDemoApp;

impl ToastDemoApp {
    /// Initializes the root component
    ///
    /// For this demo, no special initialization is needed.
    /// In other apps, you might store entity_id in global state here.
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

/// Render trait - builds the UI tree for this component
///
/// This is called by gpui when:
/// - The component is first displayed
/// - `cx.notify(entity_id)` is called
/// - Global state changes that this component depends on
impl Render for ToastDemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Step 1: Read theme for styling
        let theme = cx.theme().clone();

        // Step 2: Initialize NotificationCenter if not exists
        // NotificationCenter manages queued notifications across the app.
        // It's stored as global state so any component can access it.
        if cx.try_global::<NotificationCenter>().is_none() {
            cx.set_global(NotificationCenter::new());
        }
        // Clone for use in closures (Arc-based for thread safety)
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

        // Step 3: Build UI sections

        // Section 1: Demo title
        let title = div()
            .text_xl()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.content.primary)
            .child("Toast Notification Demo");

        // Section 2: Description
        let description = div()
            .text_sm()
            .text_color(theme.content.secondary)
            .child("Toast variants and options (static display)");

        // Section 3: Toast variants - demonstrates ToastKind enum
        // ToastKind::Success  - Green styling for successful operations
        // ToastKind::Warning  - Yellow styling for warnings
        // ToastKind::Error    - Red styling for errors
        // ToastKind::Info     - Blue styling for informational messages
        // ToastKind::Neutral  - Gray styling for neutral messages
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

        // Section 4: Additional toast options - demonstrates customization
        // .wrap(true)        - Enable text wrapping for long messages
        // .max_width()      - Set maximum width before wrapping
        // .width()          - Set fixed width
        // .icon(false)      - Hide the icon
        // .content(...)     - Render custom content inside toast
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

        // Section 5: NotificationCenter actions - demonstrates interactive notifications
        // NotificationCenter queues notifications and displays them as overlay
        // Key features:
        //   - center.notify()        - Queue a simple notification
        //   - center.notify_with_callbacks() - Queue with action callbacks
        //   - .sticky(true)         - Keep visible until manually dismissed
        //   - .dismiss(DismissStrategy::Manual) - Disable auto-dismiss
        //   - .payload(...)         - Attach JSON data to notification
        //   - .action_label(...)    - Add action button text
        let actions_title = div()
            .text_lg()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.content.primary)
            .mt_6()
            .child("Notification Center (queued)");

        // Interactive button 1: Simple notification
        // Uses center.notify() for basic queued notification
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
            // Interactive button 2: Sticky notification
            // Stays visible until user explicitly dismisses it
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
            // Interactive button 3: Notification with payload and callback
            // Demonstrates:
            //   - .payload()     - Attach arbitrary JSON data
            //   - .action_label() - Add action button text
            //   - notify_with_callbacks() - Handle user interactions
            //   - First callback (Some(...)) - Called when action button clicked
            //   - Second callback (None)   - Called when notification dismissed
            .child(
                button("demo:notify:payload")
                    .child("Notify payload")
                    .on_click({
                        let center = center.clone();
                        move |_ev, _window, cx| {
                            // Clone center for use in callback closure
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
                                // on_action callback - triggered when action button clicked
                                Some(Arc::new(move |n, _ev, window, cx| {
                                    // Extract payload data
                                    let payload = n
                                        .payload
                                        .as_ref()
                                        .and_then(|v| v.get("message"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("<missing>");

                                    // Display result in new notification
                                    center_for_cb.notify(
                                        Notification::new(format!("payload.message = {payload}"))
                                            .kind(ToastKind::Success),
                                        cx,
                                    );
                                    // Refresh window to update UI
                                    window.refresh();
                                })),
                                // on_dismiss callback - triggered when notification dismissed (None here)
                                None,
                                cx,
                            );
                        }
                    }),
            );

        // Step 4: Compose all sections into final content
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

        // Step 5: Render final UI tree
        // Root div with:
        //   - size_full()      - Fill parent container
        //   .relative()       - Enable absolute positioning for overlay
        //   .bg()             - Apply theme background color
        //   notification_host() - Render notification overlay last (paints on top)
        div()
            .size_full()
            .relative()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .min_h_0()
            .child(
                // Scrollable content area
                div()
                    .flex_1()
                    .min_h_0()
                    .id("demo:scroll")
                    .overflow_scroll()
                    .child(content),
            )
            // IMPORTANT: notification_host() must be rendered last
            // This ensures notifications appear above all other content
            // The overlay is positioned absolutely and rendered on top
            .child(notification_host())
    }
}
