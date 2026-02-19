//! yororen-ui Component Architecture
//!
//! This module demonstrates how to structure UI components in yororen-ui applications.
//! Each component in this directory represents a reusable piece of UI with a specific responsibility.
//!
//! ## Component Organization Pattern
//!
//! In yororen-ui applications, components are typically organized as follows:
//!
//! ```ignore
//! pub struct MyComponent;
//!
//! impl MyComponent {
//!     pub fn render(props: Props) -> impl IntoElement {
//!         // Build UI using fluent builder pattern
//!         div().child(...)
//!     }
//! }
//! ```
//!
//! ## Component Design Conventions
//!
//! 1. **Stateless Components**: Most components are stateless structs with only a `render` method.
//!    State is managed globally and passed in as parameters.
//! 2. **Props as Arguments**: All data needed for rendering is passed as function parameters.
//! 3. **Event Handlers**: User interactions modify global state and trigger re-renders via `cx.notify()`.
//! 4. **Fluent Builder Pattern**: Use method chaining (`.child()`, `.on_click()`, `.class()`) to construct UI.
//! 5. **Single Responsibility**: Each component should focus on one UI concern (form, list item, header, etc.).
//!
//! ## Common Event Handlers
//!
//! yororen-ui components provide various event handlers for user interaction:
//!
//! - `on_click` - Fired when buttons or clickable elements are activated
//! - `on_change` - Fired when input values change (text input, combo box)
//! - `on_toggle` - Fired when toggleable elements change state (checkbox, switch)
//! - `on_close` - Fired when dismissible elements are closed (modal, dialog)
//!
//! ## Component Module Structure
//!
//! This demo includes the following components:
//!
//! - `todo_form.rs`: New task creation form with title input and category selection
//! - `todo_header.rs`: Application header with title and view mode toggle
//! - `todo_item.rs`: Individual todo item rendering (supports compact and detailed views)
//! - `todo_modal.rs`: Modal dialog for editing existing tasks
//! - `todo_toolbar.rs`: Search and filter controls for the todo list
//!
//! ## Using yororen-ui Components
//!
//! Components are imported from the `yororen_ui::component` module:
//! ```ignore
//! use yororen_ui::component::{button, text_input, modal, checkbox, ...};
//! ```

pub mod todo_form;
pub mod todo_header;
pub mod todo_item;
pub mod todo_modal;
pub mod todo_toolbar;
