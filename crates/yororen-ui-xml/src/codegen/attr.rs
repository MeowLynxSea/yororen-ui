use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::{AstAttribute, AstElement};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::PropValue;

use crate::codegen::{
    color::parse_hex_color,
    errors::{invalid_attr, invalid_attr_expr, parse_attr, parse_enum_variant},
    parse_ts,
};

pub(crate) fn parse_let_bindings(
    element: &AstElement,
    item_default: &str,
    index_default: &str,
) -> (Option<proc_macro2::Ident>, proc_macro2::Ident) {
    // Resolve a `let:`-derived attribute's identifier. Brace
    // expressions (`let:index={i}`) carry the name in `expr`
    // (the de-braced body); value-less or string forms fall to
    // `raw`, which the normaliser may have set to `"true"` /
    // empty — those cases yield the default name.
    pub(crate) fn resolve_name(attr: Option<&AstAttribute>, default: &str) -> Option<String> {
        let attr = attr?;
        if let Some(expr) = &attr.expr {
            let trimmed = expr.trim();
            if trimmed.is_empty() {
                Some(default.to_string())
            } else {
                Some(trimmed.to_string())
            }
        } else if attr.raw == "true" || attr.raw.is_empty() {
            Some(default.to_string())
        } else {
            Some(attr.raw.clone())
        }
    }
    let item_ident = resolve_name(
        element.attributes.iter().find(|a| a.name == "let_item"),
        item_default,
    )
    .map(|s| format_ident!("{}", s));
    let index_ident = resolve_name(
        element.attributes.iter().find(|a| a.name == "let_index"),
        index_default,
    )
    .unwrap_or_else(|| index_default.to_string());
    let index_ident = format_ident!("{}", index_ident);
    (item_ident, index_ident)
}
pub(crate) fn require_attr_expr<'a>(
    element: &'a AstElement,
    name: &str,
) -> Result<&'a AstAttribute, XmlError> {
    element
        .attributes
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                format!("<{}> requires a `{name}={{...}}` attribute", element.tag),
            )
            .at(element.byte_offset)
        })
}

// --- helpers ----------------------------------------------------------------

pub(crate) const ACTION_VARIANTS: &[(&str, &str)] = &[
    ("neutral", "Neutral"),
    ("primary", "Primary"),
    ("danger", "Danger"),
];

pub(crate) const BADGE_VARIANTS: &[(&str, &str)] = &[
    ("neutral", "Neutral"),
    ("success", "Success"),
    ("warning", "Warning"),
    ("danger", "Danger"),
    ("info", "Info"),
];

pub(crate) const HEADING_LEVEL_VARIANTS: &[(&str, &str)] = &[
    ("H1", "H1"),
    ("h1", "H1"),
    ("H2", "H2"),
    ("h2", "H2"),
    ("H3", "H3"),
    ("h3", "H3"),
    ("H4", "H4"),
    ("h4", "H4"),
    ("H5", "H5"),
    ("h5", "H5"),
    ("H6", "H6"),
    ("h6", "H6"),
];

pub(crate) const KEYBINDING_INPUT_MODE_VARIANTS: &[(&str, &str)] = &[
    ("Idle", "Idle"),
    ("idle", "Idle"),
    ("Capturing", "Capturing"),
    ("capturing", "Capturing"),
];

pub(crate) const TREE_SELECTION_MODE_VARIANTS: &[(&str, &str)] = &[
    ("None", "None"),
    ("none", "None"),
    ("Single", "Single"),
    ("single", "Single"),
    ("Multiple", "Multiple"),
    ("multiple", "Multiple"),
];

pub(crate) const SPACING_VARIANTS: &[(&str, &str)] = &[
    ("xs", "Xs"),
    ("sm", "Sm"),
    ("md", "Md"),
    ("lg", "Lg"),
    ("xl", "Xl"),
    ("xxl", "Xxl"),
];

pub(crate) const INSET_VARIANTS: &[(&str, &str)] = &[
    ("xs", "Xs"),
    ("sm", "Sm"),
    ("md", "Md"),
    ("lg", "Lg"),
    ("xl", "Xl"),
];

pub(crate) const ALIGN_ITEMS_VARIANTS: &[(&str, &str)] = &[
    ("start", "Start"),
    ("end", "End"),
    ("center", "Center"),
    ("baseline", "Baseline"),
    ("stretch", "Stretch"),
];

pub(crate) const JUSTIFY_CONTENT_VARIANTS: &[(&str, &str)] = &[
    ("start", "Start"),
    ("end", "End"),
    ("center", "Center"),
    ("between", "Between"),
    ("around", "Around"),
    ("evenly", "Evenly"),
];

pub(crate) const LENGTH_VARIANTS: &[(&str, &str)] =
    &[("full", "Full"), ("fit", "Fit"), ("auto", "Auto")];

