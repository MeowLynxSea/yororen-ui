//! Schema for the built-in XML tags accepted by the `xml!` macro.
//!
//! Each entry maps an XML tag to the headless component it
//! represents plus the prop/event vocabulary the codegen is
//! allowed to emit. The validator in [`crate::codegen`]
//! cross-checks every `AstElement` against this table and
//! reports a friendly `unknown tag` / `unknown attribute`
//! error if a user typo'd something.
//!
//! Adding a new built-in is a 3-step change:
//!
//! 1. Add a [`ComponentDef`] entry here.
//! 2. Add a codegen arm in [`crate::codegen`] for the new tag.
//! 3. Add a doc test under `xml! { ... }` in the
//!    `counter-xml-demo` crate.
//!
//! ## Container vs. leaf
//!
//! - **Container** (e.g. `Column`, `Row`, `Div`) starts from
//!   `gpui::div()`, applies style attrs (the gpui `Styled`
//!   trait's setter methods), and takes children via
//!   `.child(any_element)`.
//!
//! - **Leaf** (e.g. `Label`, `Button`) calls a headless factory
//!   (e.g. `yororen_ui_core::headless::button::button`),
//!   applies prop setters, and either `.render(cx)`s (for
//!   default styling) or stays as a pure `Props` value the
//!   caller composes via `.apply(div())`.

/// A schema entry for a single XML tag.
#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub tag: &'static str,
    pub kind: ComponentKind,
    /// Optional doc string surfaced in compiler errors.
    pub doc: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    /// A `Div`-based container. The macro generates
    /// `div().<style>(...)<style>(...)...<child>(child)...`.
    Container(ContainerDef),
    /// A headless component that produces a single element via
    /// `.render(cx)` (or `.apply(div())` for the headless path).
    Leaf(LeafDef),
    /// A pseudo-tag that produces Rust control flow at
    /// codegen time. It never produces an element of its own.
    ControlFlow(ControlFlowDef),
    /// A tag not known to the schema. The codegen emits
    /// a runtime lookup against [`crate::runtime::lookup`]
    /// — useful for user-registered custom components.
    /// Compile-time validation of attributes / events is
    /// skipped for these tags.
    RuntimeLeaf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerDef {
    /// Initial flex / layout methods. They are emitted as
    /// `.flex()`, `.flex_col()`, `.flex_row()`, etc. when the
    /// corresponding XML attr is present.
    ///
    /// `attr` is the XML attribute (e.g. `"col"`), `method`
    /// is the chained method (e.g. `"flex_col"`).
    pub fixed_methods: &'static [(&'static str, &'static str)],
    /// Hint printed when an unknown prop is used on this
    /// container. (`"the gpui Styled trait"`)
    pub style_hint: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafDef {
    /// Rust path to the headless factory function
    /// (e.g. `yororen_ui_core::headless::button::button`).
    pub factory: &'static str,
    /// Factory args **between** the `id` and the optional
    /// `&mut App` / `&mut Context<T>`. Each entry corresponds
    /// to one positional argument; the macro reads the
    /// corresponding XML attribute (named by `attr`) and
    /// splices it into the call.
    ///
    /// `Text` entries fall back to the element's inner text
    /// content when the attribute is absent. `Custom` entries
    /// require the attribute to be present.
    pub extra_args: &'static [ExtraArg],
    /// How the headless props get turned into an element.
    pub render: RenderMode,
    /// Whether the factory takes `&mut App` as the last arg
    /// (most do — the macro passes `cx` here). Set to `false`
    /// for factories that don't need an app context (e.g. the
    /// newer text_input that mints state internally).
    pub needs_app: bool,
    /// Whether the leaf's `render` method needs a
    /// `&mut Window` argument. The macro passes
    /// `&mut *window` from the enclosing scope when
    /// this is set; otherwise it omits the second
    /// arg. Set to `true` for `TextInput` (and any
    /// future headless component whose render
    /// signature is `fn render(self, cx: &mut App,
    /// window: &mut Window)`).
    pub needs_window: bool,
    /// Prop vocabulary: (xml_attr, builder_method, value_transform).
    pub props: &'static [PropDef],
    /// Event vocabulary: (xml_attr like `"on_click"`, builder_method).
    pub events: &'static [(&'static str, &'static str)],
    /// Whether the leaf can wrap `.child(text)` after the render
    /// call (true for `Button` so users can do
    /// `<Button>Click me</Button>`).
    pub supports_text_child: bool,
    /// If `true`, element children are added via `.child(...)`
    /// **before** `.render(cx)` is called. Required for
    /// components whose renderer consumes children at compose
    /// time (e.g. `ButtonGroup`, `Modal`).
    pub children_before_render: bool,
    /// If `children_before_render` is `true`, this flag decides
    /// whether the child expressions are emitted in their raw
    /// rendered type (`false`, default) or without the final
    /// `.into_any_element()` wrapper (`true`). Set to `true` for
    /// `ButtonGroup`, whose `.child()` method expects the
    /// `Stateful<Div>` returned by `button(...).render(cx)`.
    pub unwrap_children: bool,
    /// Named slots for components that pass specific children to
    /// builder methods before rendering (e.g. `Popover`'s
    /// `trigger` / `content`, `Tooltip`'s `trigger`).
    pub slots: &'static [SlotDef],
}

/// A named slot on a leaf component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotDef {
    /// Slot name used in XML: `<Button slot="trigger">`.
    pub name: &'static str,
    /// Builder method called with the slotted child:
    /// `__el.trigger(child)`.
    pub setter: &'static str,
}

