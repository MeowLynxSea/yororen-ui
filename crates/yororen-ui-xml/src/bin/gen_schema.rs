//! Schema generator for `yororen-ui-xml`.
//!
//! Scans `yororen-ui-core/src/headless/*.rs`, extracts the
//! factory function, `*Props` struct, and `impl` setters,
//! and writes the result to `src/schema_generated.rs`.
//!
//! ## Usage
//!
//! ```text
//! # From the workspace root:
//! cargo run -p yororen_ui_xml --bin gen-schema                 # write the file
//! cargo run -p yororen_ui_xml --bin gen-schema -- --check       # fail if drift
//! cargo run -p yororen_ui_xml --bin gen-schema -- --headless /path/to/headless --out /path/to/output.rs
//! ```
//!
//! ## What we can auto-detect
//!
//! - Factory function (its path becomes the headless entry point).
//! - `*Props` struct (looks for `<PascalCase(file_stem)>Props`).
//! - `pub fn X(self, x: T) -> Self` setters → `PropDef` entries
//!   (bool / string / variant classified from `T`).
//! - `pub fn on_X(self, f: F) -> Self` setters → `EventDef` entries.
//! - `pub fn render(self, cx: &App)` → `RenderMode::Default`.
//! - `pub fn apply(self, el: Div)` → `RenderMode::Apply`.
//! - Whether the factory needs `cx` (i.e. the last arg is
//!   `&mut App` or `&mut Context<T>`).
//! - An "extra arg" (Label's `text`, Badge's `text`, Icon's
//!   `source`, …) when the factory has exactly one arg
//!   between the `id` and the optional `cx`.
//!
//! ## What we cannot auto-detect
//!
//! - Stateful composites: factories that take `state: Entity<XxxState>` (modal, popover, select, …).
//!   These need a separate `entity_attr = "state"` annotation in the generated output.
//! - Factories with more than one extra arg (e.g. `heading(id, level, text, cx)`).
//! - Components that don't follow the `(id, …) -> XxxProps` convention.
//! - XML-specific aliases (`Column` exposes `col` instead of `flex_col`; `Row` exposes `row` instead of `flex_row`).
//! - Whether a component "supports a text child" (Button's `.child("Save")`).
//!
//! For these we emit a comment in the generated file
//! (`// MANUAL: <hint>`) so the user can fill in the
//! `overrides.rs` table next to the binary.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::{
    FnArg, ImplItem, Item, ItemFn, ItemImpl, ItemStruct, Signature, Type, TypePath, Visibility,
};

