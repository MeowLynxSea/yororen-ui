//! Meta-crate for yororen-ui.
//!
//! Re-exports the three layers:
//!
//! - [`yororen_ui_core`] — headless primitives + i18n + a11y + rtl + animation + assets.
//! - [`yororen_ui_default_renderer`] — 54 component renderers + Theme (JSON-backed) + bundled themes.
//!
//! Plus the bundled locale catalogs (`en`, `zh-CN`, `ar`).
//!
//! Themes ship as JSON files in the renderer crate; no separate
//! theme package is needed. See `yororen_ui_default_renderer::themes`
//! for the bundled `system_light`, `system_dark`, etc. loaders.

pub use yororen_ui_core::renderer::spec::{BorderSpec, Edges, IconPosition, ShadowSpec};
pub use yororen_ui_core::renderer::{
    BuiltinVariantKey, ButtonVariant, GlobalVariantRegistry, RendererContext, RendererMarker,
    RendererRegistry, TokenVariantStyle, VariantKey, VariantRegistry, VariantState, VariantStyle,
    markers,
};
/// Re-export of the core `theme` module so user code can write
/// `use yororen_ui::theme::ActiveTheme;`.
pub use yororen_ui_core::theme;
pub use yororen_ui_core::theme::{ActiveTheme, GlobalTheme, Theme};
pub use yororen_ui_core::{a11y, animation, assets, headless, i18n, notification, rtl};
/// Translation helper macros.
pub use yororen_ui_core::{t, t_named};
pub use yororen_ui_default_renderer as renderer;
/// `ActionVariantKind` is re-exported from the renderer
/// (the canonical home in v0.3). Also reachable via
/// `yororen_ui::renderer::ActionVariantKind`.
pub use yororen_ui_default_renderer::renderers::button::ActionVariantKind;

#[cfg(feature = "brutalism")]
pub use yororen_ui_brutalism_renderer as brutalism_renderer;

#[cfg(feature = "winui")]
pub use yororen_ui_winui_renderer as winui_renderer;

#[cfg(feature = "xml")]
pub use yororen_ui_xml as xml;
#[cfg(feature = "xml")]
pub use yororen_ui_xml::register_xml_component;
#[cfg(feature = "xml")]
pub use yororen_ui_xml::runtime::ComponentDescriptor;
#[cfg(feature = "xml")]
pub use yororen_ui_xml_macro::{bind, xml, xml_file};

// When the `xml` feature is disabled (e.g. `default-features = false`),
// the real proc-macros and runtime types are unavailable. Provide
// local shims so that the error message tells the user exactly how
// to enable XML support instead of a generic "cannot find macro".
#[cfg(not(feature = "xml"))]
pub mod xml {
    #![doc = "The `xml` module requires the `xml` feature. Enable it with `features = [\"xml\"]` in Cargo.toml."]
}

#[cfg(not(feature = "xml"))]
#[macro_export]
macro_rules! xml {
    ($($tt:tt)*) => {
        ::core::compile_error!(
            "the `xml` feature is disabled; enable `features = [\"xml\"]` in Cargo.toml to use the `xml!` macro"
        )
    };
}

#[cfg(not(feature = "xml"))]
#[macro_export]
macro_rules! xml_file {
    ($($tt:tt)*) => {
        ::core::compile_error!(
            "the `xml` feature is disabled; enable `features = [\"xml\"]` in Cargo.toml to use the `xml_file!` macro"
        )
    };
}

#[cfg(not(feature = "xml"))]
#[macro_export]
macro_rules! register_xml_component {
    ($($tt:tt)*) => {
        ::core::compile_error!(
            "the `xml` feature is disabled; enable `features = [\"xml\"]` in Cargo.toml to use `register_xml_component!`"
        )
    };
}

pub use yororen_ui_locale_ar as locale_ar;
pub use yororen_ui_locale_en as locale_en;
pub use yororen_ui_locale_zh_cn as locale_zh_cn;

/// Convenience helpers for installing the bundled locales and layering
/// application-specific translations on top.
pub mod locale;