/// How the codegen should turn an XML attribute value into a
/// Rust expression for a prop setter call.
///
/// - [`PropValue::String`] — `attr="hello"` becomes
///   `"hello".into()` (works for `SharedString`, `String`,
///   `&str`, etc.).
/// - [`PropValue::Bool`] — `attr="true"` becomes `true`,
///   `attr="false"` becomes `false`. Anything else is an
///   error.
/// - [`PropValue::Variant`] — `attr="primary"` becomes
///   `::yororen_ui::ActionVariantKind::Primary`. Only the
///   three built-in variants are supported for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropValue {
    String,
    Bool,
    Variant,
    /// A `f64`-typed setter (e.g. `NumberInput::value`).
    /// The codegen emits `f64` writes through `XmlBinding<f64>`
    /// when this kind is paired with `on_change`.
    Float64,
    /// A `f32`-typed setter (e.g. `Slider::value`).
    /// The codegen emits `f32` writes through `XmlBinding<f32>`
    /// when this kind is paired with `on_change`.
    Float32,
    /// A `usize`-typed setter (e.g. `Table::selected` or
    /// `VirtualList::item_count`).
    UInt,
    /// A `BadgeVariant` setter (e.g. `Badge::variant`).
    BadgeVariant,
    /// `yororen_ui::headless::heading::HeadingLevel`.
    HeadingLevel,
    /// `yororen_ui::headless::icon::IconSource`.
    IconSource,
    /// `yororen_ui::headless::image::ImageSource`.
    ImageSource,
    /// `yororen_ui::headless::keybinding_input::KeybindingInputMode`.
    KeybindingInputMode,
    /// `yororen_ui::headless::tree::TreeSelectionMode` —
    /// `None` / `Single` / `Multiple`.
    TreeSelectionMode,
    /// `yororen_ui::headless::layout::Spacing` — gap/p/px/py
    /// token (named `xs`/`sm`/`md`/`lg`/`xl`/`xxl`).
    Spacing,
    /// `yororen_ui::headless::layout::Inset` — padding/margin
    /// token (named `xs`/`sm`/`md`/`lg`/`xl`).
    Inset,
    /// `yororen_ui::headless::layout::AlignItems` — flex
    /// cross-axis alignment.
    AlignItems,
    /// `yororen_ui::headless::layout::JustifyContent` — flex
    /// main-axis distribution.
    JustifyContent,
    /// `yororen_ui::headless::layout::Length` — width/height
    /// (`full`/`fit`/`auto`/`px(N)`/`rem(N)`/`pct(N)`).
    Length,
    /// A gpui color (`Hsla` / `Rgba`). String literals may be
    /// hex (`#rrggbb` / `#rrggbbaa`); brace expressions are
    /// passed through verbatim.
    Color,
    /// A zero-arg setter (`fn wrap(self) -> Self`). The
    /// codegen emits `.setter()` when the attribute is
    /// present (regardless of value). Used for flag-style
    /// toggles like `Label.wrap`.
    Flag,
    /// The generator couldn't classify the type — the user
    /// should review. The codegen treats it as `String`
    /// (i.e. `(#raw).to_string()`).
    Unknown,
    /// A fully custom value: the XML attribute expression is
    /// passed through verbatim (no `.to_string()` / `.into()`
    /// wrapper). Literal values are wrapped in `.into()` so
    /// they can satisfy `impl Into<…>` setters.
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropDef {
    pub name: &'static str,
    pub setter: &'static str,
    pub value: PropValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtraArg {
    pub kind: ExtraArgKind,
    /// The XML attribute that holds the value (for `Custom`).
    /// For `Text`, the macro synthesises a `String` from the
    /// `text` XML attribute (if present) or the inner text
    /// content of the element.
    pub attr: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraArgKind {
    /// Use the literal / brace expression of the `attr` XML
    /// attribute, wrapped in `.into()` to coerce to the
    /// factory's expected string type.
    Custom,
    /// A closure-type positional factory argument (e.g.
    /// `SplitButton`'s `primary` callback). Bare path
    /// expressions (`controller.method`) are auto-wrapped
    /// into a `(arg0, &mut Window, &mut App)` closure, just
    /// like component events; other expressions pass through
    /// verbatim.
    Callback,
    /// The label's text: prefer the `text` XML attribute,
    /// fall back to the element's inner text content.
    Text,
    /// A `usize` positional argument (e.g. the `item_count`
    /// of `uniform_virtual_list`). Raw string values are
    /// parsed as decimal integers; brace expressions pass
    /// through verbatim.
    UInt,
    /// A `HeadingLevel` positional argument (e.g. `Heading`'s
    /// `level`). Raw string values are mapped to
    /// `::yororen_ui::headless::heading::HeadingLevel::H1..H6`.
    HeadingLevel,
    /// An `IconSource` positional argument (e.g. `Icon`'s
    /// `source`). Raw string values become
    /// `IconSource::Builtin(...)`.
    IconSource,
    /// An `ImageSource` positional argument (e.g. `Image`'s
    /// `source`). Raw string values become
    /// `ImageSource::Resource(...)`.
    ImageSource,
    /// A `KeybindingInputMode` positional argument. Raw string
    /// values are mapped to the enum variants.
    KeybindingInputMode,
    /// A gpui colour positional argument (`Hsla` / `Rgba`).
    Color,
    /// An iterator of strings (e.g. `KeybindingDisplay`'s
    /// `keys: impl IntoIterator<Item = impl Into<String>>`).
    /// Literal XML attribute values are rejected; brace
    /// expressions pass through verbatim.
    StringList,
    /// A borrowed reference positional argument (e.g.
    /// `FocusRing`'s `handle: &FocusHandle`). The XML
    /// expression should evaluate to a reference; the
    /// factory clones the referenced value internally.
    Borrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// The default path: factory + props + `.render(cx)` →
    /// `Stateful<Div>` (or `AnyElement` for some inputs).
    /// The macro appends `.into_any_element()` so the result
    /// can be used as a child of a container.
    Default,
    /// The headless path: factory + props + `.apply(div())`
    /// — caller composes the visual entirely.
    /// The macro appends `.into_any_element()`.
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowDef {
    /// `<If condition={expr}>` — emits `if expr { ... }`.
    If,
    /// `<ElseIf condition={expr}>` — emits `else if expr { ... }`.
    ElseIf,
    /// `<Else>` — emits `else { ... }`.
    Else,
    /// `<For each={expr} let:item>{...}</For>` — emits
    /// `expr.into_iter().map(|item| { ... }).collect::<Vec<_>>()`.
    For,
    /// `<Fragment>` — emits a `Vec<AnyElement>` (for now,
    /// used only as a grouping construct).
    Fragment,
    /// `<Include src="path.xml" />` — at compile time,
    /// read the file and splice its children in. The
    /// `src` path is resolved relative to the calling
    /// source file (same convention as `xml_file!`).
    Include,
    /// `<Template name="X">…</Template>` — define a
    /// named block of XML. The body is captured but
    /// only emitted when referenced by a `Slot` match
    /// (or by `<UseTemplate name="X">` — coming soon).
    /// For the MVP, `<Template>` simply emits its
    /// children in place (compile-time inlining).
    Template,
    /// `<Slot/>` — placeholder for the caller's
    /// children. For the MVP, `<Slot/>` is a no-op
    /// (emits nothing); the children of a `Template`
    /// are inlined as-is. A future revision will wire
    /// named slots and caller-side slot-filling.
    Slot,
    /// `<Match on={expr}>` followed by `<Case pattern={...}>…</Case>`
    /// siblings — emits a Rust `match` expression. The
    /// last `<Case _>` (with literal `_` as the pattern
    /// attribute) becomes the wildcard arm.
    Match,
    /// `<VirtualList id="…" item_count={n} let:index={i}>…</VirtualList>`
    /// — a heterogeneous-height gpui virtual list. The
    /// codegen mints and persists a `VirtualListController`
    /// via `window.use_keyed_state(id, …)` so the host
    /// view never needs to carry a controller field. The
    /// children become the row-template body, re-invoked
    /// per visible index (so off-screen rows are never
    /// built). `let:index={name}` binds the visible row
    /// index inside the body (defaults to `index`); an
    /// optional `let:item` binds the `usize` itself.
    /// Optional `overdraw` (default `px(16.)`), `alignment`
    /// (`"top"` / `"bottom"`, default `"top"`), and
    /// `on_visible_range_change={…}` are forwarded.
    VirtualList,
    /// `<UniformVirtualList id="…" item_count={n} let:index={i}>…</UniformVirtualList>`
    /// — the equal-height variant. Faster (gpui measures
    /// only the first row and lays the rest in a line),
    /// but has no `on_visible_range_change` support (the
    /// underlying `gpui::uniform_list` lacks a scroll
    /// handler). Otherwise identical to `<VirtualList>`,
    /// including the auto-persisted controller.
    UniformVirtualList,
    /// `<Case pattern={...}>…</Case>` — a single arm of
    /// a `<Match>`. `pattern` is the Rust pattern (e.g.
    /// `Status::Loading` or `_` for wildcard).
    Case,
    /// `<State name="count" default="0">…</State>` — declare a
    /// local `Entity<T>` inside the `Render::render` closure.
    /// `default` is a stringified literal that becomes the
    /// initial value via `cx.new(|_| …)`. Children are emitted
    /// in the closure scope where `name` resolves to the
    /// entity.
    State,
}

impl ComponentDef {
    /// Look up a tag by name (case-sensitive).
    pub fn lookup(tag: &str) -> Option<&'static ComponentDef> {
        BUILTINS.iter().find(|c| c.tag == tag)
    }
}

// Helper for diagnostics (did-you-mean) that returns every
// built-in tag, including generated leaves and overrides.
pub fn builtin_tags() -> Vec<&'static str> {
    use crate::schema_generated::{BUILTINS_GENERATED, BUILTINS_OVERRIDES};
    let mut tags: Vec<&'static str> = BUILTINS.iter().map(|c| c.tag).collect();
    tags.extend(BUILTINS_OVERRIDES.iter().map(|c| c.tag));
    tags.extend(BUILTINS_GENERATED.iter().map(|c| c.tag));
    tags.sort_unstable();
    tags.dedup();
    tags
}

/// All built-in tags. Keep this list sorted alphabetically for
/// easy diffs. The validator iterates it linearly; for the
/// MVP that's fine. (Phase 3 can switch to a `phf` map if
/// the table grows past ~100 entries.)
pub static BUILTINS: &[ComponentDef] = &[
    ComponentDef {
        tag: "Case",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Case),
        doc: "Match arm — `<Case pattern={expr}>…</Case>`.",
    },
    ComponentDef {
        tag: "Match",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Match),
        doc: "Pattern match — `<Match on={expr}><Case>…</Case></Match>`.",
    },
    ComponentDef {
        tag: "State",
        kind: ComponentKind::ControlFlow(ControlFlowDef::State),
        doc: "Declare a local `Entity<T>` — `<State name=\"count\" default=\"0\">…</State>`.",
    },
    ComponentDef {
        tag: "VirtualList",
        kind: ComponentKind::ControlFlow(ControlFlowDef::VirtualList),
        doc: "Virtual list with an auto-persisted controller — `<VirtualList id=\"…\" item_count={n} let:index={i}>…</VirtualList>`.",
    },
    ComponentDef {
        tag: "UniformVirtualList",
        kind: ComponentKind::ControlFlow(ControlFlowDef::UniformVirtualList),
        doc: "Equal-height virtual list with an auto-persisted controller — `<UniformVirtualList id=\"…\" item_count={n} let:index={i}>…</UniformVirtualList>`.",
    },
    ComponentDef {
        tag: "Div",
        kind: ComponentKind::Container(ContainerDef {
            fixed_methods: &[],
            style_hint: "the gpui Styled trait (`.flex`, `.gap_3()`, …)",
        }),
        doc: "Plain `div()` — no flex by default. Escape hatch for raw gpui styling.",
    },
    ComponentDef {
        tag: "Else",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Else),
        doc: "Else branch of an `<If>`.",
    },
    ComponentDef {
        tag: "ElseIf",
        kind: ComponentKind::ControlFlow(ControlFlowDef::ElseIf),
        doc: "Else-if branch of an `<If>`.",
    },
    ComponentDef {
        tag: "For",
        kind: ComponentKind::ControlFlow(ControlFlowDef::For),
        doc: "Iterate a Rust expression: `<For each={items} let:item>…</For>`.",
    },
    ComponentDef {
        tag: "Fragment",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Fragment),
        doc: "Group children without producing an element wrapper.",
    },
    ComponentDef {
        tag: "Include",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Include),
        doc: "Inline another XML file at compile time: `<Include src=\"toolbar.xml\" />`.",
    },
    ComponentDef {
        tag: "Template",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Template),
        doc: "Define a reusable XML template. Use `<Slot/>` to mark insertion points.",
    },
    ComponentDef {
        tag: "Slot",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Slot),
        doc: "Placeholder inside a `<Template>` — replaced by the caller's children at the matching position.",
    },
    ComponentDef {
        tag: "If",
        kind: ComponentKind::ControlFlow(ControlFlowDef::If),
        doc: "Conditional rendering: `<If condition={cond}>…</If>`.",
    },
    // NOTE: `UniformVirtualList` and `VirtualList` are ControlFlow
    // tags (see `ControlFlowDef`), not Leaves — they have a row-
    // template body and an auto-persisted controller. The codegen
    // arms live in `codegen_virtual_list` /
    // `codegen_uniform_virtual_list`.
    //
    // NOTE: `Column` / `Row` / `Stack` / `Center` / `Expanded` /
    // `Wrap` are auto-generated as `Leaf` entries from
    // `headless/layout/*.rs` (see `BUILTINS_GENERATED`).
];

