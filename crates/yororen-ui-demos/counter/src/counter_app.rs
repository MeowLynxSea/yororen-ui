//! yororen-ui Counter Component
//!
//! Migrated to the new `headless::layout::*` API: instead of
//! hand-writing `div().flex().flex_col().items_center()…`
//! chains, the body uses `center` / `column` / `row` layout
//! primitives that read spacing and inset values from the
//! active theme. The result is the same DOM but the code is
//! declarative and theme-aware.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window};
use yororen_ui::ActiveTheme;
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::headless::layout::{Inset, Spacing, center, column, row};

use crate::state::CounterState;

pub struct CounterApp;

impl Render for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<CounterState>();
        let count = state.counter.read(cx).value;
        let inc_entity = state.counter.clone();
        let dec_entity = state.counter.clone();
        let reset_entity = state.counter.clone();

        let card = column("card", cx)
            .gap(Spacing::Lg)
            .p(Inset::Lg)
            .items_center()
            .justify_center()
            .child(
                label("subtitle", "Counter Demo", cx)
                    .strong(true)
                    .render(cx)
                    .text_size(gpui::px(28.))
                    .into_any_element(),
            )
            .child(
                label("count", count.to_string(), cx)
                    .render(cx)
                    .text_size(gpui::px(20.))
                    .into_any_element(),
            )
            .child(
                row("buttons", cx)
                    .gap(Spacing::Sm)
                    .child(
                        button("decrease", cx)
                            .on_click(move |_, _, cx| {
                                dec_entity.update(cx, |c, cx| {
                                    c.value -= 1;
                                    cx.notify();
                                });
                            })
                            .render(cx)
                            .child("-"),
                    )
                    .child(
                        button("reset", cx)
                            .on_click(move |_, _, cx| {
                                reset_entity.update(cx, |c, cx| {
                                    c.value = 0;
                                    cx.notify();
                                });
                            })
                            .render(cx)
                            .child("Reset"),
                    )
                    .child(
                        button("increase", cx)
                            .on_click(move |_, _, cx| {
                                inc_entity.update(cx, |c, cx| {
                                    c.value += 1;
                                    cx.notify();
                                });
                            })
                            .render(cx)
                            .child("+"),
                    )
                    .render(cx),
            )
            .render(cx);

        // Paint the root with the theme's surface color so the
        // window background is opaque on all platforms (notably
        // Windows, where an unset background shows through as
        // transparent).
        let surface = cx
            .theme()
            .get_color("surface.base")
            .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.98, 1.0));

        center("root", cx)
            .w_full()
            .h_full()
            .child(card)
            .render(cx)
            .bg(surface)
    }
}
