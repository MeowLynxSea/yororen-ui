use gpui::{Hsla, InteractiveElement, Styled};

pub trait HoverRefreshExt: Sized {
    fn hover_bg_refresh(self, bg: impl Into<Hsla>) -> Self;
}

impl<T> HoverRefreshExt for T
where
    T: Styled + InteractiveElement + Sized,
{
    fn hover_bg_refresh(self, bg: impl Into<Hsla>) -> Self {
        let bg = bg.into();
        self.hover(move |this| this.bg(bg))
    }
}

#[macro_export]
macro_rules! hover_bg_refresh {
    ($element:expr, $bg:expr) => {
        $crate::hover_refresh::HoverRefreshExt::hover_bg_refresh($element, $bg)
    };
}
