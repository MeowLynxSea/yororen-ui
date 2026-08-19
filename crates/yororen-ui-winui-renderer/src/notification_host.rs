use gpui::{
    App, ClickEvent, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div,
};

use gpui::prelude::FluentBuilder;

use crate::{
    component::{Icon, IconName, label, toast},
    notification::{DismissStrategy, NotificationCenter},
    theme::ActiveTheme,
};

/// A host element that renders the global [`NotificationCenter`] as a toast stack.
///
/// Render this once near the root of your window (e.g. as the last child of your app root)
/// so it paints above other content.
pub fn notification_host() -> NotificationHost {
    NotificationHost::new()
}

#[derive(IntoElement)]
pub struct NotificationHost {
    base: gpui::Div,
    max_width: gpui::Pixels,
    offset: gpui::Pixels,
}

impl Default for NotificationHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationHost {
    pub fn new() -> Self {
        Self {
            base: div(),
            max_width: gpui::px(0.),
            offset: gpui::px(0.),
        }
    }

    /// Constrain toast width.
    pub fn max_width(mut self, width: gpui::Pixels) -> Self {
        self.max_width = width;
        self
    }

    /// Offset from the top-right corner.
    pub fn offset(mut self, offset: gpui::Pixels) -> Self {
        self.offset = offset;
        self
    }
}

impl ParentElement for NotificationHost {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for NotificationHost {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for NotificationHost {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if cx.try_global::<NotificationCenter>().is_none() {
            cx.set_global(NotificationCenter::new());
        }

        let center = cx.global::<NotificationCenter>().clone();
        center.register_host_window(window.window_handle());

        let (persist_enabled, persist_key) = center.persistence_config();
        if persist_enabled {
            // IMPORTANT: `use_keyed_state` can only be called during layout/paint phases.
            // `NotificationHost::render` is executed in those phases.
            let persisted = window.use_keyed_state(
                (gpui::ElementId::from("ui:notification-center"), persist_key),
                cx,
                |_, _| crate::notification::PersistedState::default(),
            );
            center.bind_persisted_state(persisted, cx);
        } else {
            center.unbind_persisted_state();
        }

        let sub = window.observe_global::<NotificationCenter>(cx, |_window, _cx| {});
        sub.detach();

        let items = center.items();
        // Owned `Arc<Theme>` (one atomic refcount bump): the
        // `.map(move |n| { ... theme ... })` below is a `move`
        // closure that consumes `theme`, so a borrow from `cx`
        // wouldn't satisfy `'static`. See `theme::mod` for the
        // full discussion of why this is cheap.
        let theme = cx.theme().clone();

        let direction = cx.theme().text_direction;
        let offset_value: f32 = self.offset.into();
        let resolved_offset = if offset_value > 0.0 {
            self.offset
        } else {
            theme.tokens.control.notification.host_padding
        };
        let max_w_value: f32 = self.max_width.into();
        let resolved_max_w = if max_w_value > 0.0 {
            self.max_width
        } else {
            theme.tokens.control.notification.max_width
        };

        self.base
            .id("ui:notification-host")
            .absolute()
            .top_0()
            .when(direction.is_rtl(), |this| this.left_0())
            .when(!direction.is_rtl(), |this| this.right_0())
            .mt(resolved_offset)
            .when(direction.is_rtl(), |this| this.ml(resolved_offset))
            .when(!direction.is_rtl(), |this| this.mr(resolved_offset))
            .flex()
            .flex_col()
            .gap_2()
            .items_end()
            .children(items.into_iter().rev().map(move |n| {
                let id = n.id;
                let dismiss = n.dismiss.clone();

                let center_for_click = center.clone();
                let center_for_dismiss = center.clone();

                fn adjust_hover(bg: Hsla) -> Hsla {
                    let delta = if bg.l > 0.5 { -0.06 } else { 0.06 };
                    Hsla {
                        l: (bg.l + delta).clamp(0.0, 1.0),
                        ..bg
                    }
                }

                let (bg, fg) = match n.kind {
                    crate::component::ToastKind::Neutral => {
                        (theme.surface.raised, theme.content.primary)
                    }
                    crate::component::ToastKind::Success => {
                        (theme.status.success.bg, theme.content.on_status)
                    }
                    crate::component::ToastKind::Warning => {
                        (theme.status.warning.bg, theme.content.on_status)
                    }
                    crate::component::ToastKind::Error => {
                        (theme.status.error.bg, theme.content.on_status)
                    }
                    crate::component::ToastKind::Info => {
                        (theme.status.info.bg, theme.content.on_status)
                    }
                };
                let close_hover_bg = adjust_hover(bg);
                let close_border = Hsla { a: 0.25, ..fg };

                let close = div()
                    .id(("ui:notification:dismiss", id.raw()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(theme.tokens.control.tag.close_button_size)
                    .h(theme.tokens.control.tag.close_button_size)
                    .rounded_sm()
                    .cursor_pointer()
                    .text_color(fg)
                    .hover(move |this| {
                        this.bg(close_hover_bg)
                            .text_color(fg)
                            .border_1()
                            .border_color(close_border)
                    })
                    .on_click(move |_ev, window, cx| {
                        cx.stop_propagation();
                        center_for_dismiss.dismiss_from_ui(id, window, cx);
                        window.refresh();
                    })
                    .child(
                        Icon::new(IconName::Close)
                            .size(theme.tokens.control.tag.close_icon_size)
                            .color(fg),
                    );

                let mut body = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .when_some(n.title.clone(), |this, title| {
                        this.child(label(title).strong(true).inherit_color(true))
                    })
                    .child(label(n.message.clone()).inherit_color(true).ellipsis(false));

                if let Some(action) = n.action_label.clone() {
                    body = body.child(
                        div()
                            .text_xs()
                            .opacity(0.85)
                            .child(label(action).inherit_color(true)),
                    );
                }

                let toast_el = toast()
                    .kind(n.kind)
                    .wrap(true)
                    .max_width(resolved_max_w)
                    .content(body)
                    .trailing(close);

                div()
                    .id(("ui:notification", id.raw()))
                    .cursor_pointer()
                    .on_click(move |ev: &ClickEvent, window, cx| {
                        center_for_click.click(id, ev, window, cx);
                        if matches!(dismiss, DismissStrategy::After { .. }) {
                            center_for_click.dismiss_from_ui(id, window, cx);
                        }
                        window.refresh();
                    })
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap_1()
                    .child(toast_el)
            }))
    }
}