/// Reserved XML attribute names consumed by the macro itself
/// (control flow, model alias, etc.). They're not props.
pub const RESERVED_ATTRS: &[&str] = &[
    "id",        // element id (passed as the first factory arg)
    "condition", // <If condition={...}>
    "each",      // <For each={...}>
    "let",       // <For let:item>
    "model",     // alias for the surrounding ViewModel
    "key",       // <For key={...}>
    "if",        // reserved for future inline-if
];

/// Whether a string is a known shorthand suffix that we
/// should expand to a literal method call. Examples:
///
/// - `gap_3` → `.gap_3()`
/// - `p_4` → `.p_4()`
/// - `w_full` → `.w_full()`
///
/// Anything not in this table is rejected on containers
/// (with a hint pointing to the gpui `Styled` trait) — we
/// don't try to invent methods at macro time.
pub fn is_known_shorthand_method(name: &str) -> bool {
    matches!(
        name,
        // flex / layout
        "flex" | "flex_col" | "flex_row" | "flex_col_reverse" | "flex_row_reverse"
        | "flex_wrap" | "flex_wrap_reverse" | "flex_nowrap"
        | "flex_1" | "flex_auto" | "flex_initial" | "flex_none"
        | "flex_grow" | "flex_shrink" | "flex_shrink_0"
        // alignment
        | "items_start" | "items_end" | "items_center" | "items_baseline" | "items_stretch"
        | "justify_start" | "justify_end" | "justify_center"
        | "justify_between" | "justify_around" | "justify_evenly"
        // size
        | "w_full" | "h_full" | "size_full"
        | "min_w_full" | "min_h_full" | "max_w_full" | "max_h_full"
        // overflow / display
        | "overflow_hidden" | "overflow_x_hidden" | "overflow_y_hidden"
        | "overflow_scroll" | "overflow_x_scroll" | "overflow_y_scroll"
        | "visible" | "invisible" | "block" | "inline_block" | "inline_flex"
        | "hidden"
        // background
        | "bg"
        // position
        | "relative" | "absolute"
        // cursor
        | "cursor_default" | "cursor_pointer" | "cursor_text" | "cursor_move"
        | "cursor_not_allowed" | "cursor_grab" | "cursor_grabbing"
        // border
        | "border" | "border_1" | "border_2" | "border_4" | "border_8"
        | "border_t" | "border_b" | "border_l" | "border_r"
        | "border_x" | "border_y"
        | "border_color"
        | "rounded" | "rounded_sm" | "rounded_md" | "rounded_lg" | "rounded_xl" | "rounded_full"
        | "rounded_none"
        // shadow
        | "shadow_sm" | "shadow_md" | "shadow_lg" | "shadow_xl" | "shadow_none"
        // visibility
        | "opacity_0" | "opacity_25" | "opacity_50" | "opacity_75" | "opacity_100"
    ) || is_spacing_shorthand(name)
}

