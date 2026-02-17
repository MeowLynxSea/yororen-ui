//! yororen-ui Component Architecture
//!
//! This module demonstrates how to structure UI components in yororen-ui applications.
//!
//! ## yororen-ui Component Pattern
//!
//! All yororen-ui components follow this pattern:
//!
//! ```ignore
//! pub struct MyComponent;
//!
//! impl MyComponent {
//!     pub fn render(props: Props) -> impl IntoElement {
//!         // Build UI using fluent builder
//!         div().child(...)
//!     }
//! }
//! ```
//!
//! ## Key Conventions
//!
//! 1. **Stateless Functions**: Components are typically functions, not structs with state
//! 2. **Props as Arguments**: Pass data as function parameters
//! 3. **Event Handlers**: Modify global state and call `cx.notify()`
//! 4. **Fluent Builder**: Use `.child()`, `.on_click()`, etc. to build UI
//!
//! ## Common Event Handlers
//!
//! - `on_click` - Button/link clicks
//! - `on_change` - Input/text changes
//! - `on_toggle` - Checkbox/switch changes
//! - `on_close` - Modal/dialog close
//!
//! ## Using yororen-ui Components
//!
//! Import and use components from `yororen_ui::component`:
//! ```ignore
//! use yororen_ui::component::{button, text_input, modal, ...};
//! ```

pub mod todo_form;
pub mod todo_header;
pub mod todo_item;
pub mod todo_modal;
pub mod todo_toolbar;
