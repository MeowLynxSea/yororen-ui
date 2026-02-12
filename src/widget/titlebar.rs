use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Decorations, Entity, FontWeight, MouseDownEvent, SharedString,
    div, ease_out_quint, prelude::*, px,
};

use crate::{component::icon, theme::ActiveTheme};

pub const DEFAULT_NAV_ITEMS: [&str; 5] = ["Home", "Explore", "Player", "Components", "Settings"];

#[derive(Clone, Debug, Default)]
pub struct TitleBarConfig {
    pub title: SharedString,
    pub badge: Option<SharedString>,
}

impl TitleBarConfig {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            badge: None,
        }
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

pub fn titlebar(cx: &mut App) -> Entity<TitleBar> {
    cx.new(|cx| TitleBar::new(cx))
}

pub fn titlebar_with_config(cx: &mut App, config: TitleBarConfig) -> Entity<TitleBar> {
    cx.new(|cx| TitleBar::with_items_and_config(cx, DEFAULT_NAV_ITEMS, config))
}

pub fn titlebar_with_items(
    cx: &mut App,
    items: impl IntoIterator<Item = impl Into<SharedString>>,
) -> Entity<TitleBar> {
    cx.new(|cx| TitleBar::with_items(cx, items))
}

pub fn titlebar_with_items_and_config(
    cx: &mut App,
    items: impl IntoIterator<Item = impl Into<SharedString>>,
    config: TitleBarConfig,
) -> Entity<TitleBar> {
    cx.new(|cx| TitleBar::with_items_and_config(cx, items, config))
}

pub fn navigator(cx: &mut App) -> Navigator {
    Navigator::new(cx)
}

pub fn navigator_with_items(
    cx: &mut App,
    items: impl IntoIterator<Item = impl Into<SharedString>>,
) -> Navigator {
    Navigator::with_items(cx, items)
}

#[derive(IntoElement, Clone)]
pub struct Navigator {
    navigator_state: Entity<NavigatorState>,
    items: Vec<SharedString>,
}

impl Navigator {
    pub fn current(&self, cx: &App) -> usize {
        self.navigator_state.read(cx).current
    }
}

#[derive(Default)]
pub struct NavigatorState {
    current: usize,
    prev: usize,
}

impl NavigatorState {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Render for NavigatorState {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let current = self.current;
        let prev = self.prev;

        div()
            .id("slider")
            .absolute()
            .w_12()
            .h_7()
            .bg(cx.theme().action.primary.bg)
            .rounded_full()
            .with_animation(
                format!("navigator-slider-{}", current),
                Animation::new(Duration::from_millis(200)).with_easing(ease_out_quint()),
                move |this, delta| {
                    let target_left = (current * 52) as f32;
                    let current_left = (prev * 52) as f32;
                    let new_left = current_left + (target_left - current_left) * delta;
                    this.left(px(new_left))
                },
            )
    }
}

impl Navigator {
    pub fn new(cx: &mut App) -> Self {
        Self::with_items(cx, DEFAULT_NAV_ITEMS)
    }

    pub fn with_items(
        cx: &mut App,
        items: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            navigator_state: cx.new(|_cx| NavigatorState::new()),
            items: items.into_iter().map(Into::into).collect(),
        }
    }
}

impl RenderOnce for Navigator {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let state = self.navigator_state.clone();
        let current = self.navigator_state.read(cx).current;
        let items = self.items;

        div()
            .id("navigator")
            .mr_3()
            .flex()
            .flex_row()
            .items_center()
            .child(self.navigator_state)
            .child(
                div()
                    .id("menu-items")
                    .text_sm()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children(items.into_iter().enumerate().map(move |(i, t)| {
                        let state = state.clone();
                        div()
                            .id(format!("nav-item-{}", i))
                            .w_12()
                            .h_7()
                            .rounded_full()
                            .text_color(if i == current {
                                cx.theme().action.primary.fg
                            } else {
                                cx.theme().action.neutral.fg
                            })
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(t)
                            .cursor_pointer()
                            .when(current != i, |this| {
                                this.hover(|this| this.bg(cx.theme().action.neutral.hover_bg))
                            })
                            .on_click(move |_ev, _window, cx| {
                                state.update(cx, |this, _cx| {
                                    this.prev = this.current;
                                    this.current = i;
                                });
                            })
                    })),
            )
    }
}