/// Methods that live on `gpui::StatefulInteractiveElement` rather
/// than `gpui::Styled`. Containers must be promoted from a bare
/// `Div` to `Stateful<Div>` before these can be called.
pub fn is_stateful_interactive_method(name: &str) -> bool {
    matches!(
        name,
        "overflow_scroll" | "overflow_x_scroll" | "overflow_y_scroll"
    )
}

/// Numeric suffixes for `is_spacing_shorthand`. The container
/// codegen reuses this table via [`NUMERIC_SPACING_SUFFIX`] to
/// validate literal `gap="3"`, `p="4"`, `w="full"`, etc. — keep
/// it the single source of truth so the two views cannot drift.
pub const NUMERIC_SPACING_SUFFIX: &[&str] = &[
    "0", "0p5", "1", "1p5", "2", "2p5", "3", "3p5", "4", "5", "6", "7", "8", "9", "10", "11", "12",
    "16", "20", "24", "32", "40", "48", "56", "64", "72", "80", "96",
];
/// Textual spacing suffixes. Same single-source-of-truth note
/// as [`NUMERIC_SPACING_SUFFIX`].
pub const TEXTUAL_SPACING_SUFFIX: &[&str] = &[
    "full", "1_2", "1_3", "2_3", "1_4", "3_4", "1_5", "2_5", "3_5", "4_5", "1_6", "5_6", "1_12",
];

