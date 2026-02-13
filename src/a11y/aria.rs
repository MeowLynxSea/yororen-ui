//! ARIA (Accessible Rich Internet Applications) definitions and helpers.
//!
//! This module provides types and utilities for setting ARIA attributes
//! on elements to improve screen reader compatibility.

// Note: This module provides ARIA attribute definitions and builders.
// The actual attribute application depends on gpui's element API support.

/// ARIA roles for semantic element identification.
///
/// These roles map to WAI-ARIA role attributes and help screen readers
/// understand the purpose of interactive elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Identifies a dialog window.
    Dialog,
    /// Identifies a combobox input.
    Combobox,
    /// Identifies a listbox (dropdown list).
    Listbox,
    /// Identifies an option in a listbox.
    Option,
    /// Identifies a menu container.
    Menu,
    /// Identifies a menu item.
    MenuItem,
    /// Identifies a checkbox input.
    Checkbox,
    /// Identifies a radio input.
    Radio,
    /// Identifies a switch control.
    Switch,
    /// Identifies a button element.
    Button,
    /// Identifies a textbox input.
    Textbox,
    /// Identifies a grid container.
    Grid,
    /// Identifies a row in a grid.
    Row,
    /// Identifies a cell in a grid.
    Cell,
    /// Identifies a tree container.
    Tree,
    /// Identifies a tree item.
    TreeItem,
    /// Identifies a tab panel.
    TabPanel,
    /// Identifies a tab element.
    Tab,
    /// Identifies a list of tabs.
    TabList,
    /// Identifies an application region.
    Application,
    /// Identifies a landmark region.
    Region,
    /// Identifies a complementary region.
    Complementary,
    /// Identifies a navigation region.
    Navigation,
    /// Identifies a toolbar container.
    Toolbar,
    /// Identifies a tooltip element.
    Tooltip,
    /// Identifies a status message.
    Status,
    /// Identifies an alert message.
    Alert,
    /// Identifies a progressbar element.
    Progressbar,
    /// Identifies a slider control.
    Slider,
    /// Identifies a scrollbar element.
    Scrollbar,
    /// Identifies generic content.
    Group,
    /// Identifies a generic presentation (no semantics).
    Presentation,
}

impl Role {
    /// Returns the ARIA role string value.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Dialog => "dialog",
            Role::Combobox => "combobox",
            Role::Listbox => "listbox",
            Role::Option => "option",
            Role::Menu => "menu",
            Role::MenuItem => "menuitem",
            Role::Checkbox => "checkbox",
            Role::Radio => "radio",
            Role::Switch => "switch",
            Role::Button => "button",
            Role::Textbox => "textbox",
            Role::Grid => "grid",
            Role::Row => "row",
            Role::Cell => "cell",
            Role::Tree => "tree",
            Role::TreeItem => "treeitem",
            Role::TabPanel => "tabpanel",
            Role::Tab => "tab",
            Role::TabList => "tablist",
            Role::Application => "application",
            Role::Region => "region",
            Role::Complementary => "complementary",
            Role::Navigation => "navigation",
            Role::Toolbar => "toolbar",
            Role::Tooltip => "tooltip",
            Role::Status => "status",
            Role::Alert => "alert",
            Role::Progressbar => "progressbar",
            Role::Slider => "slider",
            Role::Scrollbar => "scrollbar",
            Role::Group => "group",
            Role::Presentation => "presentation",
        }
    }
}

/// Builder for ARIA attributes.
pub struct AriaAttrs {
    role: Option<Role>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    aria_describedby: Option<String>,
    aria_hidden: Option<bool>,
    aria_disabled: Option<bool>,
    aria_readonly: Option<bool>,
    aria_required: Option<bool>,
    aria_expanded: Option<bool>,
    aria_checked: Option<bool>,
    aria_selected: Option<bool>,
    aria_activedescendant: Option<String>,
    aria_autocomplete: Option<String>,
    aria_controls: Option<String>,
    aria_owns: Option<String>,
    aria_haspopup: Option<String>,
    aria_modal: Option<bool>,
    aria_orientation: Option<String>,
    aria_valuemin: Option<f64>,
    aria_valuemax: Option<f64>,
    aria_valuenow: Option<f64>,
    aria_valuetext: Option<String>,
    aria_live: Option<String>,
    aria_level: Option<i32>,
    tab_index: Option<i32>,
}

impl AriaAttrs {
    /// Creates a new AriaAttrs builder.
    pub fn new() -> Self {
        Self {
            role: None,
            aria_label: None,
            aria_labelledby: None,
            aria_describedby: None,
            aria_hidden: None,
            aria_disabled: None,
            aria_readonly: None,
            aria_required: None,
            aria_expanded: None,
            aria_checked: None,
            aria_selected: None,
            aria_activedescendant: None,
            aria_autocomplete: None,
            aria_controls: None,
            aria_owns: None,
            aria_haspopup: None,
            aria_modal: None,
            aria_orientation: None,
            aria_valuemin: None,
            aria_valuemax: None,
            aria_valuenow: None,
            aria_valuetext: None,
            aria_live: None,
            aria_level: None,
            tab_index: None,
        }
    }