#[derive(Debug, Default, serde::Deserialize)]
struct OverridesFile {
    #[serde(default)]
    override_: Vec<OverrideEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct OverrideEntry {
    tag: String,
    kind: String,
    /// Override the auto-detected factory path.
    ///
    /// Use this when two headless modules expose the same tag
    /// (e.g. the legacy `headless::spacer` and the new
    /// `headless::layout::spacer` both become `<Spacer>`), or
    /// when a tag's factory lives outside the scanned headless
    /// directory.
    #[serde(default)]
    factory: Option<String>,
    /// When `kind == "Container"`: container definition.
    #[serde(default)]
    container: Option<ContainerOverride>,
    /// When `kind == "ControlFlow"`: the control-flow kind.
    #[serde(default)]
    control_flow: Option<String>,
    /// Force `supports_text_child` to a specific value
    /// (overrides the generator's heuristic).
    #[serde(default)]
    supports_text_child: Option<bool>,
    /// Force `needs_window` to a specific value
    /// (overrides the generator's signature inspector).
    /// Used when the generator's `render_takes_window`
    /// helper misses a `&mut Window` arg (e.g. a
    /// recently-added component whose render signature
    /// the heuristic doesn't yet recognise).
    #[serde(default)]
    needs_window: Option<bool>,
    /// Add children before `.render(cx)` (e.g. `ButtonGroup`, `Modal`).
    #[serde(default)]
    children_before_render: Option<bool>,
    /// For children-before-render leaves whose `.child()` expects
    /// the unwrapped rendered type (`Stateful<Div>`) rather than
    /// `AnyElement`. Set for `ButtonGroup`.
    #[serde(default)]
    unwrap_children: Option<bool>,
    /// Named slots: each entry is `["trigger", "trigger"]`
    /// (XML slot name, builder method name).
    #[serde(default)]
    slots: Option<Vec<[String; 2]>>,
    /// Override / replace the auto-detected prop vocabulary for
    /// this leaf. Each entry is `["xml_name", "setter", "kind"]`
    /// where `kind` is one of the `PropValue` variant names.
    #[serde(default)]
    props: Option<Vec<[String; 3]>>,
    /// Override / replace the auto-detected event vocabulary for
    /// this leaf. Each entry is `["xml_name", "setter"]`.
    #[serde(default)]
    events: Option<Vec<[String; 2]>>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct ContainerOverride {
    #[serde(default)]
    fixed_methods: Vec<[String; 2]>,
}

#[derive(Debug)]
struct Extracted {
    tag: String,
    /// Path to the factory function (e.g. `::yororen_ui::headless::button::button`).
    factory: String,
    extra_args: Vec<ExtraArgInfo>,
    needs_app: bool,
    /// Whether the leaf's `render` method needs a
    /// `&mut Window` argument. Detected by inspecting
    /// the headless source: a render signature of
    /// `fn render(self, ..., &mut Window)` sets this
    /// to `true`. The schema codegen uses it to decide
    /// whether to thread `&mut *window` into the
    /// emitted `.render(...)` call.
    needs_window: bool,
    render: RenderKind,
    props: Vec<PropInfo>,
    events: Vec<(String, String)>,
    supports_text_child: bool,
    children_before_render: bool,
    unwrap_children: bool,
    slots: Vec<SlotInfo>,
    /// Free-form notes (manual overrides needed).
    notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct SlotInfo {
    name: String,
    setter: String,
}

#[derive(Debug, Clone)]
struct ExtraArgInfo {
    kind: ExtraArgKind,
    attr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtraArgKind {
    /// Factory arg is a string-like; the macro should grab
    /// it from the `attr` XML attribute (or inner text).
    Text,
    /// Factory arg is anything else; the macro should pass
    /// the `attr` XML attribute's value verbatim.
    Custom,
    /// Factory arg is a closure (`impl Fn(...)`, `Arc<dyn Fn...>`,
    /// etc.); bare path expressions are auto-wrapped into a
    /// closure by the XML macro.
    Callback,
    /// Factory arg is `usize`; raw string literals are parsed
    /// as decimal integers.
    UInt,
    /// Factory arg is `HeadingLevel`.
    HeadingLevel,
    /// Factory arg is `IconSource`.
    IconSource,
    /// Factory arg is `ImageSource`.
    ImageSource,
    /// Factory arg is `KeybindingInputMode`.
    KeybindingInputMode,
    /// Factory arg is `impl IntoIterator<Item = impl Into<String>>`.
    StringList,
    /// Factory arg is a borrowed reference (e.g. `&FocusHandle`).
    Borrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderKind {
    /// `pub fn render(self, cx: &App) -> …` exists.
    Default,
    /// `pub fn apply(self, el: Div) -> …` exists.
    Apply,
    /// Neither (the headless is rendered exclusively through
    /// the renderer trait's `compose`).
    Compose,
}

#[derive(Debug, Clone)]
struct PropInfo {
    name: String,
    setter: String,
    value: PropValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropValue {
    String,
    Bool,
    Variant,
    /// `f64` setter (e.g. `NumberInput::value`).
    /// The codegen pairs this with `on_change` to emit
    /// a `f64`-typed `XmlBinding::xml_write` call.
    Float64,
    /// `f32` setter (e.g. `Slider::value`).
    /// The codegen pairs this with `on_change` to emit
    /// a `f32`-typed `XmlBinding::xml_write` call.
    Float32,
    /// `usize` setter (e.g. `Table::selected` or
    /// `VirtualList::item_count`).
    UInt,
    /// `BadgeVariant` setter (e.g. `Badge::variant`).
    BadgeVariant,
    /// `HeadingLevel` setter (e.g. `Heading::level`).
    HeadingLevel,
    /// `IconSource` setter (e.g. `Icon::source`, `EmptyState::icon`).
    IconSource,
    /// `ImageSource` setter (e.g. `Image::source`).
    ImageSource,
    /// `KeybindingInputMode` setter (e.g. `KeybindingInput::mode`).
    KeybindingInputMode,
    /// `TreeSelectionMode` setter (e.g. `Tree::selection_mode`).
    TreeSelectionMode,
    /// `Spacing` setter (e.g. `Column::gap`). Resolved to a
    /// theme token at render time.
    Spacing,
    /// `Inset` setter (e.g. `Column::p`). Resolved to a theme
    /// token at render time.
    Inset,
    /// `AlignItems` setter (e.g. `Column::items`).
    AlignItems,
    /// `JustifyContent` setter (e.g. `Column::justify`).
    JustifyContent,
    /// `Length` setter (e.g. `Column::w` / `Column::h`).
    Length,
    /// A gpui color (`Hsla` / `Rgba`).
    Color,
    /// Zero-arg flag setter (`fn X(self) -> Self`).
    Flag,
    /// Not a recognised type — the user must annotate.
    Unknown,
    /// Pass the XML expression through verbatim (no wrapping).
    Custom,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut headless_dir: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut check_only = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--headless" => headless_dir = args.next().map(PathBuf::from),
            "--out" => out_path = args.next().map(PathBuf::from),
            "--check" => check_only = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    // Resolve default paths relative to the workspace root so the
    // binary works no matter what the current working directory is.
    // The manifest dir of this binary is `crates/yororen-ui-xml`,
    // so the workspace root is two levels up.
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|m| {
            PathBuf::from(m)
                .parent()
                .expect("crate manifest has parent")
                .parent()
                .expect("crate manifest parent has parent (workspace root)")
                .to_path_buf()
        })
        .unwrap_or_else(|_| {
            std::env::current_dir().expect("could not determine current directory")
        });
    let headless_dir =
        headless_dir.unwrap_or_else(|| workspace_root.join("crates/yororen-ui-core/src/headless"));
    let out_path = out_path
        .unwrap_or_else(|| workspace_root.join("crates/yororen-ui-xml/src/schema_generated.rs"));

    let mut entries: Vec<Extracted> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();

    // 0. Load overrides (if any). Overrides are read from
    //    `<crate-root>/overrides.toml` — the path is the
    //    dir containing the binary's source file, two
    //    levels up from the manifest.
    let overrides_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("overrides.toml");
    let overrides = load_overrides(&overrides_path);

    // Recursively walk `headless_dir`, collecting every
    // `*.rs` file (except `mod.rs`). Subdirectory layout
    // components (e.g. `headless/layout/column.rs`) become
    // factory paths of the form
    // `::yororen_ui::headless::layout::column`. The
    // relative path from `headless_dir` (minus the `.rs`
    // extension) is the module path.
    fn collect_rs_files(
        dir: &Path,
        prefix: &Path,
        out: &mut Vec<(PathBuf, String)>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, prefix, out)?;
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            if stem == "mod" {
                continue;
            }
            let rel = path
                .strip_prefix(prefix)
                .unwrap_or(&path)
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "::");
            out.push((path, rel));
        }
        Ok(())
    }

    let mut files: Vec<(PathBuf, String)> = Vec::new();
    if let Err(e) = collect_rs_files(&headless_dir, &headless_dir, &mut files) {
        eprintln!("could not read {}: {e}", headless_dir.display());
        std::process::exit(1);
    }
    files.sort_by(|a, b| a.1.cmp(&b.1));

    for (path, module_path) in &files {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                skipped.push((module_path.clone(), format!("read error: {e}")));
                continue;
            }
        };
        let ast = match syn::parse_file(&content) {
            Ok(a) => a,
            Err(e) => {
                skipped.push((module_path.clone(), format!("parse error: {e}")));
                continue;
            }
        };
        // `module_name` (used for tag + PascalCase struct lookup)
        // is the last segment of `module_path`.
        let module_name = module_path.rsplit("::").next().unwrap_or(module_path);
        match extract(&ast, module_name, module_path) {
            Ok(Some(e)) => entries.push(e),
            Ok(None) => skipped.push((module_path.clone(), "no factory found".to_string())),
            Err(reason) => skipped.push((module_path.clone(), reason)),
        }
    }

