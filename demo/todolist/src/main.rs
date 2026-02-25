//! yororen-ui Application Entry Point
//!
//! This module serves as the entry point and reference implementation for building yororen-ui applications.
//! It demonstrates the standard patterns and best practices used throughout the framework.
//!
//! ## Application Bootstrap Sequence
//!
//! Every yororen-ui application follows a specific initialization sequence:
//!
//! 1. **Create Application Instance**: Initialize the gpui Application with required assets
//!    ```rust
//!    Application::new().with_assets(UiAsset)
//!    ```
//!    The `UiAsset` provides built-in yororen-ui resources including icons, fonts, and other assets.
//!
//! 2. **Initialize Component Library**: Register all yororen-ui components
//!    ```rust
//!    component::init(cx)
//!    ```
//!    This must be called before using any yororen-ui components.
//!
//! 3. **Initialize Theme System**: Set up theming with system preference detection
//!    ```rust
//!    cx.set_global(GlobalTheme::new(cx.window_appearance()))
//!    ```
//!    The GlobalTheme automatically handles light/dark mode based on system preferences.
//!
//! 4. **Initialize Global State**: Set up application-specific state
//!    ```rust
//!    cx.set_global(YourAppState::default())
//!    ```
//!    This makes the state accessible to all components via `cx.global::<T>()`.
//!
//! 5. **Open Main Window**: Create and display the application window
//!    ```rust
//!    cx.open_window(options, |_, cx| {
//!        cx.new(|cx| YourApp::new(cx))
//!    })
//!    ```
//!
//! ## Recommended Module Structure
//!
//! A typical yororen-ui application should be organized as follows:
//!
//! - `main.rs` - Application entry point and framework initialization
//! - `state.rs` - Global state management using `gpui::Entity<T>`
//! - `*_app.rs` - Root component implementing the gpui `Render` trait
//! - `components/` - Directory containing reusable UI components
//! - `*.rs` - Domain model files with no UI dependencies
//!
//! ## Running the Demo
//!
//! To run this demo and explore yororen-ui components:
//! ```bash
//! cd demo/todolist && cargo run
//! ```
//!
//! The demo showcases several key yororen-ui components including:
//! checkbox, switch, tag, text_input, search_input, combo_box, modal, list_item,
//! icon_button, tooltip, button, and heading.

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
        // Try to change this to zh-CN or ar
        cx.set_global(i18n::load_demo_i18n(Locale::new("en").unwrap()).unwrap());

        // RECOMMENDED: Set up global application state
        // This demo stores mutable state in a gpui Entity.
        let todo_state = state::TodoState::new(cx);
        cx.set_global(todo_state);

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
