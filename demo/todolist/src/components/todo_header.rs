//! yororen-ui Header Component
//!
//! This module demonstrates the standard pattern for building header components in yororen-ui applications.
//! The TodoHeader component displays the application title and provides a toggle switch for switching
//! between compact and detailed view modes.
//!
//! ## Header Design Pattern
//!
//! Headers in yororen-ui applications typically serve as the top-level navigation and branding element.
//! This component demonstrates a common pattern where:
//!
//! - **Title Area**: Displays the application name or page title using the `heading` component
//! - **Settings/Controls**: Provides toggleable settings or view options aligned to the right
//!
//! ## Layout Approach
//!
//! The header uses flexbox layout with `justify_between` to position the title on the left
//! and controls on the right, ensuring consistent spacing and alignment regardless of content width.
//!
//! ## View Mode Toggle
//!
//! The switch component allows users to toggle between compact and detailed views of the todo list:
//!
//! - The `compact_mode` state is stored globally
//! - The switch's `on_toggle` handler updates this state when the user toggles the switch
//! - Changes to this state trigger a re-render of the root component, which passes the updated
//!   value to child components to determine their rendering style
//!
//! ## Key Components Used
//!
//! - `heading` - Page/section title with configurable heading level (H1 in this case)
//! - `label` - Inline text label for the toggle switch description
//! - `switch` - Toggle switch for boolean settings (view mode preference)

use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{heading, label, switch};
use yororen_ui::i18n::Translate;

use crate::state::TodoState;

/// Header with title and settings
pub struct TodoHeader;

impl TodoHeader {
    /// Standard header pattern
    pub fn render(cx: &gpui::App, compact_mode: bool) -> impl IntoElement {
        let title = cx.t("demo.todolist.title");
        let compact_label = cx.t("demo.todolist.compact_mode");

        div()
            .flex()
            .items_center()
            .justify_between()
            // Page title
            .child(heading(title).level(yororen_ui::component::HeadingLevel::H1))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    // Setting with label and switch
                    .child(label(compact_label))
                    .child(
                        switch("compact-mode")
                            .checked(compact_mode)
                            .on_toggle(|value, _, _window, cx| {
                                let model = cx.global::<TodoState>().model.clone();
                                model.update(cx, |model, cx| {
                                    model.compact_mode = value;
                                    cx.notify();
                                });
                            }),
                    ),
            )
    }
}
