//! AST → Rust `TokenStream2`.
//!
//! The codegen is the source of truth for the shape of the
//! expanded `xml! { ... }` invocation; the [`crate::schema`]
//! is consulted only for prop / event validation.
//!
//! ## Output type
//!
//! The macro's expansion is an expression that implements
//! `IntoElement`. The root element's natural type is preserved
//! (a container is a `Div`, a leaf with `.render(cx)` becomes a
//! `Stateful<Div>` that we wrap in `.into_any_element()`).
//!
//! ## `cx` injection
//!
//! Every leaf that requires a `&mut App` (basically every
//! built-in in the MVP) needs a way to thread `cx` from the
//! caller's scope into the factory. The macro accepts a
//! leading `cx = <expr>,` clause; if absent, it uses the bare
//! identifier `cx` (assumed to be in scope). For
//! `Render::render` closures the user can write:
//!
//! ```ignore
//! xml! { cx = &mut **cx,
//!     <Column>…</Column>
//! }
//! ```
//!
//! At each factory call site the macro passes `cx_expr` as-is,
//! so the user is responsible for supplying a `&mut App` (the
//! `&mut **cx` pattern from the v0.3 memory file works).
//!
//! ## `&mut **cx` shorthand
//!
//! For the common case (the `cx` token is a `&mut Context<T>`)
//! the macro accepts the form
//!
//! ```ignore
//! xml! { cx,
//!     <Column>…</Column>
//! }
//! ```
//!
//! and generates a local `let __yororen_xml_cx: &mut gpui::App
//! = &mut **cx;` for the factory calls. The borrow ends at the
//! last factory call so the user can still use `cx` for
//! `.read(cx)` etc. afterwards.