    entries.sort_by(|a, b| a.tag.cmp(&b.tag));
    let mut entries = apply_overrides(entries, &overrides);
    entries.sort_by(|a, b| a.tag.cmp(&b.tag));

    // 1.5. Append the override entries that *aren't* already
    //      present (e.g. `If`, `ElseIf`, `Else`, `For`,
    //      `Fragment` — pure XML control flow, no headless
    //      equivalent). These come last so they sort to the
    //      end of the generated file.
    for o in &overrides {
        if !entries.iter().any(|e| e.tag == o.tag)
            && let Some(e) = override_to_extracted(o)
        {
            entries.push(e);
        }
    }
    entries.sort_by(|a, b| a.tag.cmp(&b.tag));
    let generated = render_module(&entries, &skipped, &overrides);

    let review_needed: Vec<(String, String)> = entries
        .iter()
        .flat_map(|e| {
            e.notes.iter().filter_map(|note| {
                note.starts_with("review needed:")
                    .then_some((e.tag.clone(), note.clone()))
            })
        })
        .collect();
    if !review_needed.is_empty() {
        eprintln!("Unclassified props must be resolved before the schema can be checked in.");
        for (tag, note) in &review_needed {
            eprintln!("  {tag}: {note}");
        }
        eprintln!("Add an override in overrides.toml (props / slots / children_before_render).");
        std::process::exit(1);
    }

    if check_only {
        let existing = fs::read_to_string(&out_path).unwrap_or_default();
        if existing.trim() != generated.trim() {
            eprintln!("Schema drift detected in {}.", out_path.display());
            eprintln!("Run `cargo run -p yororen_ui_xml --bin gen-schema` to regenerate.");
            std::process::exit(1);
        }
        println!("Schema is up to date ({} entries).", entries.len());
    } else {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&out_path, &generated).unwrap();
        println!(
            "Wrote {} entries to {} (skipped: {}).",
            entries.len(),
            out_path.display(),
            skipped.len()
        );
        for (stem, reason) in &skipped {
            println!("  skipped {stem}: {reason}");
        }
    }
}

fn print_help() {
    eprintln!("Usage: gen-schema [--headless <dir>] [--out <file>] [--check]");
}

fn load_overrides(path: &Path) -> Vec<OverrideEntry> {
    if !path.exists() {
        return Vec::new();
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: could not read {}: {e}", path.display());
            return Vec::new();
        }
    };
    match toml::from_str::<OverridesFile>(&content) {
        Ok(file) => file.override_,
        Err(e) => {
            eprintln!("warning: could not parse {}: {e}", path.display());
            Vec::new()
        }
    }
}

fn apply_overrides(entries: Vec<Extracted>, overrides: &[OverrideEntry]) -> Vec<Extracted> {
    // A `kind = "ControlFlow"` override replaces the
    // auto-generated Leaf entirely — the tag has no headless
    // equivalent and is dispatched by the codegen's
    // control-flow arm. Drop those entries here so the later
    // "append overrides not already present" pass re-adds them
    // via `override_to_extracted`. This is how `VirtualList` /
    // `UniformVirtualList` escape the auto-generated Leaf shape
    // (they own a row-template body + auto-persisted controller).
    let replaced_tags: std::collections::HashSet<&str> = overrides
        .iter()
        .filter(|o| o.kind == "ControlFlow")
        .map(|o| o.tag.as_str())
        .collect();
    let by_tag: BTreeMap<String, Extracted> = entries
        .into_iter()
        .filter(|e| !replaced_tags.contains(e.tag.as_str()))
        .map(|e| (e.tag.clone(), e))
        .collect();
    let mut out: Vec<Extracted> = by_tag
        .into_iter()
        .map(|(tag, mut e)| {
            if let Some(o) = overrides.iter().find(|o| o.tag == tag) {
                if let Some(factory) = &o.factory {
                    e.factory = factory.clone();
                }
                if let Some(true) = o.supports_text_child {
                    e.supports_text_child = true;
                }
                if let Some(false) = o.supports_text_child {
                    e.supports_text_child = false;
                }
                if let Some(v) = o.needs_window {
                    e.needs_window = v;
                }
                if let Some(v) = o.children_before_render {
                    e.children_before_render = v;
                }
                if let Some(v) = o.unwrap_children {
                    e.unwrap_children = v;
                }
                if let Some(slots) = &o.slots {
                    e.slots = slots
                        .iter()
                        .map(|s| SlotInfo {
                            name: s[0].clone(),
                            setter: s[1].clone(),
                        })
                        .collect();
                }
                if let Some(props) = &o.props {
                    // Merge with the auto-detected props: override the
                    // value/setter for known names, append new ones.
                    for p in props.iter().filter(|p| p.len() == 3) {
                        if let Some(value) = parse_prop_value(&p[2]) {
                            if let Some(existing) = e.props.iter_mut().find(|x| x.name == p[0]) {
                                existing.setter = p[1].clone();
                                existing.value = value;
                            } else {
                                e.props.push(PropInfo {
                                    name: p[0].clone(),
                                    setter: p[1].clone(),
                                    value,
                                });
                            }
                        }
                    }
                }
                if let Some(events) = &o.events {
                    e.events = events
                        .iter()
                        .filter(|e| e.len() == 2)
                        .map(|e| (e[0].clone(), e[1].clone()))
                        .collect();
                }

                // Props that are handled by structural overrides
                // (children-before-render or named slots) should not
                // also appear as flat prop setters.
                let mut removed_props: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                if o.children_before_render == Some(true) {
                    removed_props.insert("child".to_string());
                    removed_props.insert("children".to_string());
                }
                if let Some(slots) = &o.slots {
                    removed_props.extend(slots.iter().map(|s| s[0].clone()));
                }
                if !removed_props.is_empty() {
                    e.props.retain(|p| !removed_props.contains(&p.name));
                }

                // Prop names that are now explicitly handled by an
                // override no longer need a "review needed" note.
                let mut cleared: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                if let Some(props) = &o.props {
                    cleared.extend(props.iter().filter_map(|p| p.first().cloned()));
                }
                if let Some(slots) = &o.slots {
                    cleared.extend(slots.iter().map(|s| s[0].clone()));
                }
                if o.children_before_render == Some(true) {
                    cleared.insert("child".to_string());
                    cleared.insert("children".to_string());
                }
                e.notes.retain(|note| {
                    note.strip_prefix("review needed: prop `")
                        .and_then(|rest| rest.split('`').next())
                        .map(|name| !cleared.contains(name))
                        .unwrap_or(true)
                });
            }
            e
        })
        .collect();
    out.sort_by(|a, b| a.tag.cmp(&b.tag));
    out
}