    /// Sets the ARIA role attribute.
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the aria-label attribute (short description).
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Sets the aria-labelledby attribute (ID reference to labeled element).
    pub fn aria_labelledby(mut self, id: impl Into<String>) -> Self {
        self.aria_labelledby = Some(id.into());
        self
    }

    /// Sets the aria-describedby attribute (ID reference to description).
    pub fn aria_describedby(mut self, id: impl Into<String>) -> Self {
        self.aria_describedby = Some(id.into());
        self
    }

    /// Sets the aria-hidden attribute.
    pub fn aria_hidden(mut self, hidden: bool) -> Self {
        self.aria_hidden = Some(hidden);
        self
    }

    /// Sets the aria-disabled attribute.
    pub fn aria_disabled(mut self, disabled: bool) -> Self {
        self.aria_disabled = Some(disabled);
        self
    }

    /// Sets the aria-readonly attribute.
    pub fn aria_readonly(mut self, readonly: bool) -> Self {
        self.aria_readonly = Some(readonly);
        self
    }

    /// Sets the aria-required attribute.
    pub fn aria_required(mut self, required: bool) -> Self {
        self.aria_required = Some(required);
        self
    }

    /// Sets the aria-expanded attribute.
    pub fn aria_expanded(mut self, expanded: bool) -> Self {
        self.aria_expanded = Some(expanded);
        self
    }

    /// Sets the aria-checked attribute.
    pub fn aria_checked(mut self, checked: bool) -> Self {
        self.aria_checked = Some(checked);
        self
    }

    /// Sets the aria-selected attribute.
    pub fn aria_selected(mut self, selected: bool) -> Self {
        self.aria_selected = Some(selected);
        self
    }

    /// Sets the aria-activedescendant attribute (ID of currently active descendant).
    pub fn aria_activedescendant(mut self, id: impl Into<String>) -> Self {
        self.aria_activedescendant = Some(id.into());
        self
    }

    /// Sets the aria-autocomplete attribute.
    pub fn aria_autocomplete(mut self, value: &str) -> Self {
        self.aria_autocomplete = Some(value.to_string());
        self
    }

    /// Sets the aria-controls attribute (ID reference to controlled element).
    pub fn aria_controls(mut self, id: impl Into<String>) -> Self {
        self.aria_controls = Some(id.into());
        self
    }

    /// Sets the aria-owns attribute (ID reference to owned element).
    pub fn aria_owns(mut self, id: impl Into<String>) -> Self {
        self.aria_owns = Some(id.into());
        self
    }

    /// Sets the aria-haspopup attribute.
    pub fn aria_haspopup(mut self, value: &str) -> Self {
        self.aria_haspopup = Some(value.to_string());
        self
    }

    /// Sets the aria-modal attribute.
    pub fn aria_modal(mut self, modal: bool) -> Self {
        self.aria_modal = Some(modal);
        self
    }

    /// Sets the aria-orientation attribute.
    pub fn aria_orientation(mut self, orientation: &str) -> Self {
        self.aria_orientation = Some(orientation.to_string());
        self
    }

    /// Sets the aria-valuemin attribute.
    pub fn aria_valuemin(mut self, value: f64) -> Self {
        self.aria_valuemin = Some(value);
        self
    }

    /// Sets the aria-valuemax attribute.
    pub fn aria_valuemax(mut self, value: f64) -> Self {
        self.aria_valuemax = Some(value);
        self
    }

    /// Sets the aria-valuenow attribute.
    pub fn aria_valuenow(mut self, value: f64) -> Self {
        self.aria_valuenow = Some(value);
        self
    }

    /// Sets the aria-valuetext attribute.
    pub fn aria_valuetext(mut self, text: impl Into<String>) -> Self {
        self.aria_valuetext = Some(text.into());
        self
    }

    /// Sets the aria-live region attribute.
    pub fn aria_live(mut self, value: &str) -> Self {
        self.aria_live = Some(value.to_string());
        self
    }

    /// Sets the aria-level attribute (for heading hierarchy).
    pub fn aria_level(mut self, level: i32) -> Self {
        self.aria_level = Some(level);
        self
    }

    /// Sets the tabindex attribute.
    pub fn tab_index(mut self, index: i32) -> Self {
        self.tab_index = Some(index);
        self
    }

