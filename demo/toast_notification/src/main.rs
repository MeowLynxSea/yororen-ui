//! Toast Notification Demo
//!
//! This demo showcases the Toast notification component with various styles.
//! It demonstrates the standard patterns for using Toast and NotificationCenter.
//!
//! ## Key Patterns (For yororen-ui Developers)
//!
//! ### 1. Toast Component Usage
//! Toast provides lightweight feedback messages:
//!   - `toast().message("...")` - Set the toast message content
//!   - `toast().kind(ToastKind::...)` - Set toast variant (Success, Warning, Error, Info, Neutral)
//!   - `toast().wrap(true)` - Enable text wrapping for long messages
//!   - `toast().width(px(...))` / `toast().max_width(px(...))` - Customize dimensions
//!   - `toast().icon(false)` - Hide the icon
//!   - `toast().content(...)` - Render custom content inside toast
//!
//! ### 2. NotificationCenter Usage
//! For queued, interactive notifications:
//!   - `cx.set_global(NotificationCenter::new())` - Initialize notification center
//!   - `cx.global::<NotificationCenter>()` - Access the center instance
//!   - `center.notify(Notification::new("..."), cx)` - Queue a notification
//!   - `center.notify_with_callbacks(...)` - Queue with action callbacks
//!
//! ### 3. Notification Options
//!   - `.kind(ToastKind::...)` - Visual style variant
//!   - `.sticky(true)` - Keep notification visible until dismissed
//!   - `.dismiss(DismissStrategy::Manual)` - Disable auto-dismiss
//!   - `.action_label("...")` - Add action button label
//!   - `.payload(...)` - Attach arbitrary JSON data
//!
//! ## Usage
//! Run this demo to explore Toast notifications:
//! ```bash
//! cd demo/toast_notification && cargo run
//! ```

mod toast_demo_app;

// Gpui framework imports
// Core types for building gpui applications
use gpui::{App, AppContext, Application, WindowOptions, px, size};

// yororen-ui framework imports
// These are the foundation of every yororen-ui application
use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::i18n::{I18n, Locale};
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

        // RECOMMENDED: Set up i18n with embedded translations
        cx.set_global(I18n::with_embedded(Locale::new("en").unwrap()));

        // Step 3: Create main window
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(500.0), px(400.0)),
                cx,
            ))),
            ..Default::default()
        };

        // Open window and render root component
        // cx.new() creates a new entity with the given closure as its impl
        cx.open_window(options, |_, cx| {
            cx.new(|cx| toast_demo_app::ToastDemoApp::new(cx))
        }).unwrap();
    });
}
