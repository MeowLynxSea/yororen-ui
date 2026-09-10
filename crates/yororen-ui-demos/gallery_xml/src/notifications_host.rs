//! Global toast / notification host for the gallery.
//!
//! Reads the global [`NotificationCenter`] and renders a stack of
//! floating cards anchored to the **top-right** of the window.
//! This replaces the previous "Notifications" section that drew
//! the same data as a scrollable list of cards inside the page
//! content.
//!
//! ## Why a custom host instead of `yororen-ui-default-renderer`'s
//! `notification_host`?
//!
//! The default-renderer ships a `notification_host.rs` file, but
//! it imports from a `crate::component` module that does not
//! exist (and it accesses strongly-typed `theme.tokens.*` fields
//! that the v0.3 JSON-backed `Theme` does not expose). The file
//! is dead code and is not part of the public API.
//!
//! Implementing the host here in the gallery:
//!
//! 1. Works with **both** renderers (default and brutalism) —
//!    colors are read with the path-based
//!    `cx.theme().get_color("...")` API that both bundled
//!    JSON themes already populate.
//! 2. Stays self-contained — does not require changes to the
//!    renderer crates.
//! 3. Keeps the toast/notification surface focused on what the
//!    gallery actually needs: a kind-colored card (background
//!    uses `status.<kind>.bg`) with bold title + message and a
//!    close (×) button on the title row.
//!
//! ## Paint order
//!
//! The host is wrapped in [`gpui::deferred(...).with_priority(3)`]
//! when added to the root by [`crate::gallery_app::GalleryApp`]'s
//! `Render`. Priority 3 places it above the modal scrim (priority
//! 2) and the popover / dropdown panels (priority 1) so toasts
//! always float on top of every other overlay, which is the
//! expected behavior of a global notification surface.
//!
//! ## Auto-dismiss
//!
//! The global [`NotificationCenter`] refuses to schedule its
//! auto-dismiss timer unless `register_host_window` has been
//! called. The gallery's `Render::render` calls
//! `center.register_host_window(window.window_handle())` on every
//! paint, so non-sticky toasts disappear after the configured
//! duration. Sticky notifications (created with
//! `Notification::sticky(true)`) are never auto-dismissed and
//! must be closed manually.

use gpui::{
    AnimationExt, Context, Hsla, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, deferred, div, hsla, px,
};

use yororen_ui::notification::center::{
    Notification, NotificationCenter, NotificationId, ToastKind,
};
use yororen_ui::theme::ActiveTheme;

use crate::view::GalleryApp;

/// The deferred-paint priority used by the global host.
///
/// Priority 3 keeps toasts above the modal scrim (priority 2) and
/// popover / dropdown panels (priority 1). Bump if a future
/// overlay needs to sit above notifications.
const HOST_PRIORITY: usize = 3;

/// Card width — chosen so a typical title + two lines of body
/// fit without truncating.
const TOAST_WIDTH: f32 = 320.0;

/// Outer padding from the window edge.
const EDGE_PADDING: f32 = 16.0;

/// Gap between stacked cards.
const STACK_GAP: f32 = 8.0;

/// Inner card padding.
const CARD_PADDING: f32 = 12.0;

/// Close (×) button side length. Small enough to stay out of the
/// way of the title, large enough to be a comfortable click
/// target.
const CLOSE_BUTTON_SIZE: f32 = 20.0;

/// How much darker / lighter the card border is compared to the
/// card background. Used so the card has a subtle outline that
/// reads against bright kind colors (the brutalism theme uses
/// saturated hues that would otherwise look "flat" against the
/// white page surface).
const BORDER_LIGHTEN_DELTA: f32 = 0.06;

/// Render the global notification host as a single element.
///
/// Callers are expected to wrap the returned value in
/// [`gpui::deferred`] (or pass it through a child slot that
/// does) so the host paints on top of the page content.
///
/// Returns the host scaffold even when the center has no items,
/// so the host is always renderable and never blocks layout.
pub fn render(cx: &mut Context<GalleryApp>) -> impl IntoElement {
    // Build the empty host scaffold up front — this also
    // establishes the fixed `Stateful<Div>` return type so the
    // "no center installed" branch below can return the same
    // type instead of forcing a unification to plain `Div`.
    let mut stack = div()
        .id("ui:notification-host")
        .absolute()
        .top_0()
        .right_0()
        .mt(px(EDGE_PADDING))
        .mr(px(EDGE_PADDING))
        .flex()
        .flex_col()
        .gap(px(STACK_GAP))
        .items_end();

    // Acquire the notification center. If it was never installed
    // (which would be a programmer error — `main.rs` always
    // sets the global), return the empty scaffold so the
    // window still composes.
    let Some(center) = cx.try_global::<NotificationCenter>() else {
        return stack;
    };
    let items: Vec<Notification> = center.items();

    for n in items.into_iter() {
        let id = n.id;
        let card = toast_card(cx, n, id);
        let distance = px(24.0);
        let animated = card.with_animation(
            ("ui:notification:enter", id.raw()),
            gpui::Animation::new(std::time::Duration::from_millis(200))
                .with_easing(yororen_ui::animation::ease_out_cubic),
            move |this, progress| {
                let eased = yororen_ui::animation::ease_out_cubic(progress);
                let translate: f32 = distance.into();
                let translate = translate * (1.0 - eased);
                // Negative margin-right shifts the card to the right
                // off-screen; as the animation progresses it slides
                // left into its final position.
                this.opacity(eased).mr(px(-translate))
            },
        );
        stack = stack.child(animated);
    }

    stack
}