/// Numeric suffixes for `is_spacing_shorthand`. Mirrors the
/// table in [`crate::codegen::container::is_valid_spacing_suffix`]
/// — keep the two in sync when adding new entries.
const SPACING_NUMERIC: &[&str] = NUMERIC_SPACING_SUFFIX;
/// Textual spacing suffixes for `is_spacing_shorthand`. Same
/// sync note as [`SPACING_NUMERIC`].
const SPACING_TEXTUAL: &[&str] = TEXTUAL_SPACING_SUFFIX;

/// Spacing shorthands for `gap`, `p`, `m`, `px`, `py`, `pt`,
/// `pb`, `pl`, `pr`, `mx`, `my`, `mt`, `mb`, `ml`, `mr`, and
/// their `gap_x_` / `gap_y_` siblings. Each numeric / textual
/// suffix maps to a real gpui method.
pub fn is_spacing_shorthand(name: &str) -> bool {
    let prefix_end = if let Some(stripped) = name.strip_suffix("_full") {
        Some(stripped)
    } else if let Some(idx) = name.rfind('_') {
        let (prefix, suffix) = name.split_at(idx);
        let suffix = &suffix[1..];
        if SPACING_NUMERIC.contains(&suffix) || SPACING_TEXTUAL.contains(&suffix) {
            Some(prefix)
        } else {
            None
        }
    } else {
        None
    };

    match prefix_end {
        Some(prefix) => matches!(
            prefix,
            "gap"
                | "gap_x"
                | "gap_y"
                | "p"
                | "px"
                | "py"
                | "pt"
                | "pb"
                | "pl"
                | "pr"
                | "m"
                | "mx"
                | "my"
                | "mt"
                | "mb"
                | "ml"
                | "mr"
                | "w"
                | "h"
                | "size"
                | "min_w"
                | "min_h"
                | "max_w"
                | "max_h"
                | "inset"
        ),
        None => false,
    }
}

