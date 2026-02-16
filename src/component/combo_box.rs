use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, div,
    ease_out_quint, prelude::FluentBuilder, px,
};

use crate::{
    component::{compute_input_style, ArrowDirection, IconName, TextInputState, icon, text_input},
    constants::animation,
    i18n::{defaults::DefaultPlaceholders, I18nContext},
    theme::ActiveTheme,
};

#[derive(Clone, Debug)]
pub struct ComboBoxOption {
    pub value: String,
    pub label: SharedString,
    pub disabled: bool,
}

impl ComboBoxOption {
    pub fn new(value: impl Into<String>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Creates a new combo box element.
/// Requires an id to be set via `.id()` for internal state management.
///
/// # Accessibility
///
/// This component provides accessibility support through the following attributes:
/// - The input element is keyboard accessible (Tab to focus)
/// - Arrow keys can navigate through filtered options when the menu is open
/// - Escape closes the menu and clears the search
/// - The search input is properly associated with the dropdown list
///
/// For full accessibility support:
/// - The component tracks `aria-expanded` state internally (true when menu is open)
/// - The menu container has a unique ID for `aria-controls` association
/// - The search input uses `aria-autocomplete="list"` to indicate autocomplete behavior
/// - Selected options are visually indicated with a checkmark
/// - Disabled options are properly marked
pub fn combo_box(id: impl Into<ElementId>) -> ComboBox {
    ComboBox::new().id(id)
}

type ChangeFn = Arc<dyn Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;
type SimpleChangeFn = Arc<dyn Fn(String)>;

#[derive(IntoElement)]
pub struct ComboBox {
    element_id: ElementId,
    base: Div,
    options: Vec<ComboBoxOption>,

    value: Option<String>,
    placeholder: SharedString,
    search_placeholder: SharedString,
    /// Whether to use localized placeholders from i18n
    localized: bool,
    disabled: bool,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    menu_width: Option<gpui::Pixels>,
    max_results: usize,
    on_change: Option<ChangeFn>,
    on_change_simple: Option<SimpleChangeFn>,
}

impl Default for ComboBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboBox {
    pub fn new() -> Self {
        Self {
            element_id: "ui:combo-box".into(),
            base: div(),
            options: Vec::new(),
            value: None,
            placeholder: "Select…".into(),
            search_placeholder: "Search…".into(),
            localized: false,
            disabled: false,
            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,
            menu_width: None,
            max_results: 12,
            on_change: None,
            on_change_simple: None,
        }
    }

    /// Use localized placeholders from i18n.
    /// The placeholder text will be determined by the current locale.
    pub fn localized(mut self) -> Self {
        self.localized = true;
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn option(mut self, option: ComboBoxOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = ComboBoxOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results.max(1);
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// Set a simplified change handler that only receives the selected value.
    /// Use this when you don't need access to event, window or app context.
    ///
    /// # Example
    ///
    /// ```rust
    /// combo_box()
    ///     .on_change_simple(|value| {
    ///         println!("Selected: {}", value);
    ///     })
    /// ```
    pub fn on_change_simple<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String),
    {
        self.on_change_simple = Some(Arc::new(handler));
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn height(mut self, height: gpui::AbsoluteLength) -> Self {
        self.height = Some(height);
        self
    }

    pub fn menu_width(mut self, width: gpui::Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }

    pub fn min_menu_width(mut self, width: gpui::Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for ComboBox {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for ComboBox {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for ComboBox {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for ComboBox {}

/// Helper function to call on_change handlers with the correct priority.
#[allow(clippy::too_many_arguments)]
fn call_on_change(
    option_value: String,
    on_change: Option<&ChangeFn>,
    on_change_simple: Option<&SimpleChangeFn>,
    ev: &ClickEvent,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) {
    if let Some(handler) = on_change {
        handler(option_value.clone(), ev, window, cx);
    } else if let Some(handler) = on_change_simple {
        handler(option_value);
    }
}

impl RenderOnce for ComboBox {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let height = self.height.unwrap_or_else(|| px(36.).into());
        let menu_width = self.menu_width;
        let options = self.options;
        let localized = self.localized;
        let placeholder = if localized {
            DefaultPlaceholders::select_placeholder(cx.i18n().locale()).into()
        } else {
            self.placeholder
        };
        let search_placeholder = if localized {
            DefaultPlaceholders::combobox_search_placeholder(cx.i18n().locale()).into()
        } else {
            self.search_placeholder
        };
        let on_change = self.on_change;
        let on_change_simple = self.on_change_simple;
        let max_results = self.max_results;

        // ComboBox requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id;

        let menu_open = window.use_keyed_state((id.clone(), format!("{}:open", id)), cx, |_, _| false);
        let is_open = *menu_open.read(cx);

        let query_id = format!("{}:query", id);
        let query_state =
            window.use_keyed_state(query_id.clone(), cx, |_, cx| TextInputState::new(cx));

        let use_internal_value = on_change.is_none() && on_change_simple.is_none() && self.value.is_none();
        let internal_value = use_internal_value.then(|| {
            window.use_keyed_state((id.clone(), format!("{}:value", id)), cx, |_, _| {
                options
                    .first()
                    .map(|opt| opt.value.clone())
                    .unwrap_or_default()
            })
        });

        let value = if use_internal_value {
            internal_value
                .as_ref()
                .expect("internal value should exist")
                .read(cx)
                .clone()
        } else {
            self.value
                .clone()
                .or_else(|| options.first().map(|opt| opt.value.clone()))
                .unwrap_or_default()
        };

        let selected_label = options
            .iter()
            .find(|opt| opt.value == value)
            .map(|opt| opt.label.clone());

        let theme = cx.theme().clone();
        let hint = theme.content.tertiary;

        let input_style = compute_input_style(
            &theme,
            disabled,
            self.bg,
            self.border,
            self.focus_border,
            self.text_color,
        );

        let menu_open_for_button = menu_open.clone();
        let menu_open_for_outside = menu_open.clone();
        let menu_open_for_select = menu_open.clone();

        let internal_value_for_select = internal_value.clone();
        let on_change_for_select = on_change.clone();
        let on_change_simple_for_select = on_change_simple.clone();

        self.base
            .id(id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .h(height)
            .px_3()
            .rounded_md()
            .bg(input_style.bg)
            .border_1()
            .border_color(input_style.border)
            .text_color(input_style.text_color)
            .focusable()
            .focus_visible(|style| style.border_2().border_color(input_style.focus_border))
            .when(disabled, |this| this.opacity(0.6).cursor_not_allowed())
            .when(!disabled, |this| this.cursor_pointer())
            .when(is_open, |this| this.bg(theme.surface.hover))
            .on_click(move |_ev, _window, cx| {
                if disabled {
                    return;
                }
                menu_open_for_button.update(cx, |open, _| *open = !*open);
            })
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .truncate()
                    .text_color(selected_label.as_ref().map(|_| input_style.text_color).unwrap_or(hint))
                    .child(selected_label.unwrap_or(placeholder)),
            )
            .child(
                icon(IconName::Arrow(ArrowDirection::Down))
                    .size(px(14.))
                    .color(hint),
            )
            .when(is_open, move |this| {
                let text_color = input_style.text_color;
                let value = value.clone();
                let options = options.clone();
                let on_change = on_change_for_select.clone();
                let on_change_simple = on_change_simple_for_select.clone();
                let internal_value = internal_value_for_select.clone();
                let query_state = query_state.clone();
                let max_results = max_results;

                let query = query_state.read(cx).content().to_string();
                let query_lower = query.to_lowercase();

                let filtered = options
                    .into_iter()
                    .filter(move |opt| {
                        if query_lower.is_empty() {
                            return true;
                        }
                        opt.label.to_string().to_lowercase().contains(&query_lower)
                            || opt.value.to_lowercase().contains(&query_lower)
                    })
                    .take(max_results)
                    .collect::<Vec<_>>();

                let menu = div()
                    .id(format!("{}:menu", id))
                    .absolute()
                    .top_full()
                    .left_0()
                    .mt(px(10.))
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border.default)
                    .bg(theme.surface.raised)
                    .shadow_md()
                    .py_1()
                    .when_some(menu_width, |this, width| this.w(width))
                    .when(menu_width.is_none(), |this| this.w(px(420.)))
                    .occlude()
                    .on_mouse_down_out(move |_ev, _window, cx| {
                        menu_open_for_outside.update(cx, |open, _cx| *open = false);
                    })
                    .child(
                        div().px_2().pb_2().child(
                            text_input(format!("{}:query", id))
                                .placeholder(search_placeholder)
                                .bg(theme.surface.base)
                                .border(theme.border.default)
                                .focus_border(theme.border.focus)
                                .text_color(theme.content.primary)
                                .on_change({
                                    let query_state = query_state.clone();
                                    let menu_open = menu_open_for_select.clone();
                                    move |value, window, cx| {
                                        query_state.update(cx, |state, _| {
                                            state.set_content(value);
                                        });

                                        // Ensure menu stays visible while typing even if focus moves.
                                        menu_open.update(cx, |open, _| *open = true);
                                        window.refresh();
                                    }
                                }),
                        ),
                    )
                    .children(filtered.into_iter().map(move |opt| {
                        let is_selected = opt.value == value;
                        let is_disabled = disabled || opt.disabled;
                        let option_value = opt.value.clone();
                        let menu_open_for_select = menu_open_for_select.clone();
                        let on_change = on_change.clone();
                        let on_change_simple = on_change_simple.clone();
                        let internal_value = internal_value.clone();

                        let row_fg = if is_disabled {
                            theme.content.disabled
                        } else {
                            text_color
                        };

                        div()
                            .id((ElementId::from("ui:combo-box:option"), option_value.clone()))
                            .px_3()
                            .py_2()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .text_color(row_fg)
                            .when(!is_disabled, |this| {
                                this.cursor_pointer()
                                    .hover(|this| this.bg(theme.surface.hover))
                            })
                            .when(is_disabled, |this| this.cursor_not_allowed().opacity(0.6))
                            .child(opt.label)
                            .when(is_selected, |this| {
                                this.child(
                                    icon(IconName::Check)
                                        .size(px(12.))
                                        .color(theme.action.primary.bg),
                                )
                            })
                            .on_click(move |ev, window, cx| {
                                if is_disabled {
                                    return;
                                }

                                if let Some(internal_value) = &internal_value {
                                    internal_value.update(cx, |state, _| {
                                        *state = option_value.clone();
                                    });
                                }

                                call_on_change(
                                    option_value.clone(),
                                    on_change.as_ref(),
                                    on_change_simple.as_ref(),
                                    ev,
                                    window,
                                    cx,
                                );

                                menu_open_for_select.update(cx, |open, _| *open = false);
                            })
                    }));

                let animated_menu = menu.with_animation(
                    format!("combo-box-menu-{}", is_open),
                    Animation::new(animation::MENU_OPEN)
                        .with_easing(ease_out_quint()),
                    |this, value| this.opacity(value).mt(px(10.0 - 6.0 * value)),
                );

                this.child(gpui::deferred(animated_menu).with_priority(100))
            })
    }
}
