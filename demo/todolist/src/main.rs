//! yororen-ui Application Template
//!
//! This demo serves as a reference implementation for building yororen-ui applications.
//! It demonstrates the standard patterns and best practices used throughout yororen-ui.
//!
//! ## Key Patterns (For yororen-ui Developers)
//!
//! ### 1. Application Bootstrap
//! Every yororen-ui app follows this initialization sequence:
//!   - `Application::new().with_assets(UiAsset)` - Create app and load UI assets
//!   - `component::init(cx)` - Register all yororen-ui components
//!   - `cx.set_global(GlobalTheme::new(...))` - Initialize theme system
//!   - `cx.set_global(YourAppState::default())` - Set app-specific global state
//!   - `cx.open_window(...)` - Create the main window
//!
//! ### 2. Module Structure
//! A typical yororen-ui application should have:
//!   - `main.rs` - Application entry point and initialization
//!   - `state.rs` - Global state management (see Arc<Mutex<T>> pattern)
//!   - `*_app.rs` - Root component implementing `Render` trait
//!   - `components/` - Reusable UI components
//!   - `*.rs` - Domain models (no UI dependencies)
//!
//! ## Usage
//! Run this demo to explore yororen-ui components:
//! ```bash
//! cd demo/todolist && cargo run
//! ```

mod todo;
mod todo_app;
mod state;
mod components;
mod i18n;

// Gpui framework imports
// Core types for building gpui applications
use gpui::{AppContext, Application, App, WindowOptions, px, size};

// yororen-ui framework imports
// These are the foundation of every yororen-ui application
use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::i18n::Locale;
use yororen_ui::theme::GlobalTheme;

/// Standard yororen-ui application entry point
///
/// This pattern should be copied for all new yororen-ui applications.
/// Key steps:
/// 1. Create and configure the Application
/// 2. Initialize yororen-ui (components, theme, state)
/// 3. Open main window with root component
fn main() {
    // Step 1: Create application instance
    // UiAsset provides built-in yororen-ui resources (icons, fonts, etc.)
    let app = Application::new().with_assets(UiAsset);

    // Step 2: Initialize application
    app.run(|cx: &mut App| {
        // REQUIRED: Initialize yororen-ui component library
        // This must be called before using any yororen-ui components
        component::init(cx);

        // REQUIRED: Set up theming
        // GlobalTheme handles light/dark mode based on system preferences
        cx.set_global(GlobalTheme::new(cx.window_appearance()));

        // RECOMMENDED: Set up i18n.
        // This demo additionally loads `demo/todolist/locales/<locale>.json` to keep demo strings
        // out of the core library locales.
        cx.set_global(i18n::load_demo_i18n(Locale::new("ar").unwrap()).unwrap());

        // RECOMMENDED: Set up global application state
        // Use Global trait + Arc<Mutex<T>> for shared state
        cx.set_global(state::TodoState::default());

        // Step 3: Create main window
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(800.0), px(600.0)),
                cx,
            ))),
            ..Default::default()
        };

        // Open window and render root component
        // cx.new() creates a new entity with the given closure as its impl
        cx.open_window(options, |_, cx| {
            cx.new(|cx| todo_app::TodoApp::new(cx))
        }).unwrap();
    });
}
