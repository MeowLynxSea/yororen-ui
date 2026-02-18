use std::path::PathBuf;

use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::label;

use crate::clipboard::FileClipboard;
use crate::format;

pub struct FileBrowserDetails;

impl FileBrowserDetails {
    pub fn render(selected_path: &Option<PathBuf>, clipboard: &Option<FileClipboard>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(label("Selected:").muted(true))
            .child(label(format::path_or_dash(selected_path)).mono(true).ellipsis(true))
            .child(div().w(px(24.)))
            .child(label("Clipboard:").muted(true))
            .child(label(format::clipboard_label(clipboard)).mono(true).ellipsis(true))
    }
}

