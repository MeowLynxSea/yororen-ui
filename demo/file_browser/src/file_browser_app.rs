//! yororen-ui Root Component Pattern
//!
//! This file demonstrates the **standard pattern** for building the root component
//! in any yororen-ui application.
//!
//! ## Root Component Responsibilities
//!
//! A root component (the one passed to `cx.open_window`) typically:
//! 1. Implements the `Render` trait from gpui
//! 2. Reads global state via `cx.global::<T>()`
//! 3. Derives UI state from global state (filtering, sorting, etc.)
//! 4. Renders child components
//! 5. Handles global notifications
//!
//! ## Render Trait Pattern
//!
//! ```
//! impl Render for MyApp {
//!     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         // Read global state
//!         let state = cx.global::<MyState>();
//!         // Derive UI state
//!         let filtered_data = state.items.lock().unwrap().filter(...);
//!         // Build UI
//!         div().children(...)
//!     }
//! }
//! ```
//!
//! ## Using This Pattern
//!
//! Copy this structure for your yororen-ui app's root component.

use gpui::{AnyElement, Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::component::divider;
use yororen_ui::theme::ActiveTheme;

use crate::components;
use crate::scan;
use crate::state::FileBrowserState;

/// Root component - the entry point for your application's UI tree
///
/// This is the component passed to `cx.open_window()` in main().
/// It serves as the parent for all other components in the application.
pub struct FileBrowserApp;

impl FileBrowserApp {
    /// Initializes the root component
    ///
    /// IMPORTANT: Store your entity_id in global state for notification purposes.
    /// This allows other components to trigger re-renders of this component.
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

/// Render trait - the core of gpui component system
///
/// This is called by gpui when:
/// - The component is first displayed
/// - `cx.notify(entity_id)` is called
/// - Global state changes that this component depends on
impl Render for FileBrowserApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Step 1: Read global state
        let state = cx.global::<FileBrowserState>();
        let theme = cx.theme().clone();

        // Step 2: Read model fields (no locks)
        let model = state.model.read(cx);
        let root = model.root.clone();
        let selected_path = model.selected_path.clone();
        let context_path = model.context_path.clone();
        let clipboard = model.clipboard.clone();
        let menu_open = model.menu_open;
        let menu_position = model.menu_position;
        let tree_nodes = model.tree_nodes.clone();
        let is_scanning = model.is_scanning;

        // Step 3: Trigger initial scan if tree is empty and not already scanning
        if tree_nodes.is_empty() && !is_scanning {
            let root = root.clone();
            window
                .spawn(cx, async move |cx| {
                    let _ = cx.update(|window, cx| scan::start_scan(root, window, cx));
                })
                .detach();
        }

        // Step 4: Build UI tree using fluent builder pattern

        // Render child components
        let header = components::header::FileBrowserHeader::render(&root);
        let details = components::details::FileBrowserDetails::render(&selected_path, &clipboard);

        let tree_panel = components::tree_panel::FileBrowserTreePanel::render(
            &theme,
            root.clone(),
            tree_nodes,
            is_scanning,
        );

        // Conditional rendering: show context menu when triggered
        let context_menu: Option<AnyElement> = if menu_open {
            Some(components::context_menu::render(
                &theme,
                menu_position,
                context_path,
                clipboard.clone(),
            ))
        } else {
            None
        };

        // Build the main UI structure
        let mut root_view = div().size_full().bg(theme.surface.base).relative();
        root_view = root_view.child(
            div()
                .size_full()
                .p(px(20.))
                .flex()
                .flex_col()
                .min_h_0()
                .gap(px(12.))
                .child(header)
                .child(details)
                .child(divider())
                .child(tree_panel),
        );

        // Add context menu overlay if open
        if let Some(context_menu) = context_menu {
            root_view = root_view.child(context_menu);
        }

        root_view
    }
}
