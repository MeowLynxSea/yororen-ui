//! yororen-ui Counter Component
//!
//! This is the **simplest** yororen-ui component demonstrating:
//! - Reading global state
//! - Handling button click events
//! - Triggering re-renders after state changes
//!
//! ## Core Pattern
//!
//! ```ignore
//! 1. Read state: let count = *state.counter.lock().unwrap();
//! 2. Modify state: *state.counter.lock().unwrap() = new_value;
//! 3. Notify: cx.notify(entity_id);  // CRITICAL: triggers UI update
//! ```

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::component::{button, label};
use yororen_ui::theme::ActiveTheme;

use crate::state::CounterState;

/// Root counter component
pub struct CounterApp;

impl CounterApp {
    /// Initialize the component
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Store our entity_id so we can be notified of state changes
        let state = cx.global::<CounterState>();
        *state.notify_entity.lock().unwrap() = Some(cx.entity().entity_id());
        Self
    }
}

/// Render trait - called by gpui when component needs to display
impl Render for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Step 1: Read state
        let state = cx.global::<CounterState>();
        let count = *state.counter.lock().unwrap();
        let theme = cx.theme();

        // Step 2: Build UI
        div()
            .size_full()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(24.))
            .p(px(32.))
            // Counter display
            .child(
                label(&count.to_string())
                    .text_size(px(64.))
                    .font_weight(gpui::FontWeight::SEMIBOLD),
            )
            .child(label("Counter Demo"))
            // Button row
            .child(
                div()
                    .flex()
                    .gap(px(12.))
                    // Decrease button
                    .child(
                        button("decrease")
                            .child("-")
                            .on_click(|_, _, cx| {
                                let state = cx.global::<CounterState>();
                                *state.counter.lock().unwrap() -= 1;
                                // CRITICAL: Notify component to re-render
                                let entity_id = state.notify_entity.lock().unwrap().clone();
                                if let Some(id) = entity_id {
                                    cx.notify(id);
                                }
                            }),
                    )
                    // Reset button
                    .child(
                        button("reset")
                            .child("Reset")
                            .on_click(|_, _, cx| {
                                let state = cx.global::<CounterState>();
                                *state.counter.lock().unwrap() = 0;
                                let entity_id = state.notify_entity.lock().unwrap().clone();
                                if let Some(id) = entity_id {
                                    cx.notify(id);
                                }
                            }),
                    )
                    // Increase button
                    .child(
                        button("increase")
                            .child("+")
                            .on_click(|_, _, cx| {
                                let state = cx.global::<CounterState>();
                                *state.counter.lock().unwrap() += 1;
                                let entity_id = state.notify_entity.lock().unwrap().clone();
                                if let Some(id) = entity_id {
                                    cx.notify(id);
                                }
                            }),
                    ),
            )
    }
}