/// Build a single toast card.
///
/// The card's **entire background** is the kind color
/// (`status.<kind>.bg`) — that color is the only visual cue for
/// the kind. There is no kind label, no "sticky" badge, and no
/// secondary status row. The bold title and the message sit on
/// the kind color with `kind_fg` as the text color, and the
/// close (×) button is on the same row as the title (right
/// side).
fn toast_card(
    cx: &mut Context<GalleryApp>,
    n: Notification,
    id: NotificationId,
) -> gpui::Stateful<gpui::Div> {
    // `n` is owned, so move title / message out instead of cloning.
    let Notification {
        title,
        message,
        kind,
        ..
    } = n;
    let kind_path = match kind {
        ToastKind::Success => "status.success",
        ToastKind::Warning => "status.warning",
        ToastKind::Error => "status.error",
        ToastKind::Info => "status.info",
        ToastKind::Neutral => "status.neutral",
    };
    let theme = cx.theme();
    // Both bundled themes (default + brutalism) populate every
    // `status.<kind>.bg` and `.fg` we use, so a missing key is
    // unusual. Fall back to a neutral surface tone + a default
    // text tone in that case.
    let kind_bg = theme
        .get_color(&format!("{kind_path}.bg"))
        .unwrap_or_else(|| {
            theme
                .get_color("surface.raised")
                .unwrap_or_else(default_surface)
        });
    let kind_fg = theme
        .get_color(&format!("{kind_path}.fg"))
        .unwrap_or_else(|| {
            theme
                .get_color("content.primary")
                .unwrap_or_else(default_text)
        });
    // The border is a slightly darkened / lightened version of
    // the kind background, so the card has a visible edge on
    // both pastel (default) and saturated (brutalism) palettes.
    let kind_border = adjust_lightness(kind_bg, BORDER_LIGHTEN_DELTA);

    // Close button. The `center` is cloned (an `Arc<Mutex<…>>`
    // underneath, so the clone is cheap) so the click closure
    // can outlive the current borrow on `cx`.
    let center = cx
        .try_global::<NotificationCenter>()
        .cloned()
        .unwrap_or_else(NotificationCenter::new);

    let close_fg = kind_fg;
    let close_bg_hover = adjust_lightness(kind_bg, -0.10);
    let close = div()
        .id(("ui:notification:dismiss", id.raw()))
        .flex()
        .items_center()
        .justify_center()
        .w(px(CLOSE_BUTTON_SIZE))
        .h(px(CLOSE_BUTTON_SIZE))
        .rounded_sm()
        .cursor_pointer()
        .text_color(close_fg)
        .opacity(0.55)
        .hover(move |this| this.bg(close_bg_hover).text_color(close_fg).opacity(1.0))
        .on_click(move |_ev, _window, cx| {
            center.dismiss(id, cx);
        })
        .child(div().text_sm().child("×"));

    // Title block on the left of the close button. Title (bold)
    // and message (regular) stack vertically, sharing a column
    // that flexes to fill the row. The close button sits at the
    // end of the same row, so it visually belongs to the title
    // area.
    let mut text_column = div().flex_1().min_w(px(0.)).flex().flex_col().gap(px(2.));
    if let Some(title) = title {
        text_column = text_column.child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(kind_fg)
                .child(title),
        );
    }
    text_column = text_column.child(div().text_sm().text_color(kind_fg).child(message));

    // Title row: [title column] [spacer] [close]. This is the
    // row the user asked the close button to share with the
    // title + subtitle (the "主标题+副标题" row in the spec).
    let title_row = div()
        .flex()
        .flex_row()
        .items_start()
        .justify_between()
        .gap(px(8.))
        .child(text_column)
        .child(close);

    div()
        .id(("ui:notification", id.raw()))
        .w(px(TOAST_WIDTH))
        .p(px(CARD_PADDING))
        .rounded(px(6.))
        // The border is the (slightly adjusted) kind color, so
        // it stays subtle against the card body and the page.
        .border_1()
        .border_color(kind_border)
        // The whole card body uses the kind color as its
        // background. This is the only visual cue for the
        // toast's category — no kind label is drawn.
        .bg(kind_bg)
        .child(title_row)
}

/// Default surface color used when a kind's bg is missing in
/// the active theme.
fn default_surface() -> Hsla {
    hsla(0.0, 0.0, 0.95, 1.0)
}

/// Default text color used when a kind's fg is missing in the
/// active theme.
fn default_text() -> Hsla {
    hsla(0.0, 0.0, 0.10, 1.0)
}

/// Return a version of `color` with lightness shifted by
/// `delta` (positive = lighter, negative = darker), clamped to
/// the valid HSL range.
fn adjust_lightness(color: Hsla, delta: f32) -> Hsla {
    let mut out = color;
    out.lightness = (color.lightness + delta).clamp(0.0, 1.0);
    out
}

/// Convenience: the host already wrapped in
/// [`gpui::deferred`] at the correct priority. Add this as a
/// child of the root element from `Render::render`.
pub fn deferred_host(cx: &mut Context<GalleryApp>) -> impl IntoElement {
    deferred(render(cx)).with_priority(HOST_PRIORITY)
}