/// Whether `name` is a **prefix** that takes a numeric /
/// textual suffix (e.g. `gap_3`, `p_4`, `w_full`). The full
/// method name is `<name>_<suffix>`. This is used by the
/// codegen to expand `gap="3"` to `.gap_3()`.
pub fn is_spacing_prefix(name: &str) -> bool {
    matches!(
        name,
        "gap"
            | "gap_x"
            | "gap_y"
            | "p"
            | "px"
            | "py"
            | "pt"
            | "pb"
            | "pl"
            | "pr"
            | "m"
            | "mx"
            | "my"
            | "mt"
            | "mb"
            | "ml"
            | "mr"
            | "w"
            | "h"
            | "size"
            | "min_w"
            | "min_h"
            | "max_w"
            | "max_h"
            | "inset"
    )
}

/// Look up a tag across all built-in schema tables.
///
/// Search order:
/// 1. Hand-written [`BUILTINS`] in this file.
/// 2. Generated overrides (`BUILTINS_OVERRIDES`).
/// 3. Auto-generated leaves (`BUILTINS_GENERATED`).
/// 4. User-supplied schema from `yororen-ui-xml-components.toml`.
pub fn lookup_component<'a>(
    tag: &str,
    user_schema: &'a [ComponentDef],
) -> Option<&'a ComponentDef> {
    use crate::schema_generated::{BUILTINS_GENERATED, BUILTINS_OVERRIDES};
    if let Some(c) = BUILTINS.iter().find(|c| c.tag == tag) {
        return Some(c);
    }
    if let Some(c) = BUILTINS_OVERRIDES.iter().find(|c| c.tag == tag) {
        return Some(c);
    }
    if let Some(c) = BUILTINS_GENERATED.iter().find(|c| c.tag == tag) {
        return Some(c);
    }
    user_schema.iter().find(|c| c.tag == tag)
}

/// Like [`lookup_component`] but returns an owned clone.
/// Useful when the caller needs to keep the result without
/// tying it to the schema slice's lifetime.
pub fn lookup_component_owned(tag: &str, user_schema: &[ComponentDef]) -> Option<ComponentDef> {
    lookup_component(tag, user_schema).cloned()
}

// ----- user-defined schema from `yororen-ui-xml-components.toml` -----

