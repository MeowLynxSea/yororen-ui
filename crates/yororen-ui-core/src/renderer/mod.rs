//! Core renderer dispatch — the trait-object store that lets
//! `cx.renderer_arc::<T, R>()` return a typed renderer reference
//! at render time.
//!
//! See [`RendererRegistry`] for storage and the
//! `RendererContext` sugar trait for the `cx.register_renderer_arc`
//! / `cx.renderer_arc` API.

mod context;
pub mod markers;
mod registry;
pub mod spec;
pub mod variant;

pub use context::{RendererContext, init_renderer_registry};
pub use markers::*;
pub use registry::{RendererMarker, RendererRegistry};
pub use spec::{BorderSpec, Edges, IconPosition, RenderState, ShadowSpec};
pub use variant::{
    ActionVariantKind, BuiltinVariantKey, ButtonVariant, GlobalVariantRegistry, TokenVariantStyle,
    VariantKey, VariantRegistry, VariantState, VariantStyle, variant_compose,
};

pub mod avatar;
pub mod badge;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
pub mod combo_box;
pub mod disclosure;
pub mod divider;
pub mod dropdown_menu;
pub mod empty_state;
pub mod file_path_input;
pub mod focus_ring;
pub mod form;
pub mod form_field;
pub mod grid_view;
pub mod heading;
pub mod icon;
pub mod icon_button;
pub mod image;
pub mod keybinding_display;
pub mod keybinding_input;
pub mod label;
pub mod list_item;
pub mod listbox;
pub mod menu;
pub mod modal;
pub mod notification;
pub mod number_input;
pub mod overlay;
pub mod panel;
pub mod password_input;
pub mod popover;
pub mod progress;
pub mod radio;
pub mod radio_group;
pub mod search_input;
pub mod select;
pub mod shortcut_hint;
pub mod skeleton;
pub mod slider;
pub mod spacer;
pub mod split_button;
pub mod switch;
pub mod table;
pub mod tag;
pub mod text;
pub mod text_area;
pub mod text_input;
pub mod toast;
pub mod toggle_button;
pub mod tooltip;
pub mod tree;
pub mod tree_item;
pub mod uniform_virtual_list;
pub mod virtual_list;
