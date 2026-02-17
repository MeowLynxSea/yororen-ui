use gpui::{IntoElement, ParentElement, Styled, div, px};
use yororen_ui::component::{heading, label, switch};

use crate::state::TodoState;

pub struct TodoHeader;

impl TodoHeader {
    pub fn render(compact_mode: bool) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(heading("Todo 任务管理器").level(yororen_ui::component::HeadingLevel::H1))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(label("紧凑模式"))
                    .child(
                        switch("compact-mode")
                            .checked(compact_mode)
                            .on_toggle(|value, _, _window, cx| {
                                let state = cx.global::<TodoState>();
                                *state.compact_mode.lock().unwrap() = value;
                            }),
                    ),
            )
    }
}
