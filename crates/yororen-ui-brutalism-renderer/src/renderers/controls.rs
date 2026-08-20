//! Brutalist control renderers: `Switch`, `Checkbox`, `Radio`,
//! `RadioGroup`, `Slider`.

use std::sync::{Arc, Mutex};

use gpui::{
    App, BorderStyle, Bounds, Corners, CursorStyle, Div, Edges as GpuiEdges, Element, FocusHandle,
    GlobalElementId, Hsla, InteractiveElement, IntoElement, LayoutId, PaintQuad, ParentElement,
    Pixels, Stateful, StatefulInteractiveElement, Style, Styled, Window, div, hsla, point, px,
    size,
};
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::animation::{AnimatedMarginElement, AnimatedOpacityElement};

use crate::style::{BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, brutal_border_color};

// =====================================================================
// Switch
// =====================================================================

pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};

pub struct BrutalSwitchRenderer;

// Inherent helpers — *not* part of the `SwitchRenderer` trait surface.
impl BrutalSwitchRenderer {
    pub fn track_w(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.track_w")
            .unwrap_or(52.0) as f32)
    }
    pub fn track_h(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.track_h")
            .unwrap_or(30.0) as f32)
    }
    pub fn knob_size(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.knob_size")
            .unwrap_or(22.0) as f32)
    }
    pub fn padding(&self, _: &SwitchRenderState, _: &Theme) -> Pixels {
        px(0.0)
    }
    pub fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or(BRUTAL_BORDER)
            } else {
                theme
                    .get_color("action.primary.bg")
                    .unwrap_or(BRUTAL_BORDER)
            }
        } else {
            theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn track_border(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            // Use a clearly visible color (content.tertiary)
            // for the unchecked hover so the track is not lost
            // against the page background (`surface.base`).
            theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.checked {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn focus_color(&self, _: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_opacity(&self, _: &SwitchRenderState, _: &Theme) -> f32 {
        0.5
    }
}

impl SwitchRenderer for BrutalSwitchRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::switch::SwitchProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = SwitchRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let track = self.track_bg(&state, theme);
        let _knob_color = self.knob_bg(&state, theme);
        let w = self.track_w(&state, theme);
        let h = self.track_h(&state, theme);
        let knob_size = self.knob_size(&state, theme);
        let pad = self.padding(&state, theme);
        let track_hover = self.track_hover_bg(&state, theme);
        let track_active = self.track_active_bg(&state, theme);
        let border = brutal_border_color(theme);

        // Cross-fade the knob colour between unchecked and checked
        // states while it slides.
        let unchecked_knob_color = self.knob_bg(
            &SwitchRenderState {
                checked: false,
                ..state
            },
            theme,
        );
        let checked_knob_color = self.knob_bg(
            &SwitchRenderState {
                checked: true,
                ..state
            },
            theme,
        );
        let knob_off = div().absolute().inset_0().bg(unchecked_knob_color);
        let knob_on = div().absolute().inset_0().bg(checked_knob_color);
        let knob_inner = div()
            .relative()
            .size(knob_size)
            .child(AnimatedOpacityElement::new(
                (props.id.clone(), "knob-off"),
                !props.checked,
                knob_off,
            ))
            .child(AnimatedOpacityElement::new(
                (props.id.clone(), "knob-on"),
                props.checked,
                knob_on,
            ));

        let slide_distance = {
            let w_f: f32 = w.into();
            let knob_f: f32 = knob_size.into();
            let pad_f: f32 = pad.into();
            px((w_f - knob_f - pad_f * 2.0).max(0.0))
        };
        let knob_animated = AnimatedMarginElement::new(
            (props.id.clone(), "knob-slide"),
            props.checked,
            slide_distance,
            knob_inner,
        );

        div()
            .id(props.id.clone())
            .bg(track)
            .border_2()
            .border_color(border)
            .w(w)
            .h(h)
            .p(pad)
            .flex()
            .items_center()
            .justify_start()
            .track_focus(focus_handle)
            .child(knob_animated)
            .hover(|s| s.bg(track_hover))
            .active(|s| s.bg(track_active))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
    }
}

// =====================================================================
// Checkbox
// =====================================================================

pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};

pub struct BrutalCheckboxRenderer;

