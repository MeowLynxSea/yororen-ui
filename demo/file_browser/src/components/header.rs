//! File Browser Header Component
//!
//! Displays the application title and action buttons.

use std::path::PathBuf;

use gpui::{IntoElement, ParentElement, Styled, div, px};

use yororen_ui::component::{button, label};
use yororen_ui::theme::ActionVariantKind;

use crate::actions;

/// Header component showing title and action buttons
pub struct FileBrowserHeader;

impl FileBrowserHeader {
    /// Renders the header with title, root path, and action buttons
    pub fn render(root: &PathBuf) -> impl IntoElement {
        let root_label = root.to_string_lossy().to_string();

        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(label("File Browser Demo").strong(true))
                    .child(label(root_label).muted(true).mono(true).ellipsis(true)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        button("file-browser:refresh")
                            .variant(ActionVariantKind::Neutral)
                            .child("Refresh")
                            .on_click(|_ev, window, cx| {
                                actions::refresh(window, cx);
                            }),
                    )
                    .child(
                        button("file-browser:root")
                            .variant(ActionVariantKind::Neutral)
                            .child("Pick Root")
                            .on_click(|_ev, window, cx| {
                                actions::prompt_for_root(window, cx);
                            }),
                    ),
            )
    }
}