fn override_to_extracted(o: &OverrideEntry) -> Option<Extracted> {
    // Convert a `kind = "Container"` or `kind = "ControlFlow"`
    // override into a synthetic `Extracted`. We don't
    // fabricate full Leaf entries here — for those, the
    // generator's output already covers everything we need.
    match o.kind.as_str() {
        "ControlFlow" => {
            // The render code for these entries is inlined
            // directly by the codegen, not through the
            // schema. We emit a minimal marker so the
            // rendered file is well-formed.
            Some(Extracted {
                tag: o.tag.clone(),
                factory: String::new(),
                extra_args: vec![],
                needs_app: false,
                needs_window: false,
                render: RenderKind::Compose,
                props: vec![],
                events: vec![],
                supports_text_child: false,
                children_before_render: false,
                unwrap_children: false,
                slots: vec![],
                notes: vec![format!("control flow: {:?}", o.control_flow)],
            })
        }
        _ => None,
    }
}

// -- extraction ----------------------------------------------------------------

fn extract(
    ast: &syn::File,
    module_name: &str,
    module_path: &str,
) -> Result<Option<Extracted>, String> {
    // 1. Find the factory function.
    let factory = find_factory(ast, module_name);
    let factory = match factory {
        Some(f) => f,
        None => return Ok(None),
    };

    // 2. Find the *Props struct. The convention is
    //    `<PascalCase(file_stem)>[Suffix]Props`, e.g.
    //    `text_input.rs` → `TextInputProps` and
    //    `progress.rs` → `ProgressBarProps`.
    let struct_prefix = pascal_case(module_name);
    let struct_item = match find_struct(ast, &struct_prefix) {
        Some(s) => s,
        None => return Ok(None),
    };
    let struct_name = struct_item.ident.to_string();

    // 3. Find the impl block.
    let impl_block = find_impl(ast, &struct_name);

    // 4. Extract props, events, render mode.
    let mut props = Vec::new();
    let mut events = Vec::new();
    let mut render = RenderKind::Compose;
    let mut needs_window = false;
    let mut supports_text_child = false;
    let mut notes = Vec::new();

    if let Some(impl_block) = impl_block {
        for item in &impl_block.items {
            if let ImplItem::Fn(method) = item {
                if !is_public(&method.vis) {
                    continue;
                }
                let name = method.sig.ident.to_string();
                // Skip the render/apply/accessor methods — they're
                // handled separately or are public-internal.
                if name == "render" {
                    render = RenderKind::Default;
                    // Inspect the render signature for a
                    // `&mut Window` arg. The signature
                    // shape is one of:
                    //   fn render(self, cx: &App) -> ...
                    //   fn render(self, cx: &mut App) -> ...
                    //   fn render(self, cx: &mut App,
                    //             window: &mut Window) -> ...
                    // We treat the presence of any
                    // `&mut Window` arg (in the last 1
                    // or 2 positions) as a signal.
                    if render_takes_window(&method.sig) {
                        needs_window = true;
                    }
                    continue;
                }
                if name == "apply" {
                    render = RenderKind::Apply;
                    continue;
                }
                if name == "focus_handle" || name == "is_focused" {
                    continue;
                }

                // Setter shape:
                //   * `pub fn X(mut self) -> Self` — zero-arg flag
                //     setter, becomes `PropValue::Flag`.
                //   * `pub fn X(mut self, x: T) -> Self` — single-arg
                //     prop setter; `T` is classified into
                //     `String` / `Bool` / `Variant` / `Unknown`.
                //
                // We require `mut self` by value so that getters
                // (`&self`) and imperative mutators (`&mut self`)
                // are not mistaken for setters.
                let inputs = &method.sig.inputs;
                let (is_self_only, arg) = match inputs.len() {
                    1 => match &inputs[0] {
                        FnArg::Receiver(r) if is_mutable_self_by_value(r) => (true, None),
                        _ => continue,
                    },
                    2 => match (&inputs[0], &inputs[1]) {
                        (FnArg::Receiver(r), FnArg::Typed(pt)) if is_mutable_self_by_value(r) => {
                            (false, Some(&*pt.ty))
                        }
                        _ => continue,
                    },
                    _ => continue,
                };

                if name.starts_with("on_") {
                    // Heuristic: in yororen-ui, every `on_X` setter is
                    // an event listener (closure / `Arc<dyn Fn…>` / a
                    // generic with a `Fn` where-bound). The exact
                    // closure type doesn't matter for the schema;
                    // the codegen passes the closure as-is.
                    events.push((name.clone(), name));
                    continue;
                }

                // `child` / `children` are internal builder methods
                // that store children in the props struct. XML
                // attributes are not the right surface for them —
                // children come from XML child nodes (handled by
                // `apply_post_render_children`). Skip them so they
                // don't appear in the schema as `Unknown` props.
                if name == "child" || name == "children" {
                    continue;
                }

                if is_self_only {
                    props.push(PropInfo {
                        name: name.clone(),
                        setter: name.clone(),
                        value: PropValue::Flag,
                    });
                } else if let Some(arg_ty) = arg {
                    let value = classify_arg(arg_ty);
                    if matches!(value, PropValue::Unknown) {
                        notes.push(format!(
                            "review needed: prop `{name}` has unclassified type `{}`",
                            arg_ty.to_token_stream()
                        ));
                    }
                    props.push(PropInfo {
                        name: name.clone(),
                        setter: name.clone(),
                        value,
                    });
                } else {
                    continue;
                }

                // Heuristic: a `caption` setter implies the
                // component supports `.child("…")` after
                // `.render(…)` (Button only today).
                if name == "caption" {
                    supports_text_child = true;
                }
            }
        }
    }

    // 5. Analyse the factory signature.
    let (extra_args, needs_app) = analyse_factory(&factory.sig)?;
    if !extra_args.is_empty() {
        notes.push(format!("extra_args = {} entries", extra_args.len()));
    }

    // 6. Build the tag. By convention, the tag is the
    //    PascalCase'd file stem (e.g. `button` → `Button`,
    //    `text_input` → `TextInput`).
    let tag = pascal_case(module_name);

    // 7. Filter out internal-only fields (those with no public
    //    setter). The struct is a hint, not a hard requirement,
    //    so we don't error if the struct can't be parsed.
    let _ = struct_item;

    Ok(Some(Extracted {
        tag,
        factory: format!("::yororen_ui::headless::{}::{}", module_path, module_name),
        extra_args,
        needs_app,
        needs_window,
        render,
        props,
        events,
        supports_text_child,
        children_before_render: false,
        unwrap_children: false,
        slots: vec![],
        notes,
    }))
}