pub(crate) fn attr_value_tokens(attr: &AstAttribute) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        let parsed = parse_ts(
            expr,
            attr.span,
            attr.byte_offset,
            &format!("attribute `{}`", attr.name),
        )?;
        Ok(quote! { #parsed })
    } else {
        let raw = attr.raw.as_str();
        // Use `.to_string()` so the type is unambiguous
        // (avoids the "multiple From<&str> impls" error when
        // the consumer's `impl Into<...>` is generic).
        Ok(quote! { (#raw).to_string() })
    }
}
pub(crate) fn prop_value_tokens(
    attr: &AstAttribute,
    kind: PropValue,
) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        // Brace expression — use as-is, `.into()` where
        // appropriate. (For `Flag` props, the main loop
        // has already rejected the expression.)
        let parsed = parse_ts(
            expr,
            attr.span,
            attr.byte_offset,
            &format!("attribute `{}`", attr.name),
        )?;
        return Ok(match kind {
            PropValue::String
            | PropValue::Variant
            | PropValue::BadgeVariant
            | PropValue::HeadingLevel
            | PropValue::IconSource
            | PropValue::ImageSource
            | PropValue::KeybindingInputMode
            | PropValue::TreeSelectionMode
            | PropValue::Spacing
            | PropValue::Inset
            | PropValue::AlignItems
            | PropValue::JustifyContent
            | PropValue::Length
            | PropValue::Color
            | PropValue::Bool
            | PropValue::UInt
            | PropValue::Float32
            | PropValue::Float64
            | PropValue::Unknown
            | PropValue::Custom => {
                quote! { #parsed }
            }
            PropValue::Flag => quote! { #parsed /* unreachable */ },
        });
    }
    let raw = attr.raw.as_str();
    match kind {
        PropValue::String => Ok(quote! { (#raw).to_string() }),
        PropValue::Bool => match raw {
            "true" => Ok(quote! { true }),
            "false" => Ok(quote! { false }),
            _ => Err(invalid_attr_expr(attr, "`true` or `false`", raw)),
        },
        PropValue::Flag => {
            // Handled by the codegen's main loop — the
            // `Flag` arm above emits `.setter()` directly
            // and bypasses `prop_value_tokens`. This arm
            // exists for match exhaustiveness.
            Ok(quote! { /* unreachable */ })
        }
        PropValue::Variant => {
            let variant = parse_enum_variant(
                attr,
                raw,
                ACTION_VARIANTS,
                "one of `neutral`, `primary`, `danger`",
            )?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::ActionVariantKind::#variant })
        }
        PropValue::BadgeVariant => {
            let variant = parse_enum_variant(
                attr,
                raw,
                BADGE_VARIANTS,
                "one of `neutral`, `success`, `warning`, `danger`, `info`",
            )?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::badge::BadgeVariant::#variant })
        }
        PropValue::HeadingLevel => {
            let variant = parse_enum_variant(attr, raw, HEADING_LEVEL_VARIANTS, "H1..H6")?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::heading::HeadingLevel::#variant })
        }
        PropValue::IconSource => Ok(quote! {
            ::yororen_ui::headless::icon::IconSource::Builtin((#raw).into())
        }),
        PropValue::ImageSource => Ok(quote! {
            ::yororen_ui::headless::image::ImageSource::Resource((#raw).into())
        }),
        PropValue::KeybindingInputMode => {
            let variant = parse_enum_variant(
                attr,
                raw,
                KEYBINDING_INPUT_MODE_VARIANTS,
                "`Idle` or `Capturing`",
            )?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::keybinding_input::KeybindingInputMode::#variant })
        }
        PropValue::TreeSelectionMode => {
            let variant = parse_enum_variant(
                attr,
                raw,
                TREE_SELECTION_MODE_VARIANTS,
                "`none`, `single`, or `multiple`",
            )?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::tree::TreeSelectionMode::#variant })
        }
        PropValue::Spacing => {
            if let Ok(n) = raw.parse::<f32>() {
                // Bare number = px (unified default).
                Ok(quote! { ::yororen_ui::headless::layout::Spacing::Px(#n) })
            } else if let Some(body) = raw.strip_suffix("px") {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `px`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Spacing::Px(#n) })
            } else if let Some(body) = raw.strip_suffix("rem") {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `rem`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Spacing::Rem(#n) })
            } else {
                let variant = parse_enum_variant(
                    attr,
                    raw,
                    SPACING_VARIANTS,
                    "a number (px), `Npx`, `Nrem`, or `xs`, `sm`, `md`, `lg`, `xl`, `xxl`",
                )?;
                let variant = format_ident!("{variant}");
                Ok(quote! { ::yororen_ui::headless::layout::Spacing::#variant })
            }
        }
        PropValue::Inset => {
            if let Ok(n) = raw.parse::<f32>() {
                Ok(quote! { ::yororen_ui::headless::layout::Inset::Px(#n) })
            } else if let Some(body) = raw.strip_suffix("px") {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `px`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Inset::Px(#n) })
            } else if let Some(body) = raw.strip_suffix("rem") {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `rem`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Inset::Rem(#n) })
            } else {
                let variant = parse_enum_variant(
                    attr,
                    raw,
                    INSET_VARIANTS,
                    "a number (px), `Npx`, `Nrem`, or `xs`, `sm`, `md`, `lg`, `xl`",
                )?;
                let variant = format_ident!("{variant}");
                Ok(quote! { ::yororen_ui::headless::layout::Inset::#variant })
            }
        }
        PropValue::AlignItems => {
            let variant = parse_enum_variant(
                attr,
                raw,
                ALIGN_ITEMS_VARIANTS,
                "`start`, `end`, `center`, `baseline`, or `stretch`",
            )?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::layout::AlignItems::#variant })
        }
        PropValue::JustifyContent => {
            let variant = parse_enum_variant(
                attr,
                raw,
                JUSTIFY_CONTENT_VARIANTS,
                "`start`, `end`, `center`, `between`, `around`, or `evenly`",
            )?;
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::layout::JustifyContent::#variant })
        }
        PropValue::Length => {
            if let Ok(n) = raw.parse::<f32>() {
                // Bare number = px (unified default).
                Ok(quote! { ::yororen_ui::headless::layout::Length::Px(#n) })
            } else if let Some(body) = raw.strip_suffix("px") {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `px`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Length::Px(#n) })
            } else if let Some(body) = raw.strip_suffix("rem") {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `rem`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Length::Rem(#n) })
            } else if let Some(body) = raw.strip_suffix('%') {
                let n = body
                    .parse::<f32>()
                    .map_err(|_| invalid_attr_expr(attr, "a number followed by `%`", raw))?;
                Ok(quote! { ::yororen_ui::headless::layout::Length::Pct(#n) })
            } else {
                let variant = parse_enum_variant(
                    attr,
                    raw,
                    LENGTH_VARIANTS,
                    "`full`, `fit`, `auto`, a number (px), `Npx`, `Nrem`, or `N%`",
                )?;
                let variant = format_ident!("{variant}");
                Ok(quote! { ::yororen_ui::headless::layout::Length::#variant })
            }
        }
        PropValue::Color => {
            // Brace expressions are passed through verbatim above;
            // this arm only handles literal strings. Hex colours are
            // converted to `gpui::rgb` / `gpui::rgba` calls; anything
            // else is rejected with a helpful note.
            parse_hex_color(raw, attr)
        }
        PropValue::Unknown => Err(invalid_attr(
            attr,
            format!(
                "attribute `{}` requires a brace expression because its type is not string-coercible",
                attr.name
            ),
        )),
        PropValue::Custom => {
            if attr.expr.is_some() {
                unreachable!("brace expressions are handled at the top of prop_value_tokens")
            } else {
                Err(invalid_attr(
                    attr,
                    format!(
                        "attribute `{}` requires a brace expression because it is a custom type",
                        attr.name
                    ),
                ))
            }
        }
        PropValue::Float64 => {
            let value = parse_attr::<f64>(attr, "an f64 literal")?;
            let lit = proc_macro2::Literal::f64_suffixed(value);
            Ok(quote! { #lit })
        }
        PropValue::Float32 => {
            let value = parse_attr::<f32>(attr, "an f32 literal")?;
            let lit = proc_macro2::Literal::f32_suffixed(value);
            Ok(quote! { #lit })
        }
        PropValue::UInt => {
            let value = parse_attr::<usize>(attr, "a usize literal")?;
            let lit = proc_macro2::Literal::usize_unsuffixed(value);
            Ok(quote! { #lit })
        }
    }
}

/// Emit the expansion of `@bind={entity}` for a given
/// component. The macro reads the current value via
/// `yororen_ui::headless::XmlBinding::xml_read` and writes
/// the new value back via
/// `yororen_ui::headless::XmlBinding::xml_write`. The
/// trait is what makes `@bind` work for **any** `T` —
/// `Entity<T>` has a blanket impl, but the user can also
/// impl `XmlBinding<MyType>` for their own handle (a
/// wrapper around `Entity<MyForm>`, an `Arc<MyState>`, …)
/// to plug into the same `@bind` sugar.
///
/// We pick the value setter (preferring a `value` /
/// `text` / `checked` named prop) and the change event
/// (preferring `on_change`, then `on_toggle` for
/// boolean-style components). The resulting token stream
pub(crate) fn attr_expr_only(attr: &AstAttribute) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        let parsed = parse_ts(
            expr,
            attr.span,
            attr.byte_offset,
            &format!("attribute `{}`", attr.name),
        )?;
        Ok(parsed)
    } else {
        Err(invalid_attr(
            attr,
            format!(
                "attribute `{}` requires a brace expression, e.g. `{}={{...}}`",
                attr.name, attr.name
            ),
        ))
    }
}
