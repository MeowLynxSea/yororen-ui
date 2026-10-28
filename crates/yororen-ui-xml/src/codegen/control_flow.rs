use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, format_ident, quote};

use crate::ast::{AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::{ComponentDef, ControlFlowDef};

use crate::codegen::{
    attr::{attr_expr_only, parse_let_bindings},
    codegen_children_as_element, parse_ts,
    scroll_view::codegen_scroll_view,
    templates::{codegen_slot, codegen_template},
    virtual_list::{codegen_uniform_virtual_list, codegen_virtual_list},
};

/// Combine a run of `If` / `ElseIf` / `Else` siblings
/// into a single block expression:
///   `{ if cond1 { body1 } else if cond2 { body2 } else { body3 } }`
///
/// The block is required so the result is a `Div`
/// (which `Div::child` expects) regardless of branch
/// type. The first branch must be `If`; `ElseIf` /
/// `Else` without a leading `If` is a hard error.
pub(crate) fn codegen_if_chain(
    branches: &[AstNode],
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    if branches.is_empty() {
        return Ok(quote! { gpui::div() });
    }
    let mut chain = TokenStream::new();
    for (i, branch) in branches.iter().enumerate() {
        let element = match branch {
            AstNode::Element(e) => e,
            _ => {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    Span::call_site(),
                    "<If>/<ElseIf>/<Else> chain cannot contain non-element nodes",
                ));
            }
        };
        let branch_expr = codegen_if_branch(
            element,
            element_tag_to_branch_kind(&element.tag)?,
            cx,
            source_file,
            user_schema,
        )?;
        chain.append_all(branch_expr);
        // After the first branch, every subsequent one
        // must be ElseIf or Else (the Rust grammar).
        if i == 0 && element.tag != "If" {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                element.span,
                format!("<{}> cannot start a chain — use <If> first", element.tag),
            )
            .at(element.byte_offset));
        }
    }
    // If the chain ends without an <Else>, append a fallback
    // `else { gpui::div() }` so the whole expression always yields
    // an element. This lets `<If condition={...}>...</If>` be used
    // as a child of leaves and containers alike.
    if branches.last().and_then(|b| match b {
        AstNode::Element(e) => Some(e.tag.as_str()),
        _ => None,
    }) != Some("Else")
    {
        chain.append_all(quote! { else { ::gpui::IntoElement::into_any_element(gpui::div()) } });
    }

    Ok(quote! { { #chain } })
}

pub(crate) fn element_tag_to_branch_kind(tag: &str) -> Result<ControlFlowDef, XmlError> {
    Ok(match tag {
        "If" => ControlFlowDef::If,
        "ElseIf" => ControlFlowDef::ElseIf,
        "Else" => ControlFlowDef::Else,
        _ => {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                Span::call_site(),
                format!("unexpected tag in if-chain: <{tag}>"),
            ));
        }
    })
}

pub(crate) fn codegen_control_flow(
    element: &AstElement,
    def: ControlFlowDef,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    match def {
        ControlFlowDef::If | ControlFlowDef::ElseIf | ControlFlowDef::Else => {
            codegen_if_branch(element, def, cx, source_file, user_schema)
        }
        ControlFlowDef::For => codegen_for(element, cx, source_file, user_schema),
        ControlFlowDef::Fragment => codegen_fragment(element, cx, source_file, user_schema),
        ControlFlowDef::Template => codegen_template(element, cx, source_file, user_schema),
        ControlFlowDef::Slot => codegen_slot(element, cx),
        ControlFlowDef::Match => codegen_match(element, cx, source_file, user_schema),
        ControlFlowDef::Case => Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<Case> must appear directly inside a <Match>",
        )
        .at(element.byte_offset)),
        ControlFlowDef::State => codegen_state(element, cx, source_file, user_schema),
        ControlFlowDef::VirtualList => codegen_virtual_list(element, cx, source_file, user_schema),
        ControlFlowDef::UniformVirtualList => {
            codegen_uniform_virtual_list(element, cx, source_file, user_schema)
        }
        ControlFlowDef::ScrollView => codegen_scroll_view(element, cx, source_file, user_schema),
        ControlFlowDef::Include => Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "internal error: <Include> should have been expanded before codegen",
        )),
    }
}