/// Inspect a render signature for the presence of a
/// `&mut Window` argument. We only need a rough
/// heuristic — the standard shapes are
/// `fn render(self, cx: &App)`,
/// `fn render(self, cx: &mut App)`, and
/// `fn render(self, cx: &mut App, window: &mut Window)`.
/// The 4-arg `on_toggle` callback shape is not
/// relevant here (it's an event, not the render fn).
///
/// We treat any `&mut Window` arg in the last 1-2
/// positions as a signal — false positives are
/// caught by the rustc compile, so an overly broad
/// heuristic is acceptable.
fn render_takes_window(sig: &Signature) -> bool {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(t) => Some(&*t.ty),
            syn::FnArg::Receiver(_) => None,
        })
        .rev()
        .take(2)
        .any(is_mut_window_ty)
}

fn is_mut_window_ty(ty: &syn::Type) -> bool {
    let s = ty.to_token_stream().to_string();
    // Match `&mut Window` or `&mut gpui :: Window`
    // (we don't bother with the exact path).
    s.replace(' ', "") == "&mutWindow" || s.contains("&mutgpui::Window") || s.contains("&mutWindow")
}

fn find_factory<'a>(ast: &'a syn::File, name: &str) -> Option<&'a ItemFn> {
    for item in &ast.items {
        if let Item::Fn(f) = item
            && f.sig.ident == name
            && is_public(&f.vis)
        {
            return Some(f);
        }
    }
    None
}

fn find_struct<'a>(ast: &'a syn::File, prefix: &str) -> Option<&'a ItemStruct> {
    // Match any `pub struct <Prefix><Maybe>Props { ... }` —
    // e.g. for prefix `Progress`, this catches both
    // `ProgressProps` and `ProgressBarProps`. We pick the
    // *first* matching public struct in the file.
    for item in &ast.items {
        if let Item::Struct(s) = item {
            if !is_public(&s.vis) {
                continue;
            }
            if s.ident.to_string().starts_with(prefix) && s.ident.to_string().ends_with("Props") {
                return Some(s);
            }
        }
    }
    None
}

fn find_impl<'a>(ast: &'a syn::File, struct_name: &str) -> Option<&'a ItemImpl> {
    for item in &ast.items {
        if let Item::Impl(imp) = item
            && let Type::Path(TypePath { qself: None, path }) = &*imp.self_ty
            && path.segments.last().is_some_and(|s| s.ident == struct_name)
            && imp.trait_.is_none()
        {
            return Some(imp);
        }
    }
    None
}

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// True for `mut self` by value (not `&self` or `&mut self`).
/// Setters take ownership and mutate; getters and imperative
/// mutators borrow and must not be treated as schema props.
fn is_mutable_self_by_value(r: &syn::Receiver) -> bool {
    r.reference.is_none() && r.mutability.is_some()
}

#[allow(dead_code)]
fn is_event_closure(ty: &Type) -> bool {
    // Kept around for future use cases that aren't covered
    // by the simpler "method name starts with `on_`"
    // heuristic. The current generator uses the latter.
    let _ = ty;
    false
}

#[allow(dead_code)]
fn is_event_generic(_method: &syn::ImplItemFn) -> bool {
    false
}

