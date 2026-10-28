//! Codegen for `<ScrollView id="…">…</ScrollView>` — a generic
//! scrollable container with renderer-drawn scrollbars.
//!
//! Follows the `<VirtualList>` precedent (see `codegen/
//! virtual_list.rs`): the controller is minted and persisted via
//! `window.use_keyed_state(id, …)` so the host view never carries
//! a controller field — or supplied externally via
//! `controller={…}` (an expression evaluating to a
//! `ScrollViewController`). The children become the scrolled
//! content, accumulated through `ScrollViewProps::child`.
//!
//! Attributes consumed here: `id` (required), `scroll_x` /
//! `scroll_y` / `framed` (bool literal or `{expr}`),
//! `scrollbar="always" | "never"` (or `{expr}`), `on_scroll={…}`,
//! `controller={…}`. Every other attribute is treated as a style
//! pass-through applied to the rendered element (`h`, `w`,
//! `size_full`, …), same vocabulary as the container codegen.

use proc_macro2::TokenStream;
use quote::quote;

use crate::ast::{AstAttribute, AstElement};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::ComponentDef;

use crate::codegen::{
    attr::{attr_expr_only, require_attr_expr},
    codegen_child,
    container::apply_container_attr,
    events::auto_wrap_event_expr,
    parse_ts,
};

pub(crate) const SV_ATTRS: &[&str] = &[
    "id",
    "scroll_x",
    "scroll_y",
    "scrollbar",
    "framed",
    "controller",
    "on_scroll",
];

/// Parse a boolean attribute: `"true"` / `"false"` literals, an
/// empty bare attribute (`scroll_x` ≡ `scroll_x="true"`), or a
/// brace expression.
fn bool_attr_tokens(a: &AstAttribute, tag: &str) -> Result<TokenStream, XmlError> {
    if a.expr.is_some() {
        return attr_expr_only(a);
    }
    match a.raw.as_str() {
        "true" | "" => Ok(quote! { true }),
        "false" => Ok(quote! { false }),
        other => Err(XmlError::new(
            XmlErrorKind::Unsupported,
            a.span,
            format!(
                "<{tag}> boolean attribute must be \"true\"/\"false\" or a brace expression, got `{other}`",
            ),
        )
        .at(a.byte_offset)),
    }
}

pub(crate) fn codegen_scroll_view(
    element: &AstElement,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    // --- required attributes ---------------------------------------
    let id_attr = require_attr_expr(element, "id")?;
    let id_expr = match &id_attr.expr {
        Some(e) => parse_ts(e, id_attr.span, id_attr.byte_offset, "<ScrollView> id")?,
        None => {
            let raw = &id_attr.raw;
            quote! { (#raw).to_string() }
        }
    };

    // --- children become the scrolled content ----------------------
    if element.children.is_empty() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<ScrollView> requires at least one child (the scrolled content)",
        )
        .at(element.byte_offset));
    }
    let mut child_stmts: Vec<TokenStream> = Vec::with_capacity(element.children.len());
    for child in &element.children {
        let body = codegen_child(child, cx, source_file, user_schema)?;
        child_stmts.push(quote! { __props = __props.child(#body); });
    }

    // --- optional bool setters -------------------------------------
    let mut setter_stmts: Vec<TokenStream> = Vec::new();
    for name in ["scroll_x", "scroll_y", "framed"] {
        if let Some(a) = element.attributes.iter().find(|a| a.name == name) {
            let t = bool_attr_tokens(a, element.tag.as_str())?;
            let setter = quote::format_ident!("{name}");
            setter_stmts.push(quote! { __props = __props.#setter(#t); });
        }
    }

    // --- scrollbar visibility --------------------------------------
    if let Some(a) = element.attributes.iter().find(|a| a.name == "scrollbar") {
        let variant = match (a.expr.is_some(), a.raw.as_str()) {
            (true, _) => attr_expr_only(a)?,
            (false, "never") => {
                quote! { ::yororen_ui::headless::scroll_view::ScrollbarVisibility::Never }
            }
            (false, "always" | "") => {
                quote! { ::yororen_ui::headless::scroll_view::ScrollbarVisibility::Always }
            }
            (false, other) => {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    a.span,
                    format!(
                        "<ScrollView> scrollbar must be \"always\" or \"never\", got `{other}`",
                    ),
                )
                .at(a.byte_offset));
            }
        };
        setter_stmts.push(quote! { __props = __props.scrollbar_visibility(#variant); });
    }

    // --- on_scroll event -------------------------------------------
    if let Some(a) = element.attributes.iter().find(|a| a.name == "on_scroll") {
        let expr = attr_expr_only(a)?;
        let wrapped = auto_wrap_event_expr(a, expr, "on_scroll", "ScrollView");
        setter_stmts.push(quote! { __props = __props.on_scroll(#wrapped); });
    }

    // --- controller: external or auto-persisted --------------------
    let controller_init = match element.attributes.iter().find(|a| a.name == "controller") {
        Some(a) => {
            let e = attr_expr_only(a)?;
            quote! { let __snap = (#e).clone(); }
        }
        None => quote! {
            let __entity = window.use_keyed_state(
                #id_expr,
                #cx,
                |_window, _cx| ::yororen_ui::headless::scroll_view::ScrollViewController::new(),
            );
            let __snap = __entity.read(#cx).clone();
        },
    };

    // --- style pass-through ----------------------------------------
    let style_container_def = crate::schema::ContainerDef {
        fixed_methods: &[],
        style_hint: "the gpui Styled trait (`.size_full()`, `.h(...)`, …)",
    };
    let mut style_stmts: Vec<TokenStream> = Vec::new();
    let mut _stateful = false;
    for attr in &element.attributes {
        if SV_ATTRS.contains(&attr.name.as_str()) {
            continue;
        }
        apply_container_attr(
            &mut style_stmts,
            attr,
            style_container_def,
            element,
            &mut _stateful,
        )?;
    }

    Ok(quote! {
        {
            #controller_init
            let mut __props = ::yororen_ui::headless::scroll_view::scroll_view(#id_expr, &__snap, #cx);
            #(#child_stmts)*
            #(#setter_stmts)*
            let mut __el = __props.render(#cx);
            #(#style_stmts)*
            ::gpui::IntoElement::into_any_element(__el)
        }
    })
}