/// Parse a TOML description of user-defined XML components.
///
/// Example `yororen-ui-xml-components.toml`:
///
/// ```toml
/// [[component]]
/// tag = "Chart"
/// factory = "my_crate::chart::chart"
/// render = "Default"
/// needs_window = false
/// supports_text_child = true
///
/// [[component.extra_args]]
/// kind = "Text"
/// attr = "title"
///
/// [[component.props]]
/// name = "data"
/// setter = "data"
/// value = "String"
///
/// [[component.events]]
/// name = "on_click"
/// setter = "on_click"
///
/// [[component.slots]]
/// name = "trigger"
/// setter = "trigger"
/// ```
///
/// `value` must be one of: `String`, `Bool`, `UInt`, `Float32`,
/// `Float64`, `Variant`, `BadgeVariant`, `HeadingLevel`,
/// `IconSource`, `ImageSource`, `KeybindingInputMode`, `Color`,
/// `Flag`, `Unknown`.
///
/// `extra_args.kind` must be one of: `Custom`, `Callback`, `Text`,
/// `UInt`, `HeadingLevel`, `IconSource`, `ImageSource`,
/// `KeybindingInputMode`, `Color`, `StringList`, `Borrow`.
#[derive(Debug, serde::Deserialize)]
struct UserFile {
    #[serde(default)]
    component: Vec<UserComponent>,
}

#[derive(Debug, serde::Deserialize)]
struct UserComponent {
    tag: String,
    factory: String,
    #[serde(default = "default_render")]
    render: String,
    #[serde(default = "default_true")]
    needs_app: bool,
    #[serde(default)]
    needs_window: bool,
    #[serde(default)]
    extra_args: Vec<UserExtraArg>,
    #[serde(default)]
    props: Vec<UserProp>,
    #[serde(default)]
    events: Vec<UserEvent>,
    #[serde(default)]
    supports_text_child: bool,
    #[serde(default)]
    children_before_render: bool,
    #[serde(default)]
    unwrap_children: bool,
    #[serde(default)]
    slots: Vec<UserSlot>,
}

fn default_render() -> String {
    "Default".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, serde::Deserialize)]
struct UserProp {
    name: String,
    setter: String,
    value: String,
}

#[derive(Debug, serde::Deserialize)]
struct UserEvent {
    name: String,
    setter: String,
}

#[derive(Debug, serde::Deserialize)]
struct UserExtraArg {
    kind: String,
    attr: String,
}

#[derive(Debug, serde::Deserialize)]
struct UserSlot {
    name: String,
    setter: String,
}

pub fn parse_user_schema(toml_str: &str) -> Result<Vec<ComponentDef>, String> {
    let file: UserFile = toml::from_str(toml_str)
        .map_err(|e| format!("invalid yororen-ui-xml-components.toml: {e}"))?;

    file.component
        .into_iter()
        .map(build_user_component)
        .collect()
}