fn classify_arg(ty: &Type) -> PropValue {
    // `bool` literal.
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        let n = seg.ident.to_string();
        if n == "bool" {
            return PropValue::Bool;
        }
        if n == "f64" {
            return PropValue::Float64;
        }
        if n == "f32" {
            return PropValue::Float32;
        }
        if n == "usize" {
            return PropValue::UInt;
        }
        // Known variant enums.
        if matches!(
            n.as_str(),
            "ActionVariantKind" | "BuiltinVariantKey" | "SliderSize" | "TagKind"
        ) {
            return PropValue::Variant;
        }
        if n == "BadgeVariant" {
            return PropValue::BadgeVariant;
        }
        if n == "HeadingLevel" {
            return PropValue::HeadingLevel;
        }
        if n == "IconSource" {
            return PropValue::IconSource;
        }
        if n == "ImageSource" {
            return PropValue::ImageSource;
        }
        if n == "KeybindingInputMode" {
            return PropValue::KeybindingInputMode;
        }
        if n == "TreeSelectionMode" {
            return PropValue::TreeSelectionMode;
        }
        if n == "Spacing" {
            return PropValue::Spacing;
        }
        if n == "Inset" {
            return PropValue::Inset;
        }
        if n == "AlignItems" {
            return PropValue::AlignItems;
        }
        if n == "JustifyContent" {
            return PropValue::JustifyContent;
        }
        if n == "Length" {
            return PropValue::Length;
        }
    }
    let rendered = ty.to_token_stream().to_string();
    // `impl Into<gpui::Pixels>` / `Pixels` — `f32` works because
    // `f32: Into<gpui::Pixels>`.
    if rendered.contains("Pixels") {
        return PropValue::Float32;
    }
    // `Hsla` / `Rgba` / `impl Into<Hsla>` — colour values.
    if rendered.contains("Hsla") || rendered.contains("Rgba") {
        return PropValue::Color;
    }
    if rendered.contains("IconSource") {
        return PropValue::IconSource;
    }
    if rendered.contains("ImageSource") {
        return PropValue::ImageSource;
    }
    if rendered.contains("KeybindingInputMode") {
        return PropValue::KeybindingInputMode;
    }
    if rendered.contains("Spacing") {
        return PropValue::Spacing;
    }
    if rendered.contains("Inset") {
        return PropValue::Inset;
    }
    if rendered.contains("AlignItems") {
        return PropValue::AlignItems;
    }
    if rendered.contains("JustifyContent") {
        return PropValue::JustifyContent;
    }
    if rendered.contains("Length") {
        return PropValue::Length;
    }
    // Typed containers / custom objects that cannot be
    // expressed as XML literals. These require a brace
    // expression (rendered as `PropValue::Custom`).
    // Check before the generic `Into` heuristic so that
    // `impl Into<TreeNodeId>` is not misclassified as String.
    if rendered.contains("TreeData")
        || rendered.contains("TreeNodeId")
        || rendered.contains("Option < SharedString >")
        || rendered.starts_with("Vec <")
        || rendered.starts_with("HashMap <")
        || rendered.starts_with("BTreeMap <")
    {
        return PropValue::Custom;
    }
    // `impl Into<SharedString>` / `impl Into<String>` / `&str` / `String`.
    // Exclude `IntoIterator` (e.g. `keys: impl IntoIterator<Item = impl Into<String>>`).
    if !rendered.contains("IntoIterator")
        && (rendered.contains("Into")
            || rendered.contains("SharedString")
            || rendered == "String"
            || rendered == "& str"
            || rendered == "&'static str")
    {
        return PropValue::String;
    }
    PropValue::Unknown
}

fn analyse_factory(sig: &Signature) -> Result<(Vec<ExtraArgInfo>, bool), String> {
    // The factory takes `self`-less args. The first one is
    // always the element id (skipped — we pass it from the
    // `id` XML attribute).
    let args: Vec<&FnArg> = sig
        .inputs
        .iter()
        .filter(|a| !matches!(a, FnArg::Receiver(_)))
        .collect();

    // The last arg might be `&mut App` (or `&mut Context<T>`).
    let (last_is_cx, rest) = match args.last() {
        Some(last) if is_cx_arg(last) => (true, &args[..args.len() - 1]),
        _ => (false, &args[..]),
    };

    // Drop the leading `id` arg.
    let extra_arg_args: &[&FnArg] = if !rest.is_empty() { &rest[1..] } else { &[] };

    let mut extra_args = Vec::new();
    for arg in extra_arg_args {
        let (ty, param_name) = match arg {
            FnArg::Typed(pt) => (&*pt.ty, param_name_from_pat(&pt.pat)),
            _ => return Err("unexpected receiver in factory arg".to_string()),
        };
        let attr = param_name.unwrap_or_else(|| "value".to_string());
        let kind = if is_usize(ty) {
            ExtraArgKind::UInt
        } else if is_heading_level(ty) {
            ExtraArgKind::HeadingLevel
        } else if is_icon_source(ty) {
            ExtraArgKind::IconSource
        } else if is_image_source(ty) {
            ExtraArgKind::ImageSource
        } else if is_keybinding_mode(ty) {
            ExtraArgKind::KeybindingInputMode
        } else if is_string_list(ty) {
            ExtraArgKind::StringList
        } else if is_borrow(ty) {
            ExtraArgKind::Borrow
        } else if is_tree_node_id(ty) {
            // Must precede `is_string_like`: `impl Into<TreeNodeId>`
            // contains `Into` and would otherwise be misclassified as Text.
            ExtraArgKind::Custom
        } else if is_string_like(ty) {
            ExtraArgKind::Text
        } else if is_callback(ty) {
            ExtraArgKind::Callback
        } else {
            ExtraArgKind::Custom
        };
        extra_args.push(ExtraArgInfo { kind, attr });
    }

    Ok((extra_args, last_is_cx))
}

fn is_cx_arg(arg: &FnArg) -> bool {
    let FnArg::Typed(pt) = arg else {
        return false;
    };
    is_app_type(&pt.ty)
}

fn is_app_type(ty: &Type) -> bool {
    if let Type::Reference(tr) = ty
        && tr.mutability.is_some()
        && let Type::Path(tp) = &*tr.elem
        && let Some(seg) = tp.path.segments.last()
    {
        let n = seg.ident.to_string();
        n == "App" || n == "Context"
    } else {
        false
    }
}

fn is_string_like(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    !rendered.contains("IntoIterator")
        && (rendered.contains("Into")
            || rendered.contains("SharedString")
            || rendered == "String"
            || rendered == "& str"
            || rendered == "&'static str")
}

fn is_string_list(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string().replace(' ', "");
    rendered.contains("IntoIterator") && rendered.contains("Into<String>")
}

fn is_borrow(ty: &Type) -> bool {
    matches!(ty, Type::Reference(_))
}

fn is_usize(ty: &Type) -> bool {
    if let Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
    {
        seg.ident == "usize"
    } else {
        false
    }
}

fn is_heading_level(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    rendered.contains("HeadingLevel")
}

fn is_icon_source(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    rendered.contains("IconSource")
}

fn is_image_source(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    rendered.contains("ImageSource")
}

fn is_keybinding_mode(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    rendered.contains("KeybindingInputMode")
}

fn is_tree_node_id(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    rendered.contains("TreeNodeId")
}