pub(crate) fn codegen_if_branch(
    element: &AstElement,
    kind: ControlFlowDef,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    let condition = if matches!(kind, ControlFlowDef::Else) {
        TokenStream::new()
    } else {
        let cond_attr = element
            .attributes
            .iter()
            .find(|a| a.name == "condition")
            .ok_or_else(|| {
                XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{:?}> requires a `condition={{...}}` attribute", kind),
                )
                .at(element.byte_offset)
            })?;
        let expr = attr_expr_only(cond_attr)?;
        quote! { #expr }
    };

    // Build the body. Multiple children are automatically
    // wrapped in a plain `gpui::div()` so the branch always
    // yields a single `impl IntoElement`.
    if element.children.is_empty() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "if/else branch must contain at least one child",
        )
        .at(element.byte_offset));
    }
    let child_expr = codegen_children_as_element(&element.children, cx, source_file, user_schema)?;

    // Wrap the branch body so every arm has the same concrete
    // element type (`AnyElement`). This keeps if/else chains
    // usable as children of leaves and containers.
    let branch_body = quote! { ::gpui::IntoElement::into_any_element(#child_expr) };
    Ok(match kind {
        ControlFlowDef::If => quote! { if #condition { #branch_body } },
        ControlFlowDef::ElseIf => quote! { else if #condition { #branch_body } },
        ControlFlowDef::Else => quote! { else { #branch_body } },
        // Unreachable
        _ => unreachable!("non-branch kind {:?}", kind),
    })
}

pub(crate) fn codegen_for(
    element: &AstElement,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    let each = element
        .attributes
        .iter()
        .find(|a| a.name == "each")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                "<For> requires an `each={...}` attribute",
            )
            .at(element.byte_offset)
        })?;
    let each_parsed = attr_expr_only(each)?;

    // `<For each={xs} let:item>…</For>` — the `let:item` part
    // sets the loop variable name (defaults to `item` when the
    // attribute is present, `it` when it is absent). Reuse the
    // same resolver as `<VirtualList>` so brace expressions like
    // `let:item={name}` are read from `expr`, not `raw`.
    let (item_ident, index_ident) = parse_let_bindings(element, "item", "i");
    let item_ident = item_ident.unwrap_or_else(|| format_ident!("it"));

    // Whether the user requested an index binding.
    let has_index = element.attributes.iter().any(|a| a.name == "let_index");

    // Optional key: `<For each={xs} key={item.id} let:item>`. When
    // present, the codegen binds a fresh `__key` ident per
    // iteration so the child can use it (e.g. in `id={...}`) and
    // — importantly — the row's wrapper `<Div>` gets its own
    // `id` derived from the key. That gives the row a stable
    // `ElementId` across re-renders: even when the user mutates
    // `each` (insertion / removal / reordering), gpui's keyed
    // state survives because the per-row id is the user's `key`,
    // not the row's `enumerate` index.
    //
    // The codegen therefore:
    //   1. Splits `<For each=… key=…>` into a `(each, key)` pair.
    //   2. Emits `let __key = (key_expr);` per iteration.
    //   3. Wraps the child in `gpui::div().id(format!(…))` so the
    //      wrapper itself is keyed.
    let key_attr = element.attributes.iter().find(|a| a.name == "key");
    if key_attr.is_none() {
        // Emit a compile-time warning for the legacy unkeyed
        // path. Stable proc-macro diagnostics are not available,
        // so we print to stderr; the warning still surfaces
        // during the build and points the user to the fix.
        let location = source_file.map(|f| format!(" in {f}")).unwrap_or_default();
        eprintln!(
            "warning: <For> without `key` uses legacy unkeyed rendering; stateful children may lose identity on reorder{}\n         help: add `key={{unique_expr}}` to give rows a stable identity",
            location
        );
    }
    if let Some(k) = key_attr
        && k.expr.is_none()
    {
        return Err(XmlError::new(
            XmlErrorKind::InvalidExpression,
            k.span,
            "<For key> requires a brace expression, e.g. `key={item.id}`",
        )
        .at(k.byte_offset));
    }
    let key_parsed = match key_attr {
        Some(k) => Some(attr_expr_only(k)?),
        None => None,
    };

    if element.children.is_empty() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<For> must contain at least one child",
        )
        .at(element.byte_offset));
    }
    let child_expr = codegen_children_as_element(&element.children, cx, source_file, user_schema)?;

    // The `<For>` body becomes a Rust `for` loop that appends
    // each row as a `.child(...)`. When a `key` is present the
    // row is wrapped in a `<Div id={format!("for_{key}", …)}>`
    // so gpui's keyed state (and the per-row `TextInputState`
    // inside it) survives reorders.
    let body = match (has_index, key_parsed.is_some()) {
        (true, true) => emit_for_loop(
            &each_parsed,
            &item_ident,
            &index_ident,
            true,
            true,
            &key_parsed.unwrap(),
            &child_expr,
        ),
        (true, false) => emit_for_loop(
            &each_parsed,
            &item_ident,
            &index_ident,
            true,
            false,
            &TokenStream::new(),
            &child_expr,
        ),
        (false, true) => emit_for_loop(
            &each_parsed,
            &item_ident,
            &index_ident,
            false,
            true,
            &key_parsed.unwrap(),
            &child_expr,
        ),
        (false, false) => emit_for_loop(
            &each_parsed,
            &item_ident,
            &index_ident,
            false,
            false,
            &TokenStream::new(),
            &child_expr,
        ),
    };
    Ok(body)
}