fn build_user_component(def: UserComponent) -> Result<ComponentDef, String> {
    let mut props: Vec<PropDef> = Vec::with_capacity(def.props.len());
    for p in def.props {
        props.push(PropDef {
            name: leak_str(p.name),
            setter: leak_str(p.setter),
            value: parse_prop_value(&p.value)?,
        });
    }

    let events: Vec<(&'static str, &'static str)> = def
        .events
        .into_iter()
        .map(|e| (leak_str(e.name), leak_str(e.setter)))
        .collect();

    let extra_args: Vec<ExtraArg> = def
        .extra_args
        .into_iter()
        .map(|e| {
            Ok(ExtraArg {
                kind: parse_extra_arg_kind(&e.kind)?,
                attr: leak_str(e.attr),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let slots: Vec<SlotDef> = def
        .slots
        .into_iter()
        .map(|s| SlotDef {
            name: leak_str(s.name),
            setter: leak_str(s.setter),
        })
        .collect();

    let render = match def.render.as_str() {
        "Default" => RenderMode::Default,
        "Apply" => RenderMode::Apply,
        other => return Err(format!("unknown render mode `{other}`")),
    };

    let leaf = LeafDef {
        factory: leak_str(def.factory),
        extra_args: leak_slice(extra_args),
        render,
        needs_app: def.needs_app,
        needs_window: def.needs_window,
        props: leak_slice(props),
        events: leak_slice(events),
        supports_text_child: def.supports_text_child,
        children_before_render: def.children_before_render,
        unwrap_children: def.unwrap_children,
        slots: leak_slice(slots),
    };

    Ok(ComponentDef {
        tag: leak_str(def.tag),
        kind: ComponentKind::Leaf(leaf),
        doc: "user-defined from yororen-ui-xml-components.toml",
    })
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn leak_slice<T>(v: Vec<T>) -> &'static [T] {
    Box::leak(v.into_boxed_slice())
}

fn parse_prop_value(raw: &str) -> Result<PropValue, String> {
    match raw {
        "String" => Ok(PropValue::String),
        "Bool" => Ok(PropValue::Bool),
        "UInt" => Ok(PropValue::UInt),
        "Float32" => Ok(PropValue::Float32),
        "Float64" => Ok(PropValue::Float64),
        "Variant" => Ok(PropValue::Variant),
        "BadgeVariant" => Ok(PropValue::BadgeVariant),
        "HeadingLevel" => Ok(PropValue::HeadingLevel),
        "IconSource" => Ok(PropValue::IconSource),
        "ImageSource" => Ok(PropValue::ImageSource),
        "KeybindingInputMode" => Ok(PropValue::KeybindingInputMode),
        "TreeSelectionMode" => Ok(PropValue::TreeSelectionMode),
        "Spacing" => Ok(PropValue::Spacing),
        "Inset" => Ok(PropValue::Inset),
        "AlignItems" => Ok(PropValue::AlignItems),
        "JustifyContent" => Ok(PropValue::JustifyContent),
        "Length" => Ok(PropValue::Length),
        "Color" => Ok(PropValue::Color),
        "Flag" => Ok(PropValue::Flag),
        "Unknown" => Ok(PropValue::Unknown),
        "Custom" => Ok(PropValue::Custom),
        other => Err(format!("unknown PropValue `{other}`")),
    }
}

fn parse_extra_arg_kind(raw: &str) -> Result<ExtraArgKind, String> {
    match raw {
        "Custom" => Ok(ExtraArgKind::Custom),
        "Callback" => Ok(ExtraArgKind::Callback),
        "Text" => Ok(ExtraArgKind::Text),
        "UInt" => Ok(ExtraArgKind::UInt),
        "HeadingLevel" => Ok(ExtraArgKind::HeadingLevel),
        "IconSource" => Ok(ExtraArgKind::IconSource),
        "ImageSource" => Ok(ExtraArgKind::ImageSource),
        "KeybindingInputMode" => Ok(ExtraArgKind::KeybindingInputMode),
        "Color" => Ok(ExtraArgKind::Color),
        "StringList" => Ok(ExtraArgKind::StringList),
        "Borrow" => Ok(ExtraArgKind::Borrow),
        other => Err(format!("unknown ExtraArgKind `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_user_schema_roundtrip() {
        let toml = r#"
[[component]]
tag = "Chart"
factory = "my_crate::chart::chart"

[[component.props]]
name = "data"
setter = "data"
value = "String"

[[component.events]]
name = "on_click"
setter = "on_click"
"#;
        let schema = parse_user_schema(toml).expect("parse succeeds");
        assert_eq!(schema.len(), 1);
        assert_eq!(schema[0].tag, "Chart");
        let ComponentKind::Leaf(leaf) = schema[0].kind else {
            panic!("expected Leaf");
        };
        assert_eq!(leaf.factory, "my_crate::chart::chart");
        assert_eq!(leaf.props.len(), 1);
        assert_eq!(leaf.props[0].name, "data");
        assert_eq!(leaf.props[0].value, PropValue::String);
        assert_eq!(leaf.events.len(), 1);
        assert_eq!(leaf.events[0].0, "on_click");
    }

    #[test]
    fn parse_user_schema_extended_leaf_def() {
        let toml = r#"
[[component]]
tag = "Dialog"
factory = "my_crate::dialog::dialog"
render = "Default"
needs_app = false
needs_window = true
supports_text_child = true
children_before_render = true
unwrap_children = true

[[component.extra_args]]
kind = "Text"
attr = "title"

[[component.props]]
name = "open"
setter = "open"
value = "Bool"

[[component.events]]
name = "on_close"
setter = "on_close"

[[component.slots]]
name = "footer"
setter = "footer"
"#;
        let schema = parse_user_schema(toml).expect("parse succeeds");
        assert_eq!(schema.len(), 1);
        let ComponentKind::Leaf(leaf) = schema[0].kind else {
            panic!("expected Leaf");
        };
        assert_eq!(leaf.factory, "my_crate::dialog::dialog");
        assert!(!leaf.needs_app);
        assert!(leaf.needs_window);
        assert!(leaf.supports_text_child);
        assert!(leaf.children_before_render);
        assert!(leaf.unwrap_children);
        assert_eq!(leaf.extra_args.len(), 1);
        assert_eq!(leaf.extra_args[0].kind, ExtraArgKind::Text);
        assert_eq!(leaf.extra_args[0].attr, "title");
        assert_eq!(leaf.slots.len(), 1);
        assert_eq!(leaf.slots[0].name, "footer");
        assert_eq!(leaf.slots[0].setter, "footer");
    }

    #[test]
    fn lookup_falls_back_to_user_schema() {
        let toml = r#"
[[component]]
tag = "CustomWidget"
factory = "my_crate::widget"
"#;
        let user = parse_user_schema(toml).expect("parse succeeds");
        assert!(lookup_component("CustomWidget", &user).is_some());
        assert!(lookup_component("DefinitelyNotARealTag", &user).is_none());
    }
}