fn is_callback(ty: &Type) -> bool {
    let rendered = ty.to_token_stream().to_string();
    // Match `impl Fn(...)`, `Arc<dyn Fn(...)>`, `Box<dyn Fn(...)>`
    // and plain function-pointer-like types. `to_token_stream()`
    // can insert spaces (e.g. `Fn ( ... )`), so normalize before
    // checking.
    rendered.replace(' ', "").contains("Fn(")
}

fn param_name_from_pat(pat: &syn::Pat) -> Option<String> {
    if let syn::Pat::Ident(pi) = pat {
        return Some(pi.ident.to_string());
    }
    None
}

#[allow(dead_code)]
fn snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn parse_prop_value(raw: &str) -> Option<PropValue> {
    match raw {
        "String" => Some(PropValue::String),
        "Bool" => Some(PropValue::Bool),
        "Variant" => Some(PropValue::Variant),
        "Flag" => Some(PropValue::Flag),
        "Unknown" => Some(PropValue::Unknown),
        "Float64" => Some(PropValue::Float64),
        "Float32" => Some(PropValue::Float32),
        "UInt" => Some(PropValue::UInt),
        "BadgeVariant" => Some(PropValue::BadgeVariant),
        "HeadingLevel" => Some(PropValue::HeadingLevel),
        "IconSource" => Some(PropValue::IconSource),
        "ImageSource" => Some(PropValue::ImageSource),
        "KeybindingInputMode" => Some(PropValue::KeybindingInputMode),
        "TreeSelectionMode" => Some(PropValue::TreeSelectionMode),
        "Spacing" => Some(PropValue::Spacing),
        "Inset" => Some(PropValue::Inset),
        "AlignItems" => Some(PropValue::AlignItems),
        "JustifyContent" => Some(PropValue::JustifyContent),
        "Length" => Some(PropValue::Length),
        "Color" => Some(PropValue::Color),
        "Custom" => Some(PropValue::Custom),
        _ => None,
    }
}

// -- rendering -----------------------------------------------------------------

fn render_module(
    entries: &[Extracted],
    skipped: &[(String, String)],
    overrides: &[OverrideEntry],
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "//! Auto-generated by `gen-schema` — DO NOT EDIT.\n\
         //! Regenerate with `cargo run -p yororen_ui_xml --bin gen-schema`.\n\
         //!\n\
         //! Source of truth: `yororen-ui-core/src/headless/*.rs`\n\
         //! plus `yororen-ui-xml/overrides.toml`.\n\
         //! Last regenerated with: {} entries ({} from overrides), {} skipped.\n\
         //!\n\
         //! Skipped files (need manual schema entry or a schema\n\
         //! extension — see `gen_schema.rs` notes):\n",
        entries.len(),
        overrides.len(),
        skipped.len()
    ));
    for (stem, reason) in skipped {
        out.push_str(&format!("//! - `{stem}`: {reason}\n"));
    }
    out.push('\n');
    out.push_str("#![cfg_attr(rustfmt, rustfmt::skip)]\n");
    out.push_str("#![allow(dead_code)]\n\n");
    out.push_str("use crate::schema::{ComponentDef, ComponentKind, ContainerDef, ControlFlowDef, ExtraArg, ExtraArgKind, LeafDef, PropDef, PropValue, RenderMode, SlotDef};\n\n");

    out.push_str("/// Auto-generated schema entries for every leaf component\n");
    out.push_str("/// extracted from the headless source. Combined with the\n");
    out.push_str("/// hand-written `BUILTINS` in `schema.rs` (which holds the\n");
    out.push_str("/// Phase 1 container / control-flow entries), the macro\n");
    out.push_str("/// codegen can resolve any known tag.\n");
    out.push_str("pub static BUILTINS_GENERATED: &[ComponentDef] = &[\n");

    for e in entries {
        if e.factory.is_empty() {
            // Synthetic marker (control-flow pseudo-tag).
            // Rendered as an empty ComponentKind variant.
            continue;
        }
        out.push_str(&render_entry(e));
    }
    out.push_str("];\n");

    // Render the container / control-flow overrides as a
    // second static so they live alongside the generated
    // entries. The codegen consults BOTH tables.
    out.push_str("\n/// Container / control-flow entries sourced from\n");
    out.push_str("/// `overrides.toml`. These have no headless equivalent\n");
    out.push_str("/// (they're XML-only pseudo-tags or layout containers).\n");
    out.push_str("pub static BUILTINS_OVERRIDES: &[ComponentDef] = &[\n");
    for o in overrides {
        if let Some(s) = render_override(o) {
            out.push_str(&s);
        }
    }
    out.push_str("];\n");

    out
}

fn render_override(o: &OverrideEntry) -> Option<String> {
    match o.kind.as_str() {
        "Container" => {
            let container = o.container.as_ref()?;
            let mut fixed = String::new();
            for [attr, method] in &container.fixed_methods {
                fixed.push_str(&format!("({attr:?}, {method:?}), "));
            }
            Some(format!(
                "    ComponentDef {{\n\
                 \x20\x20\x20\x20        tag: {:?},\n\
                 \x20\x20\x20\x20        kind: ComponentKind::Container(ContainerDef {{\n\
                 \x20\x20\x20\x20            fixed_methods: &[{fixed}],\n\
                 \x20\x20\x20\x20            style_hint: \"the gpui Styled trait (`.flex`, `.items_center`, `.gap_3()`, …)\",\n\
                 \x20\x20\x20\x20        }}),\n\
                 \x20\x20\x20\x20        doc: \"from `overrides.toml`\",\n\
                 \x20\x20\x20\x20    }},\n",
                o.tag
            ))
        }
        "ControlFlow" => {
            let cf = o.control_flow.as_deref()?;
            let variant = match cf {
                "If" => "ControlFlowDef::If",
                "ElseIf" => "ControlFlowDef::ElseIf",
                "Else" => "ControlFlowDef::Else",
                "For" => "ControlFlowDef::For",
                "Fragment" => "ControlFlowDef::Fragment",
                "Include" => "ControlFlowDef::Include",
                "Template" => "ControlFlowDef::Template",
                "Slot" => "ControlFlowDef::Slot",
                "Match" => "ControlFlowDef::Match",
                "Case" => "ControlFlowDef::Case",
                "State" => "ControlFlowDef::State",
                "VirtualList" => "ControlFlowDef::VirtualList",
                "UniformVirtualList" => "ControlFlowDef::UniformVirtualList",
                other => {
                    eprintln!("warning: unknown control flow variant `{other}`");
                    return None;
                }
            };
            Some(format!(
                "    ComponentDef {{\n\
                 \x20\x20\x20\x20        tag: {:?},\n\
                 \x20\x20\x20\x20        kind: ComponentKind::ControlFlow({variant}),\n\
                 \x20\x20\x20\x20        doc: \"from `overrides.toml`\",\n\
                 \x20\x20\x20\x20    }},\n",
                o.tag
            ))
        }
        "Leaf" => None, // Leaf entries are produced by the generator.
        _ => None,
    }
}

