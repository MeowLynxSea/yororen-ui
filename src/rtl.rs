//! RTL (right-to-left) layout helpers.
//!
//! GPUI itself doesn't provide a global layout direction flag for style resolution,
//! so this module provides small helpers to flip common "start/end" concepts.

use gpui::{Length, Pixels, relative};

use crate::i18n::TextDirection;

/// Convert a logical *start* alignment into a concrete GPUI `TextAlign`.
pub fn text_align_start(direction: TextDirection) -> gpui::TextAlign {
    match direction {
        TextDirection::Ltr => gpui::TextAlign::Left,
        TextDirection::Rtl => gpui::TextAlign::Right,
    }
}

/// Convert a logical *end* alignment into a concrete GPUI `TextAlign`.
pub fn text_align_end(direction: TextDirection) -> gpui::TextAlign {
    match direction {
        TextDirection::Ltr => gpui::TextAlign::Right,
        TextDirection::Rtl => gpui::TextAlign::Left,
    }
}

/// Map a logical arrow direction for UI affordances.
///
/// In RTL, "back" should generally point right and "forward" should point left.
pub fn flip_left_right<T>(direction: TextDirection, left: T, right: T) -> T {
    match direction {
        TextDirection::Ltr => left,
        TextDirection::Rtl => right,
    }
}

/// Return `row` or `row-reverse` based on layout direction.
pub fn is_row_reverse(direction: TextDirection) -> bool {
    direction.is_rtl()
}

/// Place an absolutely positioned element at the logical start.
pub fn place_start(style: &mut gpui::StyleRefinement, direction: TextDirection, value: Pixels) {
    if direction.is_rtl() {
        style.inset.right = Some(Length::from(value));
    } else {
        style.inset.left = Some(Length::from(value));
    }
}

/// Place an absolutely positioned element at the logical start with 0px.
pub fn place_start_0(style: &mut gpui::StyleRefinement, direction: TextDirection) {
    if direction.is_rtl() {
        style.inset.right = Some(relative(0.).into());
    } else {
        style.inset.left = Some(relative(0.).into());
    }
}

/// Place an absolutely positioned element at the logical end.
pub fn place_end(style: &mut gpui::StyleRefinement, direction: TextDirection, value: Pixels) {
    if direction.is_rtl() {
        style.inset.left = Some(Length::from(value));
    } else {
        style.inset.right = Some(Length::from(value));
    }
}

/// Place an absolutely positioned element at the logical end with 0px.
pub fn place_end_0(style: &mut gpui::StyleRefinement, direction: TextDirection) {
    if direction.is_rtl() {
        style.inset.left = Some(relative(0.).into());
    } else {
        style.inset.right = Some(relative(0.).into());
    }
}
