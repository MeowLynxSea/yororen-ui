//! Renderer switching: install either the default, brutalism or the
//! WinUI renderer (with one of the two bundled themes for each).
//!
//! ## Why this is "per-render install"
//!
//! `RendererRegistry::register_arc` is `HashMap.insert` (`last-wins`),
//! and `theme::install` overwrites the global `GlobalTheme`. So
//! `default_renderer::install_with`, `brutalism_renderer::install_with`
//! and `winui_renderer::install_with` can be called any number of
//! times — re-calling is how the gallery hot-swaps the look without
//! restarting. The gallery calls `install_renderer` at the top of
//! every `Render::render`, so the window always reflects the latest
//! toolbar click.
//!
//! See:
//! - `yororen-ui-core/src/renderer/registry.rs:97-99` (last-wins)
//! - `yororen-ui-default-renderer/src/themes.rs:77-80` (install_with)
//! - `yororen-ui-brutalism-renderer/src/lib.rs:77-80` (install_with)
//! - `yororen-ui-winui-renderer/src/themes.rs:75-85` (install_with)

use gpui::App;

use yororen_ui::theme::Theme;
use yororen_ui_brutalism_renderer as brutalism_renderer;
use yororen_ui_default_renderer as default_renderer;
use yororen_ui_winui_renderer as winui_renderer;

/// Which renderer to install. Switched at runtime by the toolbar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RendererKind {
    /// Token-based default renderer (rounded corners, soft shadows,
    /// neutral palette, system-ui font).
    #[default]
    Default,
    /// Neo-brutalism renderer (0-radius corners, thick black borders,
    /// hard offset shadows, monospace font, high-contrast palette).
    Brutalism,
    /// WinUI-style renderer (Segoe UI palette, 4px controls, subtle
    /// borders, accent-focus inputs).
    WinUI,
}

/// Light or dark palette. The bundled JSON for each is keyed by
/// `system-light` / `system-dark` for the default renderer,
/// `brutalism-light` / `brutalism-dark` for the brutalism renderer,
/// and `winui-light` / `winui-dark` for the WinUI renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DarkMode {
    #[default]
    Light,
    Dark,
}

const BRUTAL_LIGHT_JSON: &str =
    include_str!("../../../yororen-ui-brutalism-renderer/themes/brutalism-light.json");
const BRUTAL_DARK_JSON: &str =
    include_str!("../../../yororen-ui-brutalism-renderer/themes/brutalism-dark.json");

fn brutalism_theme(dark: DarkMode) -> Theme {
    let json = if matches!(dark, DarkMode::Dark) {
        BRUTAL_DARK_JSON
    } else {
        BRUTAL_LIGHT_JSON
    };
    Theme::from_json(json).expect("bundled brutalism theme JSON is valid")
}

/// Install the chosen renderer + bundled theme onto `cx`. Idempotent;
/// re-calling is the way the gallery swaps at runtime.
pub fn install_renderer(cx: &mut App, kind: RendererKind, dark: DarkMode) {
    match kind {
        RendererKind::Default => {
            let theme = if matches!(dark, DarkMode::Dark) {
                default_renderer::themes::system_dark()
            } else {
                default_renderer::themes::system_light()
            };
            default_renderer::install_with(cx, theme);
        }
        RendererKind::Brutalism => {
            let theme = brutalism_theme(dark);
            brutalism_renderer::install_with(cx, theme);
        }
        RendererKind::WinUI => {
            let theme = if matches!(dark, DarkMode::Dark) {
                winui_renderer::themes::winui_dark()
            } else {
                winui_renderer::themes::winui_light()
            };
            winui_renderer::install_with(cx, theme);
        }
    }
}