fn render_entry(e: &Extracted) -> String {
    let mut s = String::new();
    s.push_str("    ComponentDef {\n");
    s.push_str(&format!("        tag: {:?},\n", e.tag));
    s.push_str(&format!("        kind: {},\n", render_kind(e)));
    s.push_str(&format!(
        "        doc: {:?},\n",
        format!(
            "auto-generated from `headless::{}`",
            e.factory.rsplit("::").next().unwrap_or("?")
        )
    ));
    s.push_str("    },\n");
    for note in &e.notes {
        s.push_str(&format!("    // NOTE: {note}\n"));
    }
    s
}

fn render_kind(e: &Extracted) -> String {
    let mode = match e.render {
        RenderKind::Default => "RenderMode::Default",
        RenderKind::Apply => "RenderMode::Apply",
        RenderKind::Compose => "RenderMode::Apply",
    };
    let slots = e
        .slots
        .iter()
        .map(|s| format!("SlotDef {{ name: {:?}, setter: {:?} }}", s.name, s.setter))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "ComponentKind::Leaf(LeafDef {{\n\
         \x20\x20\x20\x20        factory: {:?},\n\
         \x20\x20\x20\x20        extra_args: &[{}],\n\
         \x20\x20\x20\x20        render: {},\n\
         \x20\x20\x20\x20        needs_app: {},\n\
         \x20\x20\x20\x20        needs_window: {},\n\
         \x20\x20\x20\x20        props: &[{}],\n\
         \x20\x20\x20\x20        events: &[{}],\n\
         \x20\x20\x20\x20        supports_text_child: {},\n\
         \x20\x20\x20\x20        children_before_render: {},\n\
         \x20\x20\x20\x20        unwrap_children: {},\n\
         \x20\x20\x20\x20        slots: &[{}],\n\
         \x20\x20\x20\x20    }})",
        e.factory,
        render_extra_args(&e.extra_args),
        mode,
        e.needs_app,
        e.needs_window,
        render_props(&e.props),
        render_events(&e.events),
        e.supports_text_child,
        e.children_before_render,
        e.unwrap_children,
        slots,
    )
}

fn render_extra_args(args: &[ExtraArgInfo]) -> String {
    let mut s = String::new();
    for ea in args {
        let kind = match ea.kind {
            ExtraArgKind::Text => "ExtraArgKind::Text",
            ExtraArgKind::Custom => "ExtraArgKind::Custom",
            ExtraArgKind::Callback => "ExtraArgKind::Callback",
            ExtraArgKind::UInt => "ExtraArgKind::UInt",
            ExtraArgKind::HeadingLevel => "ExtraArgKind::HeadingLevel",
            ExtraArgKind::IconSource => "ExtraArgKind::IconSource",
            ExtraArgKind::ImageSource => "ExtraArgKind::ImageSource",
            ExtraArgKind::KeybindingInputMode => "ExtraArgKind::KeybindingInputMode",
            ExtraArgKind::StringList => "ExtraArgKind::StringList",
            ExtraArgKind::Borrow => "ExtraArgKind::Borrow",
        };
        s.push_str(&format!(
            "ExtraArg {{ kind: {}, attr: {:?} }}, ",
            kind, ea.attr
        ));
    }
    s
}

fn render_props(props: &[PropInfo]) -> String {
    let mut s = String::new();
    for p in props {
        let pv = match p.value {
            PropValue::String => "PropValue::String",
            PropValue::Bool => "PropValue::Bool",
            PropValue::Variant => "PropValue::Variant",
            PropValue::Flag => "PropValue::Flag",
            PropValue::Unknown => "PropValue::Unknown",
            PropValue::Float64 => "PropValue::Float64",
            PropValue::Float32 => "PropValue::Float32",
            PropValue::UInt => "PropValue::UInt",
            PropValue::BadgeVariant => "PropValue::BadgeVariant",
            PropValue::HeadingLevel => "PropValue::HeadingLevel",
            PropValue::IconSource => "PropValue::IconSource",
            PropValue::ImageSource => "PropValue::ImageSource",
            PropValue::KeybindingInputMode => "PropValue::KeybindingInputMode",
            PropValue::TreeSelectionMode => "PropValue::TreeSelectionMode",
            PropValue::Spacing => "PropValue::Spacing",
            PropValue::Inset => "PropValue::Inset",
            PropValue::AlignItems => "PropValue::AlignItems",
            PropValue::JustifyContent => "PropValue::JustifyContent",
            PropValue::Length => "PropValue::Length",
            PropValue::Color => "PropValue::Color",
            PropValue::Custom => "PropValue::Custom",
        };
        s.push_str(&format!(
            "PropDef {{ name: {:?}, setter: {:?}, value: {} }},\n            ",
            p.name, p.setter, pv
        ));
    }
    s
}

fn render_events(events: &[(String, String)]) -> String {
    let mut s = String::new();
    for (xml_attr, setter) in events {
        s.push_str(&format!("({xml_attr:?}, {setter:?}), "));
    }
    s
}

#[allow(dead_code)]
fn _unused(_: &Path, _: &BTreeMap<String, ()>) {}
