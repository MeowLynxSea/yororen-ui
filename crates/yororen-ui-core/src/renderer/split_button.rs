//! `SplitButtonRenderer` — visual contract for `SplitButton`.
//!
//! Trait surface is just `compose`. Inherent helpers
//! (primary_bg / primary_fg / chevron_bg / chevron_fg /
//! chevron_hover_bg / min_height / border_radius / gap)
//! stay on the concrete renderer type.

use std::any::Any;

use gpui::{App, Div};

use crate::headless::split_button::SplitButtonProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct SplitButtonRenderState {
    pub open: bool,
    pub disabled: bool,
    /// Toggle-mode "on" value. `false` for classic action split
    /// buttons (`props.toggled` is `None` or `Some(false)`).
    pub toggled: bool,
}

pub trait SplitButtonRenderer: Any + Send + Sync {
    fn compose(&self, props: &SplitButtonProps, cx: &App) -> Div;
}
