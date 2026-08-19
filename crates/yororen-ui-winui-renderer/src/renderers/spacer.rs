//! `WinUISpacerRenderer` — default `SpacerRenderer` impl.
//!
//! Returns a flexible empty `Stateful<Div>`. The caller is expected
//! to layer explicit width / height on top via `.w(...)` / `.h(...)`
//! after `.render(cx)`; when no explicit sizing is supplied the
//! spacer simply expands to fill available space.

use std::sync::Arc;

use gpui::{App, Div, InteractiveElement, Stateful, Styled, div};

use yororen_ui_core::headless::spacer::SpacerProps;

pub use yororen_ui_core::renderer::spacer::{SpacerRenderState, SpacerRenderer};

pub struct WinUISpacerRenderer;

impl SpacerRenderer for WinUISpacerRenderer {
    fn compose(&self, props: &SpacerProps, _cx: &App) -> Stateful<Div> {
        div().id(props.id.clone()).flex_1()
    }
}

pub fn arc_spacer<T: SpacerRenderer + 'static>(r: T) -> Arc<dyn SpacerRenderer> {
    Arc::new(r)
}
