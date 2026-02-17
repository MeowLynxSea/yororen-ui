//! yororen-ui Header Component
//!
//! Demonstrates header/toolbar patterns in yororen-ui.
//!
//! ## Key Components Used
//!
//! - `heading` - Page/section titles
//! - `label` - Inline text labels
//! - `switch` - Toggle switches for boolean settings

use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{heading, label, switch};

use crate::state::TodoState;

/// Header with title and settings
pub struct TodoHeader;

impl TodoHeader {
    /// Standard header pattern
    pub fn render(compact_mode: bool) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            // Page title
            .child(heading("Todo Manager").level(yororen_ui::component::HeadingLevel::H1))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    // Setting with label and switch
                    .child(label("Compact Mode"))
                    .child(
                        switch("compact-mode")
                            .checked(compact_mode)
                            .on_toggle(|value, _, _window, cx| {
                                // Update global state
                                let state = cx.global::<TodoState>();
                                *state.compact_mode.lock().unwrap() = value;
                            }),
                    ),
            )
    }
}
