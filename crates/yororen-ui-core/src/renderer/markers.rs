//! Component marker types — one per renderer.
//!
//! Each marker is a zero-sized type that identifies a
//! component in the global [`RendererRegistry`]. Renderer
//! crates call
//! `cx.register_renderer_arc::<markers::Button, dyn ButtonRenderer>(…)`
//! at install time; render-time code calls
//! `cx.renderer_arc::<markers::Button, dyn ButtonRenderer>()`
//! to fetch the registered renderer.
//!
//! Centralising the 56 markers here (rather than scattering
//! them across 56 headless modules) keeps the registry API
//! in one place and makes "is this component wired up?" a
//! single-file grep. Third-party components can add their own
//! marker types next to their headless module — these 56 are
//! the built-in set.

use super::registry::RendererMarker;

macro_rules! marker {
    ($($name:ident),* $(,)?) => {
        $(
            #[doc = concat!("Marker for the `", stringify!($name), "` component.")]
            #[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
            pub struct $name;
            impl RendererMarker for $name {}
        )*
    };
}

marker!(
    Button,
    ButtonGroup,
    IconButton,
    ToggleButton,
    Label,
    Heading,
    Divider,
    FocusRing,
    Badge,
    Tag,
    ProgressBar,
    Skeleton,
    Slider,
    Tooltip,
    Avatar,
    Switch,
    Checkbox,
    Radio,
    TextInput,
    TextArea,
    PasswordInput,
    NumberInput,
    FilePathInput,
    SearchInput,
    Select,
    ComboBox,
    Modal,
    Popover,
    DropdownMenu,
    Disclosure,
    Toast,
    Notification,
    Panel,
    Card,
    Form,
    GridView,
    ListItem,
    Listbox,
    TreeItem,
    KeybindingInput,
    SplitButton,
    EmptyState,
    Image,
    KeybindingDisplay,
    ShortcutHint,
    Icon,
    Text,
    Spacer,
    Overlay,
    Menu,
    FormField,
    RadioGroup,
    Table,
    Tree,
    VirtualList,
    UniformVirtualList,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::registry::RendererRegistry;

    #[test]
    fn each_marker_is_registerable() {
        // Smoke: every marker is distinct and 'static.
        let mut r = RendererRegistry::new();
        r.register_arc::<Button, dyn std::any::Any + Send + Sync>(
            std::sync::Arc::new(()) as std::sync::Arc<dyn std::any::Any + Send + Sync>
        );
        // We can't check much more without a real trait, but the
        // registration must not panic.
        drop(r);
    }
}