pub struct TitleBar {
    navigator: Navigator,
    title: SharedString,
    badge: Option<SharedString>,
}

impl TitleBar {
    pub fn new(cx: &mut App) -> Self {
        Self {
            navigator: navigator(cx),
            title: SharedString::default(),
            badge: None,
        }
    }

    pub fn with_items(
        cx: &mut App,
        items: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self::with_items_and_config(cx, items, TitleBarConfig::default())
    }

    pub fn with_items_and_config(
        cx: &mut App,
        items: impl IntoIterator<Item = impl Into<SharedString>>,
        config: TitleBarConfig,
    ) -> Self {
        Self {
            navigator: navigator_with_items(cx, items),
            title: config.title,
            badge: config.badge,
        }
    }

    pub fn current_page(&self, cx: &App) -> usize {
        self.navigator.current(cx)
    }
}

#[cfg(all(target_os = "macos", macos_sdk_26))]
const TRAFFIC_LIGHT_WIDTH: f32 = 73.;

#[cfg(all(target_os = "macos", not(macos_sdk_26)))]
const TRAFFIC_LIGHT_WIDTH: f32 = 66.;

#[cfg(not(target_os = "macos"))]
const TRAFFIC_LIGHT_WIDTH: f32 = 0.;

impl Render for TitleBar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let decorations = window.window_decorations();
        let is_tiled = match decorations {
            Decorations::Client { tiling } => tiling.is_tiled(),
            _ => false,
        };

        let window_is_maximized = is_tiled;

        let drag_area = div()
            .id("titlebar:drag-area")
            .window_control_area(gpui::WindowControlArea::Drag)
            .h_full()
            .flex()
            .flex_row()
            .items_center()
            // Occupy remaining space so nav items stay on the right.
            // (Also keeps the empty area draggable on Windows.)
            .flex_grow()
            .min_w(px(0.0))
            .pl_3()
            .when(
                !window.is_fullscreen() && cfg!(target_os = "macos"),
                |this| this.child(div().id("traffic-light-pos").w(px(TRAFFIC_LIGHT_WIDTH))),
            )
            .child(
                div()
                    .id("title")
                    .font_weight(FontWeight::SEMIBOLD)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(self.title.clone())
                    .children(self.badge.clone().map(|badge| {
                        div()
                            .h_6()
                            .px_2()
                            .bg(cx.theme().surface.raised)
                            .text_color(cx.theme().content.primary)
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .rounded_3xl()
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(badge)
                            .into_any_element()
                    })),
            )
            // Keep the draggable region on the left side (including empty space before the nav)
            // so it doesn't interfere with nav items or window controls.
            .child(div().flex_grow());

        div()
            .id("titlebar")
            .w_full()
            .h_10()
            .text_color(cx.theme().content.primary)
            .flex()
            .flex_row()
            .items_center()
            .child(drag_area)
            .child(self.navigator.clone())
            .when(cfg!(not(target_os = "macos")) && !is_tiled, |this| {
                this.children((0..3).map(|i| {
                    let (area, icon_path): (gpui::WindowControlArea, SharedString) = match i {
                        0 => (
                            gpui::WindowControlArea::Min,
                            "icons/window-minimize.svg".into(),
                        ),
                        1 => (
                            gpui::WindowControlArea::Max,
                            if window_is_maximized {
                                "icons/window-maximize-on.svg".into()
                            } else {
                                "icons/window-maximize-off.svg".into()
                            },
                        ),
                        _ => (
                            gpui::WindowControlArea::Close,
                            "icons/window-close.svg".into(),
                        ),
                    };

                    div()
                        .id(("window-control", i as usize))
                        .window_control_area(area)
                        .w(px(56.))
                        .h_full()
                        .flex()
                        .justify_center()
                        .items_center()
                        .child(icon(icon_path).size(px(12.)))
                        .cursor_pointer()
                        .map(|this| this.hover(|this| this.bg(cx.theme().action.neutral.hover_bg)))
                        .on_click(cx.listener(move |_this, _ev, window, cx| match i {
                            0 => window.minimize_window(),
                            1 => {
                                window.zoom_window();
                                cx.notify();
                            }
                            2 => window.remove_window(),
                            _ => {}
                        }))
                }))
            })
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |_this, ev: &MouseDownEvent, window, cx| {
                    if ev.click_count > 1 {
                        window.zoom_window();
                        cx.notify();
                    }
                }),
            )
    }
}