// Inherent helpers — *not* part of the `CheckboxRenderer` trait surface.
impl BrutalCheckboxRenderer {
    pub fn box_size(&self, _: &CheckboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.checkbox.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn check_size(&self, _: &CheckboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.checkbox.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else if state.checked || state.indeterminate {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or(BRUTAL_BORDER)
            } else {
                theme
                    .get_color("action.primary.bg")
                    .unwrap_or(BRUTAL_BORDER)
            }
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if (state.checked || state.indeterminate) && state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            brutal_border_color(theme)
        }
    }
    pub fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked || state.indeterminate {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked || state.indeterminate {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn check_fg(&self, _: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn focus_color(&self, _: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_opacity(&self, _: &CheckboxRenderState, _: &Theme) -> f32 {
        0.5
    }
}

impl CheckboxRenderer for BrutalCheckboxRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::checkbox::CheckboxProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = CheckboxRenderState {
            checked: props.checked,
            indeterminate: props.indeterminate,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let bg = self.box_bg(&state, theme);
        let border = self.box_border(&state, theme);
        let size = self.box_size(&state, theme);
        let check_size = self.check_size(&state, theme);
        let hover_bg = self.box_hover_bg(&state, theme);
        let active_bg = self.box_active_bg(&state, theme);

        // The glyph (checkmark or mixed-state dash) is always
        // mounted and faded in/out so the state transition is
        // animated. The brutal dash is a wide flat bar — thicker
        // than the token renderer's so it reads at a glance.
        let check_color = self.box_border(
            &CheckboxRenderState {
                checked: true,
                ..state
            },
            theme,
        );
        let check_f: f32 = check_size.into();
        let check = if props.indeterminate {
            div()
                .bg(check_color)
                .w(px(check_f * 1.4))
                .h(px((check_f * 0.4).max(4.0)))
        } else {
            div().bg(check_color).size(check_size)
        };
        let animated_check = AnimatedOpacityElement::new(
            (props.id.clone(), "check"),
            props.checked || props.indeterminate,
            check,
        );

        div()
            .id(props.id.clone())
            .bg(bg)
            .border_2()
            .border_color(border)
            .size(size)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle)
            .child(animated_check)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
    }
}

// =====================================================================
// Radio
// =====================================================================

pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};

pub struct BrutalRadioRenderer;

// Inherent helpers — *not* part of the `RadioRenderer` trait surface.
impl BrutalRadioRenderer {
    pub fn ring_size(&self, _: &RadioRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.radio.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn dot_size(&self, _: &RadioRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.radio.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked && state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            brutal_border_color(theme)
        }
    }
    pub fn ring_hover_bg(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn ring_active_bg(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
    }
    pub fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn focus_color(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_opacity(&self, _: &RadioRenderState, _: &Theme) -> f32 {
        0.5
    }
}

impl RadioRenderer for BrutalRadioRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::radio::RadioProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = RadioRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let bg = self.ring_bg(&state, theme);
        let border = self.ring_border(&state, theme);
        let ring_size = self.ring_size(&state, theme);
        let dot_size = self.dot_size(&state, theme);
        let dot_fg = self.dot_fg(&state, theme);
        let hover_bg = self.ring_hover_bg(&state, theme);
        let active_bg = self.ring_active_bg(&state, theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_2()
            .border_color(border)
            .size(ring_size)
            .rounded(px(9999.))
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);
        if props.checked {
            el = el.child(div().bg(dot_fg).size(dot_size).rounded(px(9999.)));
        }
        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
    }
}

// =====================================================================
// RadioGroup
// =====================================================================

pub use yororen_ui_core::renderer::radio_group::{RadioGroupRenderState, RadioGroupRenderer};

pub struct BrutalRadioGroupRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalRadioGroupRenderer {
    /// Horizontal gap between radio buttons. Brutalism keeps a
    /// slightly wider gap than the default renderer so adjacent
    /// thick borders don't visually collide.
    pub fn gap(&self, _state: &RadioGroupRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.spacing.gap_3")
            .or_else(|| theme.get_number("tokens.spacing.normal"))
            .unwrap_or(12.0) as f32)
    }
}

impl RadioGroupRenderer for BrutalRadioGroupRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::radio_group::RadioGroupProps,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = RadioGroupRenderState {
            selected_index: props.selected_index,
        };
        let gap = self.gap(&state, theme);

        div()
            .id(props.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
    }
}

// =====================================================================
// Slider
// =====================================================================

pub use yororen_ui_core::renderer::slider::{
    SliderRenderOutput, SliderRenderState, SliderRenderer,
};

