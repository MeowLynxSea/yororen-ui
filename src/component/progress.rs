use gpui::{
    Animation, AnimationExt, Div, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, div, px, relative,
};

use gpui::InteractiveElement;

use crate::{animation::constants::duration, theme::ActiveTheme};

use crate::animation::ease_in_out_clamped;

/// Creates a new spinner element.
pub fn spinner() -> Spinner {
    Spinner::new()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpinnerSize {
    Sm,
    Md,
    Lg,
}

impl SpinnerSize {
    fn pixels(self) -> Pixels {
        match self {
            Self::Sm => px(12.),
            Self::Md => px(16.),
            Self::Lg => px(20.),
        }
    }

    fn stroke(self) -> Pixels {
        match self {
            Self::Sm => px(1.5),
            Self::Md => px(2.0),
            Self::Lg => px(2.5),
        }
    }
}

#[derive(IntoElement)]
pub struct Spinner {
    element_id: ElementId,
    base: Div,
    size: SpinnerSize,
    diameter: Option<Pixels>,
    stroke: Option<Pixels>,
    color: Option<Hsla>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            element_id: "ui:spinner".into(),
            base: div(),
            size: SpinnerSize::Md,
            diameter: None,
            stroke: None,
            color: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    /// Spinner diameter.
    ///
    /// When set, overrides the preset `SpinnerSize`.
    pub fn diameter(mut self, diameter: Pixels) -> Self {
        self.diameter = Some(diameter);
        self
    }

    /// Stroke thickness.
    ///
    /// When set, overrides the preset thickness from `SpinnerSize`.
    pub fn stroke(mut self, stroke: Pixels) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for Spinner {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Spinner {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self.element_id.clone();
        let diameter = self.diameter.unwrap_or_else(|| self.size.pixels());
        let stroke = self
            .stroke
            .unwrap_or_else(|| self.size.stroke())
            .max(px(1.));

        let theme = cx.theme();
        let track = theme.border.muted;
        let mut indicator = self.color.unwrap_or(theme.action.primary.bg);
        indicator.a = indicator.a.min(0.9);

        // Draw both track + spinner arc with the same radius to avoid mismatched rings.
        // Animation is done by rebuilding the canvas with a different start angle each frame.
        let make_canvas = move |rotation_radians: f32| {
            let track = track;
            let indicator = indicator;
            gpui::canvas(
                move |_bounds, _window, _cx| (),
                move |bounds, _, window, _cx| {
                    let tau = std::f32::consts::TAU;
                    let start_angle = -std::f32::consts::FRAC_PI_2 + rotation_radians;

                    let center = gpui::point(
                        bounds.origin.x + (bounds.size.width / 2.0),
                        bounds.origin.y + (bounds.size.height / 2.0),
                    );
                    let radius = (bounds.size.width.min(bounds.size.height) / 2.0) - (stroke / 2.0);
                    if radius <= px(0.5) {
                        return;
                    }

                    let steps = 128usize;

                    // Track circle
                    let mut track_path = gpui::PathBuilder::stroke(stroke);
                    for i in 0..=steps {
                        let frac = i as f32 / steps as f32;
                        let angle = start_angle + (tau * frac);
                        let p = gpui::point(
                            center.x + radius * angle.cos(),
                            center.y + radius * angle.sin(),
                        );
                        if i == 0 {
                            track_path.move_to(p);
                        } else {
                            track_path.line_to(p);
                        }
                    }
                    if let Ok(path) = track_path.build() {
                        window.paint_path(path, track);
                    }

                    // Spinner arc
                    let sweep = 0.28 * tau;
                    let mut arc_path = gpui::PathBuilder::stroke(stroke);
                    for i in 0..=steps {
                        let frac = i as f32 / steps as f32;
                        let angle = start_angle + (sweep * frac);
                        let p = gpui::point(
                            center.x + radius * angle.cos(),
                            center.y + radius * angle.sin(),
                        );
                        if i == 0 {
                            arc_path.move_to(p);
                        } else {
                            arc_path.line_to(p);
                        }
                    }
                    if let Ok(path) = arc_path.build() {
                        window.paint_path(path, indicator);
                    }
                },
            )
            .w_full()
            .h_full()
        };

        let animated = make_canvas(0.0).with_animation(
            (id.clone(), "spin"),
            Animation::new(duration::PROGRESS_SPINNER)
                .repeat()
                .with_easing(ease_in_out_clamped),
            move |_this, delta| make_canvas(delta * std::f32::consts::TAU),
        );

        self.base
            .id(self.element_id.clone())
            .relative()
            .w(diameter)
            .h(diameter)
            .child(animated)
    }
}

/// Creates a new progress bar element.
pub fn progress_bar() -> ProgressBar {
    ProgressBar::new()
}

#[derive(IntoElement)]
pub struct ProgressBar {
    element_id: ElementId,
    base: Div,
    value: f32,
    indeterminate: bool,
    height: Pixels,
    track_color: Option<Hsla>,
    fill_color: Option<Hsla>,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            element_id: "ui:progress".into(),
            base: div().w_full(),
            value: 0.0,
            indeterminate: false,
            height: px(10.),
            track_color: None,
            fill_color: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Progress value in `[0.0, 1.0]`.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self.indeterminate = false;
        self
    }

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn track_color(mut self, color: impl Into<Hsla>) -> Self {
        self.track_color = Some(color.into());
        self
    }

    pub fn fill_color(mut self, color: impl Into<Hsla>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for ProgressBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ProgressBar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        // Extract element_id
        let element_id = self.element_id.clone();

        let theme = cx.theme();
        let track = self.track_color.unwrap_or(theme.surface.hover);
        let fill = self.fill_color.unwrap_or(theme.action.primary.bg);

        let height = self.height;
        let t = self.value.clamp(0.0, 1.0);
        let indeterminate = self.indeterminate;

        let indeterminate_id: ElementId =
            (element_id.clone(), "ui:progress-bar:indeterminate").into();
        let fill_id: ElementId = (element_id.clone(), "ui:progress-bar:fill").into();

        let base = self
            .base
            .id(element_id)
            .relative()
            .h(height)
            .rounded_full()
            .bg(track)
            .border_1()
            .border_color(theme.border.muted)
            .overflow_hidden();

        if indeterminate {
            base.child(
                div()
                    .id(indeterminate_id)
                    .absolute()
                    .top_0()
                    .h(height)
                    .rounded_full()
                    .bg(fill)
                    .with_animation(
                        "ui:progress-bar:indeterminate:anim",
                        Animation::new(duration::PROGRESS_CIRCLE)
                            .repeat()
                            .with_easing(ease_in_out_clamped),
                        move |this, delta| {
                            // A more dynamic indeterminate animation: bar grows and shrinks as it
                            // moves, similar to common loading indicators.
                            let width = 0.18 + 0.32 * (1.0 - (2.0 * delta - 1.0).abs());
                            let x = -width + (1.0 + width) * delta;
                            this.left(relative(x)).w(relative(width))
                        },
                    ),
            )
        } else {
            base.child(
                div()
                    .id(fill_id)
                    .absolute()
                    .top_0()
                    .left_0()
                    .h(height)
                    .rounded_full()
                    .bg(fill)
                    .w(relative(t)),
            )
        }
    }
}

/// Creates a new progress circle element.
pub fn progress_circle() -> ProgressCircle {
    ProgressCircle::new()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressCircleSize {
    Sm,
    Md,
    Lg,
}

impl ProgressCircleSize {
    fn pixels(self) -> Pixels {
        match self {
            Self::Sm => px(16.),
            Self::Md => px(24.),
            Self::Lg => px(32.),
        }
    }

    fn stroke(self) -> Pixels {
        match self {
            Self::Sm => px(2.),
            Self::Md => px(3.),
            Self::Lg => px(4.),
        }
    }
}

#[derive(IntoElement)]
pub struct ProgressCircle {
    element_id: ElementId,
    base: Div,
    value: f32,
    size: ProgressCircleSize,
    diameter: Option<Pixels>,
    stroke: Option<Pixels>,
    track_color: Option<Hsla>,
    indicator_color: Option<Hsla>,
}

impl Default for ProgressCircle {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressCircle {
    pub fn new() -> Self {
        Self {
            element_id: "ui:progress-circle".into(),
            base: div(),
            value: 0.0,
            size: ProgressCircleSize::Md,
            diameter: None,
            stroke: None,
            track_color: None,
            indicator_color: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Progress value in `[0.0, 1.0]`.
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn size(mut self, size: ProgressCircleSize) -> Self {
        self.size = size;
        self
    }

    /// Progress circle diameter.
    ///
    /// When set, overrides the preset `ProgressCircleSize`.
    pub fn diameter(mut self, diameter: Pixels) -> Self {
        self.diameter = Some(diameter);
        self
    }

    /// Stroke thickness.
    ///
    /// When set, overrides the preset thickness from `ProgressCircleSize`.
    pub fn stroke(mut self, stroke: Pixels) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn track_color(mut self, color: impl Into<Hsla>) -> Self {
        self.track_color = Some(color.into());
        self
    }

    pub fn indicator_color(mut self, color: impl Into<Hsla>) -> Self {
        self.indicator_color = Some(color.into());
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for ProgressCircle {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ProgressCircle {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ProgressCircle {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let _id = self.element_id.clone();

        let theme = cx.theme();
        let track = self.track_color.unwrap_or(theme.border.muted);
        let indicator = self.indicator_color.unwrap_or(theme.action.primary.bg);

        let diameter = self.diameter.unwrap_or_else(|| self.size.pixels());
        let stroke = self
            .stroke
            .unwrap_or_else(|| self.size.stroke())
            .max(px(1.));
        let t = self.value.clamp(0.0, 1.0);

        self.base
            .id(self.element_id.clone())
            .relative()
            .w(diameter)
            .h(diameter)
            .child(
                gpui::canvas(
                    move |_bounds, _window, _cx| (),
                    move |bounds, _, window, _cx| {
                        let tau = std::f32::consts::TAU;
                        let start_angle = -std::f32::consts::FRAC_PI_2;

                        let center = gpui::point(
                            bounds.origin.x + (bounds.size.width / 2.0),
                            bounds.origin.y + (bounds.size.height / 2.0),
                        );
                        let radius =
                            (bounds.size.width.min(bounds.size.height) / 2.0) - (stroke / 2.0);
                        if radius <= px(0.5) {
                            return;
                        }

                        let steps = 128usize;

                        // Track circle
                        let mut track_path = gpui::PathBuilder::stroke(stroke);
                        for i in 0..=steps {
                            let frac = i as f32 / steps as f32;
                            let angle = start_angle + (tau * frac);
                            let p = gpui::point(
                                center.x + radius * angle.cos(),
                                center.y + radius * angle.sin(),
                            );
                            if i == 0 {
                                track_path.move_to(p);
                            } else {
                                track_path.line_to(p);
                            }
                        }
                        if let Ok(path) = track_path.build() {
                            window.paint_path(path, track);
                        }

                        // Indicator arc
                        let sweep = (t * tau).clamp(0.0, tau);
                        if sweep > 0.0 {
                            let mut arc_path = gpui::PathBuilder::stroke(stroke);
                            for i in 0..=steps {
                                let frac = i as f32 / steps as f32;
                                let angle = start_angle + (sweep * frac);
                                let p = gpui::point(
                                    center.x + radius * angle.cos(),
                                    center.y + radius * angle.sin(),
                                );
                                if i == 0 {
                                    arc_path.move_to(p);
                                } else {
                                    arc_path.line_to(p);
                                }
                            }
                            if let Ok(path) = arc_path.build() {
                                window.paint_path(path, indicator);
                            }
                        }
                    },
                )
                .w_full()
                .h_full(),
            )
    }
}