/// Emit the runtime loop body for `<For>`. The shape is
/// one of four variants (with or without index, with or
/// without key) — they all build a `gpui::div()` and
/// append each row as a child. The keyed variants wrap
/// each row in a `gpui::div().id(format!(…))` so the row
/// has a stable `ElementId` derived from the key.
pub(crate) fn emit_for_loop(
    each_parsed: &TokenStream,
    item_ident: &proc_macro2::Ident,
    index_ident: &proc_macro2::Ident,
    has_index: bool,
    has_key: bool,
    key_parsed: &TokenStream,
    child_expr: &TokenStream,
) -> TokenStream {
    // We always wrap each row in a fresh `gpui::div()` with
    // an id that is either the key (stable) or a combination
    // of index + key (useful when the child needs both). The
    // key-as-id is what makes per-row state survive
    // reorderings.
    let row_wrapper = if has_key {
        quote! {
            {
                let __row = gpui::div();
                let __row = ::gpui::InteractiveElement::id(
                    __row,
                    format!("for_row_{}", #key_parsed),
                );
                ::gpui::ParentElement::child(__row, #child_expr)
            }
        }
    } else {
        quote! {
            {
                let __row = gpui::div();
                ::gpui::ParentElement::child(__row, #child_expr)
            }
        }
    };

    if has_index {
        quote! {
            {
                let mut __div = gpui::div();
                for (__i, #item_ident) in (#each_parsed).iter().enumerate() {
                    let #index_ident = __i;
                    __div = ::gpui::ParentElement::child(__div, #row_wrapper);
                }
                __div
            }
        }
    } else {
        quote! {
            {
                let mut __div = gpui::div();
                for #item_ident in (#each_parsed).iter() {
                    __div = ::gpui::ParentElement::child(__div, #row_wrapper);
                }
                __div
            }
        }
    }
}

pub(crate) fn codegen_fragment(
    element: &AstElement,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    // A Fragment is just a transparent group of children.
    // When it has multiple children we wrap them in a plain
    // `gpui::div()` so the result is always a single
    // `impl IntoElement`.
    codegen_children_as_element(&element.children, cx, source_file, user_schema)
}

fn codegen_match(
    element: &AstElement,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    let on_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "on")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                "<Match> requires an `on={...}` attribute (the value being matched)",
            )
            .at(element.byte_offset)
        })?;
    let on_parsed = attr_expr_only(on_attr)?;

    let mut arms = TokenStream::new();
    let mut arm_count = 0usize;
    for child in &element.children {
        let arm = match child {
            AstNode::Element(e) if e.tag == "Case" => e,
            AstNode::Element(e) => {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    e.span,
                    format!("<{}> is not a valid arm of <Match> — use <Case>", e.tag),
                )
                .at(e.byte_offset));
            }
            AstNode::Text { .. } | AstNode::Expr { .. } => {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    Span::call_site(),
                    "<Match> arms must be <Case> elements",
                ));
            }
        };
        let pattern_attr = arm
            .attributes
            .iter()
            .find(|a| a.name == "pattern")
            .ok_or_else(|| {
                XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    arm.span,
                    "<Case> requires a `pattern={...}` attribute",
                )
                .at(arm.byte_offset)
            })?;
        let pattern_parsed = if pattern_attr.expr.is_none() && pattern_attr.raw == "_" {
            quote! { _ }
        } else if let Some(_expr) = &pattern_attr.expr {
            // Brace expression — the typical case for
            // pattern-style arms (`Status::Loading`,
            // `Some(x)`, etc.).
            attr_expr_only(pattern_attr)?
        } else {
            // Bare literal pattern: `pattern="0"`,
            // `pattern='"hi"'`, `pattern="true"`. The
            // user provides a Rust-syntax literal as
            // the attribute value; we emit it verbatim.
            let raw = pattern_attr.raw.as_str();
            quote! { #raw }
        };
        if arm.children.is_empty() {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                arm.span,
                "<Case> must contain at least one child",
            )
            .at(arm.byte_offset));
        }
        let body = codegen_children_as_element(&arm.children, cx, source_file, user_schema)?;
        arms.append_all(quote! { #pattern_parsed => { #body }, });
        arm_count += 1;
    }
    if arm_count == 0 {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<Match> must contain at least one <Case>",
        )
        .at(element.byte_offset));
    }
    Ok(quote! { match (#on_parsed) { #arms } })
}

fn codegen_state(
    element: &AstElement,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    let name_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "name")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                "<State> requires a `name=\"...\"` attribute",
            )
            .at(element.byte_offset)
        })?;
    if name_attr.expr.is_some() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            name_attr.span,
            "<State name> requires a literal identifier, not a brace expression",
        )
        .at(name_attr.byte_offset));
    }
    let name_ident = format_ident!("{}", name_attr.raw);

    let default_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "default")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                "<State> requires a `default=\"...\"` attribute (initial value)",
            )
            .at(element.byte_offset)
        })?;
    let default_expr: TokenStream = if let Some(expr) = &default_attr.expr {
        parse_ts(
            expr,
            default_attr.span,
            default_attr.byte_offset,
            "<State default>",
        )?
    } else {
        let raw = default_attr.raw.as_str();
        // Wrap a stringified number / bool in its
        // matching Rust literal form. The convention:
        //   default="0"   → 0
        //   default="0.0" → 0.0
        //   default="true"/"false" → bool
        //   default='"hi"' → "hi"
        // Otherwise we emit the literal as-is, which
        // works for `String::from("…")` etc.
        if raw == "true" {
            quote! { true }
        } else if raw == "false" {
            quote! { false }
        } else if raw.contains('.') && raw.parse::<f64>().is_ok() {
            // Float literal (contains a `.`): emit as f64.
            let lit = raw.parse::<f64>().unwrap();
            quote! { #lit }
        } else if raw.parse::<i64>().is_ok() {
            // Integer literal (no `.`).
            let lit = raw.parse::<i64>().unwrap();
            quote! { #lit }
        } else {
            // Treat as a string — wrap in `String::from`.
            quote! { String::from(#raw) }
        }
    };

    let body = codegen_children_as_element(&element.children, cx, source_file, user_schema)?;

    Ok(quote! {
        {
            let #name_ident = (#cx).new(|_| #default_expr);
            #body
        }
    })
}