pub struct BrutalSliderRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalSliderRenderer {
    pub fn track_h(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.track_h")
            .unwrap_or(12.0) as f32
    }
    pub fn knob_size(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.thumb_size")
            .unwrap_or(22.0) as f32
    }
    pub fn track_w(&self, _state: &SliderRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.slider.track_w")
            .unwrap_or(240.0) as f32)
    }
    pub fn border_width(&self, _state: &SliderRenderState, theme: &Theme) -> f32 {
        theme
            .get_number("tokens.control.slider.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32
    }
    pub fn track_bg(&self, state: &SliderRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn fill_bg(&self, state: &SliderRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn knob_bg(&self, state: &SliderRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn border(&self, _state: &SliderRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
}

impl SliderRenderer for BrutalSliderRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::slider::SliderProps,
        cx: &App,
    ) -> SliderRenderOutput {
        let theme = cx.theme();
        let state = SliderRenderState {
            disabled: props.disabled,
        };
        let track_h = self.track_h(&state, theme);
        let knob_size = self.knob_size(&state, theme);
        let track_w = self.track_w(&state, theme);
        let border_w = self.border_width(&state, theme);
        let track_bg = self.track_bg(&state, theme);
        let fill_bg = self.fill_bg(&state, theme);
        let knob_bg = self.knob_bg(&state, theme);
        let border = self.border(&state, theme);

        let pct = ((props.value - props.min) / (props.max - props.min)).clamp(0.0, 1.0);

        let bounds_store: Arc<Mutex<Option<Bounds<Pixels>>>> = Arc::new(Mutex::new(None));

        // The total row height is enough for the knob plus the
        // brutalist border on both sides, with a small gap.
        let row_h = (knob_size + border_w * 2.0 + 4.0).max(30.0);

        let track_element = BrutalSliderTrackElement {
            bounds: bounds_store.clone(),
            pct,
            row_h,
            track_h,
            knob_size,
            border_w,
            track_bg,
            fill_bg,
            knob_bg,
            border,
        };

        let visual = div()
            .id(props.id.clone())
            .w(track_w)
            .h(px(row_h))
            .cursor(if props.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .child(track_element);

        SliderRenderOutput {
            visual,
            track_bounds: bounds_store,
        }
    }
}

/// Internal `Element` painting the brutalism slider track, fill,
/// and knob. Same bounds-publishing contract as the default
/// renderer so the headless drag handlers in
/// `SliderProps::render` can resolve mouse positions.
struct BrutalSliderTrackElement {
    bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    pct: f32,
    row_h: f32,
    track_h: f32,
    knob_size: f32,
    border_w: f32,
    track_bg: Hsla,
    fill_bg: Hsla,
    knob_bg: Hsla,
    border: Hsla,
}

impl IntoElement for BrutalSliderTrackElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for BrutalSliderTrackElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = px(self.row_h).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        *self.bounds.lock().unwrap() = Some(bounds);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let track_y = bounds.top() + px((self.row_h - self.track_h) / 2.0);
        let knob_y = bounds.top() + px((self.row_h - self.knob_size) / 2.0);
        let track_w: f32 = bounds.size.width.into();
        let fill_w = px(self.pct * (track_w - self.knob_size));
        let knob_x = bounds.left() + px(self.pct * (track_w - self.knob_size));

        // Track — sharp-cornered brutalist bar with a thick
        // border instead of the default renderer's pill shape.
        let track_bounds = Bounds::new(
            point(bounds.left(), track_y),
            size(bounds.size.width, px(self.track_h)),
        );
        window.paint_quad(PaintQuad {
            bounds: track_bounds,
            corner_radii: Corners::all(px(0.0)),
            background: self.track_bg.into(),
            border_color: self.border,
            border_widths: GpuiEdges::all(px(self.border_w)),
            border_style: BorderStyle::Solid,
        });

        // Fill — sharp-cornered, painted up to (but not over)
        // the knob so the knob's border isn't covered.
        let fill_bounds = Bounds::new(
            point(bounds.left(), track_y),
            size(fill_w, px(self.track_h)),
        );
        window.paint_quad(PaintQuad {
            bounds: fill_bounds,
            corner_radii: Corners::all(px(0.0)),
            background: self.fill_bg.into(),
            border_color: hsla(0., 0., 0., 0.),
            border_widths: GpuiEdges::default(),
            border_style: BorderStyle::default(),
        });

        // Knob — square (no rounding) with a thick brutalist
        // border on top of the track.
        let knob_bounds = Bounds::new(
            point(knob_x, knob_y),
            size(px(self.knob_size), px(self.knob_size)),
        );
        window.paint_quad(PaintQuad {
            bounds: knob_bounds,
            corner_radii: Corners::all(px(0.0)),
            background: self.knob_bg.into(),
            border_color: self.border,
            border_widths: GpuiEdges::all(px(self.border_w)),
            border_style: BorderStyle::Solid,
        });
    }
}