    /// Builds the ARIA attributes into a vector of (key, value) pairs.
    pub fn build(self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();

        if let Some(role) = self.role {
            attrs.push(("role".to_string(), role.as_str().to_string()));
        }

        if let Some(label) = self.aria_label {
            attrs.push(("aria-label".to_string(), label));
        }

        if let Some(id) = self.aria_labelledby {
            attrs.push(("aria-labelledby".to_string(), id));
        }

        if let Some(id) = self.aria_describedby {
            attrs.push(("aria-describedby".to_string(), id));
        }

        if let Some(hidden) = self.aria_hidden {
            attrs.push(("aria-hidden".to_string(), hidden.to_string()));
        }

        if let Some(disabled) = self.aria_disabled {
            attrs.push(("aria-disabled".to_string(), disabled.to_string()));
        }

        if let Some(readonly) = self.aria_readonly {
            attrs.push(("aria-readonly".to_string(), readonly.to_string()));
        }

        if let Some(required) = self.aria_required {
            attrs.push(("aria-required".to_string(), required.to_string()));
        }

        if let Some(expanded) = self.aria_expanded {
            attrs.push(("aria-expanded".to_string(), expanded.to_string()));
        }

        if let Some(checked) = self.aria_checked {
            attrs.push(("aria-checked".to_string(), checked.to_string()));
        }

        if let Some(selected) = self.aria_selected {
            attrs.push(("aria-selected".to_string(), selected.to_string()));
        }

        if let Some(id) = self.aria_activedescendant {
            attrs.push(("aria-activedescendant".to_string(), id));
        }

        if let Some(value) = self.aria_autocomplete {
            attrs.push(("aria-autocomplete".to_string(), value));
        }

        if let Some(id) = self.aria_controls {
            attrs.push(("aria-controls".to_string(), id));
        }

        if let Some(id) = self.aria_owns {
            attrs.push(("aria-owns".to_string(), id));
        }

        if let Some(value) = self.aria_haspopup {
            attrs.push(("aria-haspopup".to_string(), value));
        }

        if let Some(modal) = self.aria_modal {
            attrs.push(("aria-modal".to_string(), modal.to_string()));
        }

        if let Some(orientation) = self.aria_orientation {
            attrs.push(("aria-orientation".to_string(), orientation));
        }

        if let Some(value) = self.aria_valuemin {
            attrs.push(("aria-valuemin".to_string(), value.to_string()));
        }

        if let Some(value) = self.aria_valuemax {
            attrs.push(("aria-valuemax".to_string(), value.to_string()));
        }

        if let Some(value) = self.aria_valuenow {
            attrs.push(("aria-valuenow".to_string(), value.to_string()));
        }

        if let Some(text) = self.aria_valuetext {
            attrs.push(("aria-valuetext".to_string(), text));
        }

        if let Some(value) = self.aria_live {
            attrs.push(("aria-live".to_string(), value));
        }

        if let Some(level) = self.aria_level {
            attrs.push(("aria-level".to_string(), level.to_string()));
        }

        if let Some(index) = self.tab_index {
            attrs.push(("tabindex".to_string(), index.to_string()));
        }

        attrs
    }
}

impl Default for AriaAttrs {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new AriaAttrs builder.
pub fn aria() -> AriaAttrs {
    AriaAttrs::new()
}

/// Predefined ARIA attribute sets for common component patterns.

/// Attributes for a dialog/modal element.
pub mod dialog {
    /// Creates aria attributes for a dialog.
    pub fn attrs(labelledby: Option<&str>, describedby: Option<&str>) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("role".to_string(), "dialog".to_string()),
            ("aria-modal".to_string(), "true".to_string()),
        ];

        if let Some(id) = labelledby {
            attrs.push(("aria-labelledby".to_string(), id.to_string()));
        }

        if let Some(id) = describedby {
            attrs.push(("aria-describedby".to_string(), id.to_string()));
        }

        attrs
    }
}

/// Attributes for a combobox element.
pub mod combobox {
    /// Creates aria attributes for a combobox.
    pub fn attrs(
        expanded: bool,
        activedescendant: Option<&str>,
        controls: Option<&str>,
    ) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("role".to_string(), "combobox".to_string()),
            ("aria-expanded".to_string(), expanded.to_string()),
            ("aria-haspopup".to_string(), "listbox".to_string()),
            ("aria-autocomplete".to_string(), "none".to_string()),
        ];

        if let Some(id) = activedescendant {
            attrs.push(("aria-activedescendant".to_string(), id.to_string()));
        }

        if let Some(id) = controls {
            attrs.push(("aria-controls".to_string(), id.to_string()));
        }

        attrs
    }
}

/// Attributes for a listbox option.
pub mod option {
    /// Creates aria attributes for a listbox option.
    pub fn attrs(selected: bool, disabled: bool) -> Vec<(String, String)> {
        vec![
            ("role".to_string(), "option".to_string()),
            ("aria-selected".to_string(), selected.to_string()),
            ("aria-disabled".to_string(), disabled.to_string()),
        ]
    }
}

/// Attributes for a menu item.
pub mod menuitem {
    /// Creates aria attributes for a menu item.
    pub fn attrs(disabled: bool) -> Vec<(String, String)> {
        vec![
            ("role".to_string(), "menuitem".to_string()),
            ("aria-disabled".to_string(), disabled.to_string()),
        ]
    }
}

/// Extension trait for adding role directly to elements.
pub trait RoleExt {
    /// Set the role attribute on this element.
    fn role(self, _role: Role) -> Self
    where
        Self: Sized,
    {
        self
    }
}

impl<E> RoleExt for E {}