use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::ast::{AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::{ComponentDef, ComponentKind};

// Submodules
pub(crate) mod attr;
pub(crate) mod color;
pub(crate) mod container;
pub(crate) mod control_flow;
pub(crate) mod diagnostics;
pub(crate) mod errors;
pub(crate) mod events;
pub(crate) mod includes;
pub(crate) mod leaf;
pub(crate) mod scroll_view;
pub(crate) mod templates;
pub(crate) mod text;
pub(crate) mod virtual_list;

// Explicit imports from submodules used by the dispatch path.
use crate::codegen::{
    attr::attr_value_tokens,
    container::codegen_container,
    control_flow::codegen_control_flow,
    diagnostics::did_you_mean,
    includes::expand_includes,
    leaf::codegen_leaf,
    templates::{collect_templates, expand_template_invocations},
};

/// Parse a string of Rust tokens into a `TokenStream`. Any
/// error is converted into an `XmlError::InvalidExpression`.
/// `byte_offset` is the position in the original XML to
/// surface in the diagnostic; pass `0` if not meaningful.
pub(crate) fn parse_ts(
    src: &str,
    span: Span,
    byte_offset: usize,
    context: &str,
) -> Result<TokenStream, XmlError> {
    src.parse::<TokenStream>().map_err(|e| {
        XmlError::new(
            XmlErrorKind::InvalidExpression,
            span,
            format!("could not parse {context} `{src}`: {e}"),
        )
        .at(byte_offset)
    })
}

/// Compile an `xml! { ... }` invocation.
///
/// `xml_text` is the literal XML the user wrote between the
/// braces. `outer_span` is the span of the literal (the macro
/// entry point supplies the call site). `cx_expr` is the
/// `cx = <expr>` token stream from the optional preamble, or
/// `None` to default to the bare identifier `cx`.
/// `source_file` is the absolute path of the file that
/// invoked the macro (used to resolve relative `<Include
/// src="…">` paths); pass `None` to fall back to the
/// current working directory (the runtime / test path).
/// `user_schema` is an optional slice of user-supplied
/// component definitions (e.g. from
/// `yororen-ui-xml-components.toml`) that augment the
/// built-in schema without modifying the xml crate.
pub fn codegen(
    xml_text: &str,
    outer_span: Span,
    cx_expr: Option<TokenStream>,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    codegen_with_includes(xml_text, outer_span, cx_expr, source_file, user_schema).map(|(ts, _)| ts)
}

/// Same as [`codegen`] but also returns every XML file that was
/// read (the top-level text plus anything pulled in via
/// `<Include src="…">`). The proc-macro uses this to emit
/// `include_str!` directives so Cargo tracks these files as
/// compilation dependencies.
pub fn codegen_with_includes(
    xml_text: &str,
    outer_span: Span,
    cx_expr: Option<TokenStream>,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<(TokenStream, Vec<std::path::PathBuf>), XmlError> {
    let line_starts = crate::parser::line_starts(xml_text);
    let location = crate::parser::LocationTracker {
        line_starts: &line_starts,
        xml: xml_text,
        outer_span,
    };
    let mut element = crate::parser::parse(xml_text, outer_span, &location)?;
    // Resolve `<Include src="…">` before template processing
    // so that shared templates and other compile-time definitions
    // can live in included XML files. Errors inside an included
    // file are rendered with that file's own line/col location.
    let mut included_paths = Vec::new();
    {
        let mut visited = std::collections::HashSet::new();
        expand_includes(&mut element, source_file, &mut visited, &mut included_paths)?;
    }
    // Template pre-pass: collect every `<Template name="…">`
    // in the root's children, then walk the rest of the tree
    // and substitute `<X>` invocations with the template body,
    // replacing each `<Slot>` with the caller's matching
    // content. Templates themselves are dropped from the
    // output (they're compile-time-only).
    let templates = collect_templates(&element)?;
    expand_template_invocations(&mut element, &templates, outer_span)?;
    let cx_tokens = match cx_expr {
        Some(expr) => quote! { (#expr) },
        None => quote! { cx },
    };
    let body = codegen_element(&element, &cx_tokens, source_file, user_schema)?;
    // The generated body uses fully-qualified trait method
    // calls (`::gpui::Styled::#m(__el, …)`,
    // `::gpui::ParentElement::child(__el, …)`, etc.) so the
    // caller does not need to import any gpui traits. The
    // result is a plain block expression returning the root
    // element.
    Ok((quote! { { #body } }, included_paths))
}

pub(crate) fn codegen_element(
    element: &AstElement,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    // Unknown tags fall through to the runtime registry
    // (see `crate::runtime` and the `register_xml_component!`
    // declarative macro). The user gets a working
    // render via inventory lookup — at the cost of
    // losing compile-time attribute / event validation
    // for that tag.
    let def = crate::schema::lookup_component_owned(&element.tag, user_schema)
        .unwrap_or(RUNTIME_LEAF_FALLBACK.clone());

    match def.kind {
        ComponentKind::Container(c) => codegen_container(element, c, cx, source_file, user_schema),
        ComponentKind::Leaf(l) => codegen_leaf(element, l, cx, source_file, true, user_schema),
        ComponentKind::ControlFlow(c) => {
            codegen_control_flow(element, c, cx, source_file, user_schema)
        }
        ComponentKind::RuntimeLeaf => codegen_runtime_leaf(element, cx),
    }
}

/// Sentinel returned by `lookup_component` when the tag
/// is not in the built-in schema. We hand the codegen a
/// `RuntimeLeaf` variant instead of erroring so that
/// custom registered tags compile cleanly.
pub(crate) const RUNTIME_LEAF_FALLBACK: ComponentDef = ComponentDef {
    tag: "<runtime>",
    kind: ComponentKind::RuntimeLeaf,
    doc: "runtime-registered component",
};

/// Render an element whose tag wasn't found in the
/// built-in schema. Emits a runtime lookup against
/// the [`crate::runtime`] registry — works for any
/// tag the user has registered via
/// [`crate::register_xml_component!`].
///
/// This is the "extension hook" for the schema-less
/// path: unknown tags compile (rather than error) and
/// resolve at runtime. The trade-off is that
/// attribute / event validation can't happen at
/// compile time for these tags.
pub(crate) fn codegen_runtime_leaf(
    element: &AstElement,
    cx: &TokenStream,
) -> Result<TokenStream, XmlError> {
    let tag = element.tag.as_str();
    let id_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "id")
        .ok_or_else(|| {
            let builtins = crate::schema::builtin_tags();
            let suggestion = did_you_mean(tag, &builtins);
            let hint = suggestion.map_or_else(
                String::new,
                |s| format!(" — did you mean `<{s}>`?"),
            );
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                format!(
                    "<{tag}> is not a built-in tag and (as a runtime-registered component) requires an `id` attribute{hint}"
                ),
            )
            .at(element.byte_offset)
        })?;
    let id_expr = attr_value_tokens(id_attr)?;
    // We deliberately ignore every other attribute;
    // the user's renderer is responsible for parsing
    // them. This keeps the contract minimal.
    let _ = cx;
    // `render_or_empty` accepts a borrowed `&str`, so
    // emitting the literal tag string is sufficient and
    // no clone or leak is required.
    Ok(quote! {
        ::yororen_ui_xml::runtime::render_or_empty(#tag, #id_expr, #cx)
    })
}
pub(crate) fn codegen_child(
    node: &AstNode,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    match node {
        AstNode::Element(e) => codegen_element(e, cx, source_file, user_schema),
        AstNode::Expr {
            expr,
            span,
            byte_offset,
        } => parse_ts(expr, *span, *byte_offset, "child expression"),
        AstNode::Text { text, .. } => {
            // Text content inside a container is uncommon — only
            // meaningful for `<Button>Click me</Button>` (handled
            // by `supports_text_child`) or `<Label>Hello</Label>`.
            // For all other parents, surface an error.
            Ok(quote! { #text })
        }
    }
}

/// Turn a list of XML children into a single `impl IntoElement`
/// expression.
///
/// - A single child is emitted directly.
/// - Multiple children are wrapped in a plain `gpui::div()` and
///   appended via `.child(...)`, so the result always composes
///   into a parent container.
/// - An empty list yields an empty `gpui::div()`.
///
/// This is the workhorse behind multi-child `<If>`, `<For>` rows,
/// `<Match>` arms, `<State>`, `<Fragment>`, and `<Include>`.
pub(crate) fn codegen_children_as_element(
    children: &[AstNode],
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    if children.is_empty() {
        Ok(quote! { gpui::div() })
    } else if children.len() == 1 {
        codegen_child(&children[0], cx, source_file, user_schema)
    } else {
        let mut stmts: Vec<TokenStream> = Vec::new();
        stmts.push(quote! { let __el = gpui::div(); });
        for child in children {
            let child_expr = codegen_child(child, cx, source_file, user_schema)?;
            stmts.push(quote! {
                let __el = ::gpui::ParentElement::child(__el, #child_expr);
            });
        }
        Ok(quote! {
            {
                #(#stmts)*
                __el
            }
        })
    }
}

/// Codegen a child element without the final `.into_any_element()`
/// wrapper. Used by components whose builder `.child()` expects the
/// rendered leaf type directly (e.g. `ButtonGroup`).
pub(crate) fn codegen_child_unwrapped(
    node: &AstNode,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    match node {
        AstNode::Element(e) => {
            let def = crate::schema::lookup_component_owned(&e.tag, user_schema)
                .unwrap_or(RUNTIME_LEAF_FALLBACK.clone());
            match def.kind {
                ComponentKind::Leaf(l) => codegen_leaf(e, l, cx, source_file, false, user_schema),
                _ => codegen_element(e, cx, source_file, user_schema),
            }
        }
        AstNode::Expr {
            expr,
            span,
            byte_offset,
        } => parse_ts(expr, *span, *byte_offset, "child expression"),
        AstNode::Text { text, .. } => Ok(quote! { #text }),
    }
}

#[cfg(test)]
mod tests;
