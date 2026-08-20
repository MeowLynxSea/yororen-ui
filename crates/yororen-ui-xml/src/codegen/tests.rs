//! End-to-end tests for the codegen. Each test runs the
//! XML through [`crate::parser::parse`] + [`codegen`]
//! and asserts the resulting token stream can be
//! parsed back as valid Rust. We don't actually compile
//! the generated code here (the proc-macro harness does
//! that), but we make sure the tokens are well-formed
//! and contain the expected fragments.

use super::*;
use crate::codegen::events::{
    is_known_key_filter, split_event_modifiers, wrap_event_body_with_modifiers,
};
use crate::codegen::includes::resolve_include_path;
use crate::schema_generated::BUILTINS_OVERRIDES;
use proc_macro2::Span;
fn render(xml: &str) -> String {
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen succeeds");
    ts.to_string()
}

#[test]
fn empty_column() {
    let s = render(r#"<Column col />"#);
    // Column is now a Leaf (auto-generated factory).
    // `col` passes through as `flex_col` via the style
    // pass-through.
    assert!(s.contains("layout :: column :: column"), "{s}");
    assert!(s.contains("flex_col"), "{s}");
}

#[test]
fn column_with_gap_and_padding() {
    let s = render(r#"<Column flex col gap="3" p="4" />"#);
    // `gap="3"` resolves to `Spacing::Px(3f32)`,
    // `p="4"` to `Inset::Px(4f32)`, both via the
    // layout factory. `flex` and `col` pass through
    // as `flex` / `flex_col` on the rendered div.
    assert!(s.contains("layout :: column :: column"), "{s}");
    assert!(s.contains("Spacing :: Px (3f32)"), "{s}");
    assert!(s.contains("Inset :: Px (4f32)"), "{s}");
    assert!(s.contains("flex"), "{s}");
    assert!(s.contains("flex_col"), "{s}");
}

#[test]
fn label_with_text_attribute() {
    let s = render(r#"<Label id="title" text="Hello" strong="true" />"#);
    assert!(s.contains("headless :: label :: label"), "{s}");
    assert!(s.contains("\"title\""), "{s}");
    assert!(s.contains("\"Hello\""), "{s}");
    assert!(s.contains("strong (true)"), "{s}");
}

#[test]
fn label_with_brace_expression() {
    let s = render(r#"<Label id="title" text={value} />"#);
    assert!(s.contains("value"), "{s}");
}

#[test]
fn button_with_on_click_closure() {
    let s = render(r#"<Button id="inc" caption="+" on_click={move |_, _, cx| { x += 1; }} />"#);
    assert!(s.contains("headless :: button :: button"), "{s}");
    assert!(s.contains("caption ((\"+\") . to_string ())"), "{s}");
    assert!(s.contains("on_click"), "{s}");
    assert!(s.contains("x += 1"), "{s}");
}

#[test]
fn button_with_variant() {
    let s = render(r#"<Button id="save" variant="primary" />"#);
    assert!(s.contains("ActionVariantKind :: Primary"), "{s}");
}

#[test]
fn nested_row_inside_column() {
    let s = render(
        r#"<Column flex col>
    <Label id="a" text="A" />
    <Row flex row>
        <Button id="b" caption="B" />
        <Button id="c" caption="C" />
    </Row>
</Column>"#,
    );
    // Child wiring now uses fully-qualified
    // `::gpui::ParentElement::child(__el, ...)`, so we
    // look for the method name rather than the dotted
    // syntax.
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(normalised.contains("child"), "{normalised}");
    // Two `child` calls inside the column for label/row,
    // then two more inside the row.
    assert_eq!(normalised.matches("child").count(), 4, "{normalised}");
}

#[test]
fn div_container_flex_col_gap() {
    let s = render(
        r#"<Div flex_col gap="4" p="4">
        <Label id="a" text="A" />
        <Label id="b" text="B" />
    </Div>"#,
    );
    // Container shorthand attrs should resolve to Styled method calls.
    // Bare numbers now mean px, so `gap="4"` → `.gap(px(4.))`.
    assert!(s.contains("flex_col"), "{s}");
    assert!(s.contains("gap"), "{s}");
    assert!(s.contains("px (4f32)"), "{s}");
    assert!(s.contains("p"), "{s}");
    // Children should be wired via ParentElement::child.
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(normalised.matches("child").count(), 2, "{normalised}");
}

#[test]
fn if_without_else() {
    let s = render(r#"<Column><If condition={show}><Label id="x" text="hi" /></If></Column>"#);
    assert!(s.contains("if"), "{s}");
    assert!(s.contains("show"), "{s}");
}

#[test]
fn if_else_chain() {
    // If/ElseIf/Else are siblings — each is a separate
    // block. The codegen stitches them into a Rust
    // `if/else if/else` chain.
    let s = render(
        r#"<Column>
    <If condition={a}>
        <Label id="x" text="A" />
    </If>
    <ElseIf condition={b}>
        <Label id="y" text="B" />
    </ElseIf>
    <Else>
        <Label id="z" text="C" />
    </Else>
</Column>"#,
    );
    assert!(s.contains("if"), "{s}");
    assert!(s.contains("else if"), "{s}");
    assert!(s.contains("else"), "{s}");
}

#[test]
fn for_loop_with_item() {
    let s = render(
        r#"<Column>
    <For each={items} let:item>
        <Label id="i" text={item.name} />
    </For>
</Column>"#,
    );
    assert!(s.contains("iter ()"), "{s}");
    assert!(s.contains("items"), "{s}");
    // The loop variable is the `let:item` name.
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(normalised.contains("foritem"), "{normalised}");
}

#[test]
fn for_loop_with_custom_item_name() {
    // `let:item={name}` must read the identifier from the
    // de-braced expression, not from the quoted raw value.
    let s = render(
        r#"<Column>
    <For each={items} let:item={name}>
        <Label id="i" text={name.clone()} />
    </For>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalised.contains("forname"),
        "loop variable should be `name`; got {normalised}"
    );
    assert!(
        normalised.contains("name.clone"),
        "body should reference `name`; got {normalised}"
    );
}

#[test]
fn for_loop_with_custom_index_name() {
    // `let:index={idx}` must bind the user's chosen name,
    // not silently fall back to `i`.
    let s = render(
        r#"<Column>
    <For each={items} let:item={name} let:index={idx}>
        <Label id="i" text={format!("{}-{}", name, idx)} />
    </For>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalised.contains("letidx=__i"),
        "index binding should use `idx`; got {normalised}"
    );
}

#[test]
fn for_loop_with_key_wraps_rows_in_keyed_div() {
    // When `<For key={item.id}>` is supplied, each row
    // gets a fresh wrapper `<Div id=format!("for_row_{key}")>`
    // so the row has a stable `ElementId` across reorders.
    // Without this, gpui's per-row `TextInputState` (keyed
    // by ElementId) would be lost when the user mutates
    // the underlying list (e.g. reorders or inserts).
    let s = render(
        r#"<Column>
    <For each={todos} let:item key={item.id}>
        <Checkbox id="cb" @bind={item.done} />
    </For>
</Column>"#,
    );
    // The wrapper div is present and uses the key
    // expression in its id.
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalised.contains("for_row_"),
        "row wrapper should use `for_row_{{}}` id format; got {normalised}"
    );
    // The key expression itself must be in the output
    // (we splice `item.id` into the format! call).
    assert!(
        normalised.contains("item.id"),
        "key expression must be spliced into the wrapper id; got {normalised}"
    );
}

#[test]
fn for_loop_without_key_does_not_emit_keyed_wrapper() {
    // The legacy `<For each={xs} let:item>` (no key)
    // path doesn't pay the per-row `format!` cost — the
    // row wrapper is a plain `gpui::div()`. This keeps
    // existing showcase XMLs compiling unchanged.
    let s = render(
        r#"<Column>
    <For each={items} let:item>
        <Label id="l" text={item.name} />
    </For>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        !normalised.contains("for_row_"),
        "unkeyed <For> must not emit a keyed wrapper; got {normalised}"
    );
}

#[test]
fn for_loop_key_must_be_brace_expression() {
    // A bare `key="…"` is an error — keys must be
    // expressions so they're per-iteration, not static.
    let err = codegen(
        r#"<Column>
    <For each={items} let:item key="static">
        <Label id="l" text="x" />
    </For>
</Column>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("key"), "{}", err.message);
}

#[test]
fn for_loop_each_must_be_brace_expression() {
    // `<For each="literal">` is rejected — `each` is the
    // loop source and has to be a runtime expression so
    // the iterator is re-evaluated each frame.
    let err = codegen(
        r#"<Column>
    <For each="vec![1, 2, 3]" let:item>
        <Label id="l" text={item} />
    </For>
</Column>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("brace expression"), "{}", err.message);
}

#[test]
fn container_flag_with_non_true_literal_is_an_error() {
    // `<Div flex_grow="false">` previously compiled to
    // `__el.flex_grow(__el, "false")` — a confusing
    // "too many arguments" rustc error. Now it's a
    // clear XML-layer diagnostic.
    let err = codegen(
        r#"<Column flex_grow="false" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("flag"), "{}", err.message);
    assert!(err.message.contains("flex_grow"), "{}", err.message);
    assert!(
        err.offset.is_some(),
        "flag error should carry byte_offset; got {:?}",
        err.offset
    );
}

#[test]
fn container_spacing_with_bad_suffix_carries_offset() {
    // `<Div gap="999pxx">` (unrecognised suffix) must surface a
    // diagnostic with `at line N, column M:` rather than
    // the single-line fallback. (`Column` is now a Leaf
    // and accepts any numeric `gap` as `Spacing::Px`; bare
    // numbers on `<Div>` are now interpreted as px too.)
    let err = codegen(
        r#"<Div gap="999pxx" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("invalid value"), "{}", err.message);
    assert!(
        err.offset.is_some(),
        "spacing suffix error should carry byte_offset; got {:?}",
        err.offset
    );
}

#[test]
fn unknown_tag_falls_through_to_runtime_registry() {
    // Unknown tags used to be a hard error; with the
    // runtime registry (`register_xml_component!`)
    // they now compile and resolve at runtime via
    // `runtime::render_or_empty`. The codegen must
    // emit a call into the runtime module rather
    // than erroring.
    let ts =
        codegen(r#"<MyWidget id="x" />"#, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("render_or_empty"), "{s}");
    assert!(s.contains("\"MyWidget\""), "{s}");
}

#[test]
fn unknown_tag_without_id_is_still_an_error() {
    // The runtime registry needs an `id` to call
    // the factory — the codegen still validates
    // this even on the runtime path.
    let err = codegen("<MyWidget />", Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
        "{err:?}"
    );
    assert!(err.message.contains("runtime-registered"));
}

#[test]
fn unknown_attribute_on_leaf_is_an_error() {
    let err = codegen(
        r#"<Label id="x" text="hi" href="bad" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
        "{err:?}"
    );
}

#[test]
fn unknown_attribute_on_container_is_an_error() {
    let err = codegen(
        r#"<Column flex hover="red" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
        "{err:?}"
    );
}

#[test]
fn missing_id_on_leaf_is_an_error() {
    let err = codegen(r#"<Label text="hi" />"#, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
        "{err:?}"
    );
    assert!(err.message.contains("id"));
}

#[test]
fn missing_id_is_a_helpful_message() {
    let err = codegen(
        r#"<Button caption="Save" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("Button"), "{err}");
}

#[test]
fn bad_bool_value_errors() {
    let err = codegen(
        r#"<Label id="x" text="hi" strong="maybe" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(
        err.message.contains("true") || err.message.contains("false"),
        "{err}"
    );
}

#[test]
fn bad_variant_value_errors() {
    let err = codegen(
        r#"<Button id="x" variant="catastrophic" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(
        err.message.contains("primary")
            || err.message.contains("neutral")
            || err.message.contains("danger"),
        "{err}"
    );
}

#[test]
fn xml_parse_error_propagates() {
    let err = codegen("<Column>", Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::ParseError),
        "{err:?}"
    );
}

#[test]
fn diagnostic_carries_byte_offset_and_snippet() {
    // The `<UnknownTag>` on line 3 used to error,
    // but now it falls through to the runtime
    // registry. To still exercise the diagnostic
    // machinery we use a bad attribute value
    // (`variant="catastrophic"`) on a known tag —
    // that produces an `InvalidExpression` error
    // pointing at the offending attribute.
    let xml = "<Column>\n  <Label id=\"a\" text=\"hi\" />\n  <Button id=\"x\" variant=\"catastrophic\" />\n</Column>";
    let err = codegen(xml, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::InvalidExpression),
        "{err:?}"
    );
    assert!(err.offset.is_some(), "error should carry a byte offset");

    // Render the error with a location tracker and
    // assert the multi-line format.
    let line_starts = crate::parser::line_starts(xml);
    let loc = crate::parser::LocationTracker {
        line_starts: &line_starts,
        xml,
        outer_span: Span::call_site(),
    };
    let rendered = err.render_with(Some(&loc));
    assert!(rendered.contains("line 3"), "{rendered}");
    assert!(rendered.contains("variant"), "{rendered}");
    assert!(rendered.contains('^'), "{rendered}");
}

#[test]
fn diagnostic_render_without_location_falls_back() {
    // When no LocationTracker is provided the
    // diagnostic must still be useful.
    let err = codegen(
        r#"<Label id="x" text="hi" href="bad" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    let rendered = err.render_with(None);
    assert!(rendered.contains("href"), "{rendered}");
}

#[test]
fn bad_bool_value_is_a_useful_diagnostic() {
    // Booleans must be `true` / `false`; anything else
    // is a hard error pointing at the offending attr.
    let err = codegen(
        r#"<Label id="x" text="hi" strong="maybe" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.offset.is_some(), "bad-bool error should carry offset");
    let line_starts = crate::parser::line_starts(r#"<Label id="x" text="hi" strong="maybe" />"#);
    let loc = crate::parser::LocationTracker {
        line_starts: &line_starts,
        xml: r#"<Label id="x" text="hi" strong="maybe" />"#,
        outer_span: Span::call_site(),
    };
    let rendered = err.render_with(Some(&loc));
    assert!(
        rendered.contains("true") && rendered.contains("false"),
        "{rendered}"
    );
}

#[test]
fn split_event_modifiers_recognises_dot_suffixes() {
    // Single modifier.
    let (base, mods) = split_event_modifiers("on_key_down.enter").unwrap();
    assert_eq!(base, "on_key_down");
    assert_eq!(mods, vec!["enter"]);
    // Multiple chained modifiers.
    let (base, mods) = split_event_modifiers("on_key_down.ctrl.enter").unwrap();
    assert_eq!(base, "on_key_down");
    assert_eq!(mods, vec!["ctrl", "enter"]);
    // No modifier.
    assert!(split_event_modifiers("on_click").is_none());
    // Malformed names are rejected (no spurious `.`).
    assert!(split_event_modifiers("on_click.").is_none());
    assert!(split_event_modifiers("on_click..enter").is_none());
}

#[test]
fn event_modifier_emits_keystroke_filter_for_known_keys() {
    // Test the helper directly — the schema doesn't
    // currently register `on_key_down` as a built-in
    // event, but the body generator should still
    // produce the right shape around an inner call.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    let body = wrap_event_body_with_modifiers(&["enter"], inner_call, Span::call_site(), "__ev")
        .expect("wrap with .enter");
    let s = body.to_string();
    assert!(s.contains("keystroke"), "{s}");
    assert!(s.contains("enter"), "{s}");
}

#[test]
fn event_modifier_chains_multiple_filters() {
    // Two modifiers wrap the inner call — the call is
    // reached only when both gates pass. `ctrl` uses
    // `__ev.modifiers().control` and `enter` uses
    // `__ev.keystroke().key`; the body contains both
    // accessors.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    let body =
        wrap_event_body_with_modifiers(&["ctrl", "enter"], inner_call, Span::call_site(), "__ev")
            .expect("wrap with .ctrl.enter");
    let s = body.to_string();
    // The outer modifier check is `modifiers().control`.
    assert!(s.contains("modifiers"), "{s}");
    assert!(s.contains("control"), "{s}");
    // The inner key check is `keystroke().key == "enter"`.
    assert!(s.contains("keystroke"), "{s}");
    assert!(s.contains("enter"), "{s}");
}

#[test]
fn event_modifier_stop_emits_stop_propagation() {
    // `.stop` must call `cx.stop_propagation()` so the
    // gpui dispatcher skips ancestor handlers for the
    // same event. Verify the body contains the exact
    // API call.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    let body = wrap_event_body_with_modifiers(&["stop"], inner_call, Span::call_site(), "__ev")
        .expect("wrap .stop");
    let s = body.to_string();
    assert!(s.contains("stop_propagation"), "{s}");
}

#[test]
fn event_modifier_prevent_emits_window_prevent_default() {
    // `.prevent` must call `window.prevent_default()`
    // (a `Window` method) — the closure receives the
    // window as its 2nd arg, so we splice that.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    let body = wrap_event_body_with_modifiers(&["prevent"], inner_call, Span::call_site(), "__ev")
        .expect("wrap .prevent");
    let s = body.to_string();
    assert!(s.contains("prevent_default"), "{s}");
    assert!(s.contains("__window"), "{s}");
}

#[test]
fn event_modifier_shift_uses_modifiers_accessor() {
    // `.shift` should gate on `Modifiers::shift`, not on
    // a (non-existent) keystroke key called "shift".
    // This is the bug the audit fixed: previously
    // `.shift` was treated as a keyboard filter and
    // checked `keystroke().key == "shift"` which never
    // fires.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    let body = wrap_event_body_with_modifiers(&["shift"], inner_call, Span::call_site(), "__ev")
        .expect("wrap .shift");
    let s = body.to_string();
    // The wrapper reads `modifiers().shift`, not a
    // keystroke comparison.
    assert!(s.contains("modifiers"), "{s}");
    assert!(s.contains("shift"), "{s}");
    assert!(
        !s.contains("\"shift\""),
        ".shift must not compile to a key-string compare; got {s}"
    );
}

#[test]
fn event_modifier_alt_and_meta_alias_platform() {
    // `.alt` reads `modifiers().alt`. `.meta` is
    // accepted as a Windows/Linux-friendly alias for
    // `.platform` (the macOS Command key) — both
    // splice to the same `Modifiers::platform` field.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    for mod_name in ["alt", "meta", "platform", "cmd"] {
        let body = wrap_event_body_with_modifiers(
            &[mod_name],
            inner_call.clone(),
            Span::call_site(),
            "__ev",
        )
        .unwrap_or_else(|e| panic!("wrap .{mod_name}: {e}"));
        let s = body.to_string();
        assert!(
            s.contains("modifiers"),
            ".{mod_name} should splice modifiers() access; got {s}"
        );
    }
}

#[test]
fn event_modifier_known_keys_list_includes_arrows_and_fkeys() {
    // Spot-check the well-known key set: arrow keys,
    // F-keys, and navigation keys.
    for k in [
        "enter", "escape", "tab", "up", "down", "f12", "home", "end", "pageup",
    ] {
        assert!(is_known_key_filter(k), "{k} should be a known key");
    }
    // Garbage keys are rejected.
    assert!(!is_known_key_filter("garbage"));
    assert!(!is_known_key_filter("return"));
}

#[test]
fn event_modifier_unknown_modifier_is_an_error() {
    // A typo'd modifier (`.stpo` instead of `.stop`)
    // must surface a clear compile error rather than
    // silently never firing.
    let inner_call: TokenStream =
        syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
    let err = wrap_event_body_with_modifiers(&["stpo"], inner_call, Span::call_site(), "__ev")
        .expect_err("unknown modifier should error");
    assert!(err.message.contains("stpo"), "{}", err.message);
    assert!(err.message.contains("`stop`"), "{}", err.message);
}

#[test]
fn event_modifier_unknown_base_event_is_an_error() {
    // The base event must exist in the schema;
    // `on_key_down` is not a built-in event today,
    // so the modifier dispatch falls through to the
    // unknown-attribute error.
    let xml = r#"<TextInput id="x" on_key_down.enter={move |_, _, _| {}} />"#;
    let err = codegen(xml, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(matches!(
        err.kind,
        crate::error::XmlErrorKind::UnknownAttribute
    ));
    assert!(err.message.contains("on_key_down.enter"));
}

#[test]
fn event_bare_path_is_auto_wrapped_into_closure() {
    // `<Button on_click={controller.increment}>` is
    // a bare path expression — the codegen auto-wraps
    // it into a closure that adapts the standard
    // 3-arg event signature to the user's method.
    let xml = r#"<Button id="x" caption="+" on_click={controller.increment} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    // The pre-cloned receiver is captured by the
    // closure; the method call uses it (not the
    // original `controller`).
    assert!(s.contains("move"), "{s}");
    assert!(s.contains("__auto_clone"), "{s}");
    assert!(s.contains(". increment"), "{s}");
    assert!(s.contains("__ev"), "{s}");
    assert!(s.contains("__window"), "{s}");
    assert!(s.contains("cx"), "{s}");
}

#[test]
fn event_auto_wrap_pre_clones_receiver() {
    // For `controller.method`, the codegen emits
    // `let __auto_clone_N = (controller).clone();`
    // BEFORE the closure, so each handler captures
    // its own clone and the original `controller`
    // can be used by the next handler.
    let xml = r#"<Button id="x" caption="x" on_click={controller.handle} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("(controller) . clone"), "{s}");
    // Two separate clone idents would mean two
    // closures sharing the controller — but for a
    // single button we just need one.
    assert!(s.contains("__auto_clone"), "{s}");
}

#[test]
fn event_multiple_auto_wraps_have_distinct_clone_idents() {
    // Two buttons, each referencing `controller.x`
    // and `controller.y`, must each get their own
    // pre-cloned receiver (otherwise the second
    // closure sees a moved `controller`).
    let xml = r#"
            <Column>
                <Button id="a" caption="a" on_click={controller.handle_a} />
                <Button id="b" caption="b" on_click={controller.handle_b} />
            </Column>
        "#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    // Two distinct `__auto_clone` bindings (proc-macro
    // hygiene via Span::mixed_site).
    assert!(s.matches("__auto_clone").count() >= 2, "{s}");
    assert!(s.contains("handle_a"), "{s}");
    assert!(s.contains("handle_b"), "{s}");
}

#[test]
fn event_closure_passes_through_unwrapped() {
    // When the user writes a closure, the codegen
    // must NOT auto-wrap (otherwise the args would
    // be doubled).
    let xml =
        r#"<Button id="x" caption="x" on_click={move |ev, w, cx| controller.handle(ev, w, cx)} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    // The user's `__arg0` / `__w` / `__cx` placeholder
    // names must NOT appear (they're only used by the
    // auto-wrap path).
    assert!(!s.contains("__arg0"), "{s}");
    assert!(!s.contains("__w"), "{s}");
    assert!(!s.contains("__cx"), "{s}");
    // The user's closure body should pass through.
    assert!(s.contains("controller . handle"), "{s}");
}

#[test]
fn event_call_expression_is_not_wrapped() {
    // `<Button on_click={some_fn()}>` is a call
    // expression (parens present) — it must pass
    // through verbatim, NOT be wrapped.
    let xml = r#"<Button id="x" caption="x" on_click={build_handler()} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(!s.contains("__arg0"), "{s}");
    assert!(s.contains("build_handler ()"), "{s}");
}

#[test]
fn event_method_call_factory_is_pre_cloned() {
    // `<Button on_click={controller.goto(Section::Actions)}>` is a
    // method-call factory: it returns the closure that should be
    // wired up. The receiver must be pre-cloned so the original
    // `controller` remains available for other handlers.
    let xml = r#"<Button id="x" caption="x" on_click={controller.goto(Section::Actions)} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("__auto_clone"), "{s}");
    assert!(s.contains("(controller) . clone"), "{s}");
    assert!(s.contains(". goto (Section :: Actions)"), "{s}");
    // The factory result is passed through, not wrapped again.
    assert!(!s.contains("move | __arg0"), "{s}");
}

#[test]
fn event_modifier_method_call_factory_pre_clones() {
    // `on_click.stop={controller.goto(Section::Actions)}` clones
    // the receiver and invokes the returned closure with the
    // standard event args.
    let xml = r#"<Button id="x" caption="x" on_click.stop={controller.goto(Section::Actions)} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("__auto_clone"), "{s}");
    assert!(s.contains("(controller) . clone"), "{s}");
    assert!(
        s.contains(". goto (Section :: Actions) (__ev , __window , cx)"),
        "{s}"
    );
}

#[test]
fn event_modifier_call_chain_passes_through() {
    // `on_click.stop={controller.method(args)(extra)}` is an
    // already-invoked call chain. The codegen must NOT append
    // `(__ev, __window, cx)` to it.
    let xml = r#"<Button id="x" caption="x" on_click.stop={controller.method(args)(extra)} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    // The chain call itself must appear verbatim.
    assert!(s.contains(". method (args) (extra)"), "{s}");
    // No duplicated event args after the chain.
    assert!(!s.contains("(extra) (__ev , __window , cx)"), "{s}");
    // The modifier wrapper still runs.
    assert!(s.contains("stop_propagation"), "{s}");
}

#[test]
fn event_macro_call_passes_through() {
    // Macro invocations like `handler!()` are parsed and passed
    // through verbatim; the old string heuristic is no longer used.
    let xml = r#"<Button id="x" caption="x" on_click={handler!()} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("handler ! ()"), "{s}");
    // Not auto-wrapped into a fresh closure.
    assert!(!s.contains("__arg0"), "{s}");
}

#[test]
fn event_reference_passes_through() {
    // `&controller.handle` is not a bare path; it should be parsed
    // and passed through without being auto-wrapped.
    let xml = r#"<Button id="x" caption="x" on_click={&controller.handle} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("& controller . handle"), "{s}");
    assert!(!s.contains("__auto_clone"), "{s}");
}

#[test]
fn tree_item_on_toggle_uses_click_event_signature() {
    // `on_toggle` on TreeItem is a click-style event
    // (`&ClickEvent, &mut Window, &mut App`), not the
    // 4-arg boolean toggle used by Checkbox/Switch.
    // Bare controller methods are auto-wrapped with the
    // click signature, so the method must match it.
    let xml = r#"<TreeItem id="x" node_id="x" label="x" on_toggle={controller.handle_toggle} />"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("on_toggle"), "{s}");
    assert!(s.contains("__ev : & gpui :: ClickEvent"), "{s}");
    assert!(!s.contains("__arg0 : bool"), "{s}");
}

#[test]
fn match_emits_rust_match_with_cases() {
    // `<Match on={status}>` with two `<Case>` arms
    // becomes `match status { A => { … }, B => { … } }`.
    let xml = r#"
            <Match on={status}>
                <Case pattern={Status::Loading}>
                    <Label id="l" text="Loading..." />
                </Case>
                <Case pattern={Status::Ready}>
                    <Label id="r" text="Ready" />
                </Case>
            </Match>
        "#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("match"), "{s}");
    assert!(s.contains("Status :: Loading"), "{s}");
    assert!(s.contains("Status :: Ready"), "{s}");
}

#[test]
fn match_supports_wildcard_via_underscore_literal() {
    // `pattern="_"` is the conventional wildcard;
    // the macro turns it into a Rust `_` pattern.
    // For literal patterns like `pattern={0}` the
    // user uses a brace expression so the integer
    // literal isn't mistaken for a string.
    let xml = r#"
            <Match on={n}>
                <Case pattern={0}>
                    <Label id="z" text="zero" />
                </Case>
                <Case pattern="_">
                    <Label id="o" text="other" />
                </Case>
            </Match>
        "#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("0 =>"), "{s}");
    assert!(s.contains("_ =>"), "{s}");
}

#[test]
fn match_without_cases_is_an_error() {
    let xml = r#"<Match on={x} />"#;
    let err = codegen(xml, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::Unsupported),
        "{err:?}"
    );
    assert!(err.message.contains("at least one"));
}

#[test]
fn case_outside_match_is_an_error() {
    let xml = r#"<Column><Case pattern={A}><Label id="x" text="hi" /></Case></Column>"#;
    let err = codegen(xml, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::Unsupported),
        "{err:?}"
    );
}

#[test]
fn state_emits_cx_new_with_default() {
    // `<State name="count" default="0">` becomes
    // `let count = (cx).new(|_| 0); <child>`.
    let xml = r#"
            <State name="count" default="0">
                <Label id="l" text={count.read(cx).to_string()} />
            </State>
        "#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("let count"), "{s}");
    assert!(s.contains(". new"), "{s}");
    assert!(s.contains("count . read"), "{s}");
}

#[test]
fn state_default_handles_bool_and_string() {
    // Bool literal.
    let xml = r#"<State name="on" default="true"><Label id="l" text="x" /></State>"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("true"), "{s}");
    // String literal.
    let xml = r#"<State name="name" default="anonymous"><Label id="l" text="x" /></State>"#;
    let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen ok");
    let s = ts.to_string();
    assert!(s.contains("String :: from"), "{s}");
    assert!(s.contains("anonymous"), "{s}");
}

#[test]
fn state_without_default_is_an_error() {
    let xml = r#"<State name="x"><Label id="l" text="hi" /></State>"#;
    let err = codegen(xml, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
        "{err:?}"
    );
}

#[test]
fn generated_schema_picks_up_new_components() {
    // The generated schema (`BUILTINS_GENERATED`) is
    // included by `lib.rs` and the codegen falls back to
    // it for any tag not in the hand-written BUILTINS.
    // We assert that a handful of high-frequency components
    // are reachable through the lookup.
    for tag in [
        "Checkbox",
        "Switch",
        "TextInput",
        "Avatar",
        "Badge",
        "Card",
        "Icon",
        "Tag",
        "Progress",
        "Slider",
        "Radio",
        "ToggleButton",
        "GridView",
    ] {
        assert!(
            crate::schema_generated::BUILTINS_GENERATED
                .iter()
                .any(|c| c.tag == tag),
            "tag {tag:?} should be present in BUILTINS_GENERATED"
        );
    }
}

#[test]
fn checkbox_codegen_routes_to_generated_schema() {
    // `<Checkbox checked on_toggle={...} />` should
    // expand to a call into `headless::checkbox` and
    // emit both the `checked` prop and the `on_toggle`
    // event — proving the codegen → generated-schema
    // path is wired.
    let s = render(
        r#"<Checkbox id="agree" checked="true" on_toggle={move |v, _, _, _| { let _ = v; }} />"#,
    );
    assert!(s.contains("headless :: checkbox :: checkbox"), "{s}");
    assert!(s.contains("checked (true)"), "{s}");
    assert!(s.contains("on_toggle"), "{s}");
}

#[test]
fn text_input_codegen_uses_generated_schema() {
    // TextInput factory doesn't take `cx` — the
    // generated schema sets `needs_app: false`. The
    // generated call must therefore omit the trailing
    // `cx` argument.
    let s = render(
        r#"<TextInput id="name" placeholder="Your name" on_change={move |v, _, _| { let _ = v; }} />"#,
    );
    assert!(s.contains("headless :: text_input :: text_input"), "{s}");
    // Needs-app = false → no `, cx` after the args.
    assert!(
        !s.contains("text_input ((\"name\") . to_string () , cx)"),
        "{s}"
    );
    assert!(s.contains("text_input ((\"name\") . to_string ())"), "{s}");
    assert!(s.contains("on_change"), "{s}");
}

#[test]
fn string_interpolation_in_text_attr() {
    let s = render(r#"<Label id="x" text="Count: {count}" />"#);
    assert!(s.contains("format !"), "{s}");
    // The format string is `Count: {}` (one
    // placeholder, no literal braces to escape).
    assert!(s.contains("Count: {}"), "{s}");
    assert!(s.contains("count"), "{s}");
}

#[test]
fn utf8_chars_in_string_attrs_preserved_in_codegen() {
    // Multi-byte UTF-8 characters in string-valued
    // attributes must round-trip through the
    // preprocessor + quote! unchanged, so the
    // resulting Rust source contains the same
    // bytes the user wrote in the XML.
    let s = render(r#"<Label id="x" text="Type here…" />"#);
    // The codegen emits `("Type here…").to_string()`
    // (3 bytes 0xE2 0x80 0xA6 for `…`).
    assert!(s.contains("Type here"), "{s}");
    // The raw `…` byte sequence should survive
    // unchanged. If the codegen mangles UTF-8
    // strings, this assertion fails.
    let ellipsis_bytes = "\u{2026}".as_bytes();
    let s_bytes = s.as_bytes();
    let mut found = false;
    for window in s_bytes.windows(ellipsis_bytes.len()) {
        if window == ellipsis_bytes {
            found = true;
            break;
        }
    }
    assert!(found, "ellipsis bytes not preserved in: {s}");
}

#[test]
fn string_interpolation_with_no_braces_uses_literal_path() {
    // `text="hello"` has no braces, so the fast path
    // emits `("hello").to_string()` — no `format!`.
    let s = render(r#"<Label id="x" text="hello" />"#);
    assert!(s.contains("\"hello\""), "{s}");
    assert!(s.contains("to_string ()"), "{s}");
    assert!(!s.contains("format !"), "{s}");
}

#[test]
fn string_interpolation_multiple_segments() {
    let s = render(r#"<Label id="x" text="x{a}y{b}z" />"#);
    assert!(s.contains("format !"), "{s}");
    // 2 placeholders, no literal braces to escape.
    assert!(s.contains("\"x{}y{}z\""), "{s}");
    assert!(s.contains("a"), "{s}");
    assert!(s.contains("b"), "{s}");
}

#[test]
fn bind_attribute_on_text_input() {
    // `@bind={entity}` on TextInput emits the
    // on_change write-back closure. (TextInput
    // doesn't expose a `value` setter — its value
    // lives in the `Entity<TextInputState>` that the
    // renderer mints internally — so we just verify
    // the on_change side of the binding here.)
    let s = render(r#"<TextInput id="x" @bind={self.name} placeholder="Name" />"#);
    // Strip spaces to make the assertion robust
    // against `quote!`'s token-spacing behaviour.
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("placeholder"), "{s}");
    // The codegen goes through the `XmlBinding` trait
    // for both the read and the write side. The
    // read side emits `xml_read` (text_input's `value`
    // setter now exists; the renderer mints the state
    // with the supplied initial value). The write
    // side emits `xml_write` for `on_change`.
    assert!(compact.contains("xml_read"), "{s}");
    assert!(compact.contains("xml_write"), "{s}");
    assert!(compact.contains("on_change"), "{s}");
}

#[test]
fn bind_attribute_emits_value_read_for_components_with_value_setter() {
    // Checkbox has a `checked` setter + `on_toggle`
    // event. `@bind` emits a `XmlBinding::xml_read`
    // call into the `checked` setter and a
    // write-back closure via `XmlBinding::xml_write`
    // in `on_toggle`. The codegen no longer touches
    // `Entity::read` / `Entity::update` directly —
    // all access goes through the trait so user
    // impls (a wrapper handle around a complex
    // entity) get picked up automatically.
    let s = render(r#"<Checkbox id="x" @bind={self.flag} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("xml_read"), "{s}");
    assert!(compact.contains("xml_write"), "{s}");
    assert!(compact.contains("on_toggle"), "{s}");
}

#[test]
fn bind_on_text_input_emits_value_setter() {
    // `@bind={self.name}` on `<TextInput>` emits both:
    //   1. `.value(XmlBinding::xml_read(&entity, cx))`
    //      — read the current value of the bound
    //      entity and pass it to the setter.
    //   2. `.on_change({ … XmlBinding::xml_write(&entity, …) })`
    //      — write the new value back when the user
    //      edits the input.
    let s = render(r#"<TextInput id="x" @bind={self.name} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    // The read side: `XmlBinding::xml_read` is called
    // and the result fed to `.value(…)`.
    assert!(compact.contains("xml_read"), "{s}");
    assert!(compact.contains(".value("), "{s}");
    // The write side: `xml_write` is in the
    // `on_change` closure.
    assert!(compact.contains("xml_write"), "{s}");
    assert!(compact.contains("on_change"), "{s}");
}

#[test]
fn bind_attribute_on_checkbox_uses_bool_writeback() {
    // Checkbox's `checked` prop is `Bool` and its
    // `on_toggle` passes the new boolean state. `@bind`
    // must emit `XmlBinding::<bool>::xml_write`.
    let s = render(r#"<Checkbox id="x" @bind={self.flag} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("XmlBinding::<bool>"), "{s}");
    assert!(compact.contains("xml_write"), "{s}");
    assert!(compact.contains("on_toggle"), "{s}");
}

#[test]
fn bind_attribute_on_tree_item_toggles_bool_on_click() {
    // TreeItem's `on_toggle` is a click callback, not a
    // boolean setter. `@bind` must read the current
    // boolean value, negate it, and write it back.
    let s = render(r#"<TreeItem id="x" node_id="a" label="A" @bind={self.selected} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("xml_read"), "{s}");
    assert!(compact.contains("xml_write"), "{s}");
    assert!(compact.contains("on_toggle"), "{s}");
    assert!(compact.contains("!__current"), "{s}");
}

#[test]
fn event_modifier_on_toggle_checkbox_uses_four_args() {
    // `on_toggle.stop` on Checkbox must emit a 4-arg
    // closure matching the headless setter.
    let s = render(r#"<Checkbox id="x" checked="true" on_toggle.stop={handler} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("__arg0:bool"), "{s}");
    assert!(compact.contains("Option<&gpui::ClickEvent>"), "{s}");
    assert!(compact.contains("stop_propagation"), "{s}");
}

#[test]
fn event_modifier_on_toggle_tree_item_uses_three_args() {
    // `on_toggle.stop` on TreeItem must emit a 3-arg
    // click-style closure.
    let s = render(r#"<TreeItem id="x" node_id="a" label="A" on_toggle.stop={handler} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(!compact.contains("__arg0:bool"), "{s}");
    assert!(compact.contains("&gpui::ClickEvent"), "{s}");
    assert!(compact.contains("stop_propagation"), "{s}");
}

#[test]
fn bind_macro_in_event_attr_expands_with_signature() {
    // `bind!(controller.increment, 1)` should be expanded by
    // the XML macro into a closure that matches the event.
    let s = render(r#"<Button id="x" caption="x" on_click={bind!(controller.increment, 1)} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("__auto_clone"), "{s}");
    assert!(compact.contains("controller"), "{s}");
    assert!(compact.contains(".increment(1"), "{s}");
}

#[test]
fn bind_macro_in_event_modifier_expands_with_signature() {
    // `on_click.stop={bind!(controller.increment, 1)}` clones
    // the receiver and inserts the bound arg before event args.
    let s =
        render(r#"<Button id="x" caption="x" on_click.stop={bind!(controller.increment, 1)} />"#);
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("__auto_clone"), "{s}");
    assert!(compact.contains("stop_propagation"), "{s}");
    assert!(compact.contains(".increment(1"), "{s}");
}

#[test]
fn template_requires_name_attribute() {
    // `<Template>` without `name` is an error — the
    // tag's whole point is to define a *named*
    // template that the rest of the file can call.
    let err = codegen(
        r#"<Column>
    <Template>
        <Label id="a" text="A" />
    </Template>
</Column>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("name"), "{}", err.message);
}

#[test]
fn template_is_dropped_from_output() {
    // `<Template>` is compile-time-only; the
    // generated code must NOT emit anything for the
    // definition itself, only for its callers.
    let s = render(
        r#"<Column>
    <Template name="X">
        <Label id="a" text="A" />
    </Template>
</Column>"#,
    );
    // The Column should be empty — the Template was
    // dropped, leaving no children.
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(!compact.contains("label::label"), "{s}");
}

#[test]
fn template_invocation_substitutes_default_slot() {
    // A `<X>…</X>` call inlines the template body, with
    // the caller's children replacing the default
    // `<Slot/>` placeholder. The template's wrapping
    // `<Div>` is preserved.
    let s = render(
        r#"<Column>
    <Template name="Card">
        <Div>
            <Slot/>
        </Div>
    </Template>
    <Card>
        <Label id="body" text="Hello" />
    </Card>
</Column>"#,
    );
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    // The caller's Label must appear in the output.
    assert!(compact.contains("label::label"), "{s}");
    assert!(compact.contains("Hello"), "{s}");
    // The wrapping div is preserved too.
    assert!(compact.contains("gpui::div()"), "{s}");
}

#[test]
fn template_invocation_substitutes_named_slot() {
    // `<Slot name="header"/>` in the template body is
    // replaced by the caller's `<Slot name="header">…</Slot>`
    // content; the default slot goes to the unnamed
    // children of the call.
    let s = render(
        r#"<Column>
    <Template name="Card">
        <Div>
            <Slot name="header"/>
            <Slot/>
        </Div>
    </Template>
    <Card>
        <Slot name="header">
            <Label id="h" text="Title" />
        </Slot>
        <Label id="body" text="Hello" />
    </Card>
</Column>"#,
    );
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    // Both the header and the body must be present.
    assert!(compact.contains("Title"), "{s}");
    assert!(compact.contains("Hello"), "{s}");
}

#[test]
fn template_duplicate_name_is_an_error() {
    // Two `<Template name="X">` in the same file is
    // ambiguous — the second definition wins silently,
    // which is a footgun. We error explicitly.
    let err = codegen(
        r#"<Column>
    <Template name="X">
        <Label id="a" text="A" />
    </Template>
    <Template name="X">
        <Label id="b" text="B" />
    </Template>
</Column>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("duplicate"), "{}", err.message);
}

#[test]
fn slot_in_root_is_a_no_op_when_no_template_invocation() {
    // `<Slot/>` at the root (outside any template
    // invocation) is meaningless and just disappears —
    // it has no template to be substituted into. The
    // surrounding Column's child chain is preserved.
    let s = render(r#"<Column><Slot/></Column>"#);
    assert!(s.contains("layout :: column :: column"), "{s}");
}

#[test]
fn include_requires_src() {
    let err = codegen(
        r#"<Column><Include /></Column>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("src"), "{err}");
}

#[test]
fn include_resolves_relative_to_source_file() {
    // The resolver should join a relative `<Include
    // src="…">` against the source file's parent
    // directory, not the current working directory.
    use std::path::PathBuf;
    let resolved =
        resolve_include_path("ui/header.xml", Some("/home/dev/proj/src/view.rs")).expect("resolve");
    assert_eq!(resolved, PathBuf::from("/home/dev/proj/src/ui/header.xml"));
}

#[test]
fn include_passes_absolute_paths_through() {
    // Absolute `src` paths skip the join and are
    // used verbatim.
    use std::path::PathBuf;
    let resolved =
        resolve_include_path("/etc/foo.xml", Some("/home/dev/proj/src/view.rs")).expect("resolve");
    assert_eq!(resolved, PathBuf::from("/etc/foo.xml"));
}

#[test]
fn include_falls_back_to_cwd_without_source_file() {
    // When the caller doesn't supply a source file
    // (the runtime loader, or a test), the resolver
    // falls back to the current working directory —
    // matching the behaviour tests rely on.
    use std::path::PathBuf;
    let resolved = resolve_include_path("ui/header.xml", None).expect("resolve");
    assert_eq!(resolved, PathBuf::from("./ui/header.xml"));
}

#[test]
fn include_provides_templates_to_the_including_file() {
    // Templates defined in an included XML file must be
    // visible to invocations in the parent file. This lets
    // shared layout components live in a single place.
    use std::fs;

    let dir = std::env::temp_dir().join("yororen_ui_xml_include_template_test");
    fs::create_dir_all(&dir).expect("create temp dir");

    let shared = dir.join("shared.xml");
    let main = dir.join("main.xml");
    fs::write(
        &shared,
        r#"<Fragment>
    <Template name="Card">
        <Div>
            <Slot name="title"/>
            <Slot/>
        </Div>
    </Template>
</Fragment>"#,
    )
    .expect("write shared.xml");
    fs::write(
        &main,
        r#"<Column>
    <Include src="shared.xml" />
    <Card>
        <Slot name="title"><Label id="t" text="Title" /></Slot>
        <Label id="b" text="Body" />
    </Card>
</Column>"#,
    )
    .expect("write main.xml");

    let source = dir.join("view.rs");
    let contents = fs::read_to_string(&main).expect("read main.xml");
    let ts = codegen(&contents, Span::call_site(), None, source.to_str(), &[])
        .expect("codegen should succeed");
    let s = ts.to_string();
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(compact.contains("Title"), "{s}");
    assert!(compact.contains("Body"), "{s}");
}

#[test]
fn bind_attribute_without_braces_errors() {
    let err = codegen(
        r#"<TextInput id="x" @bind="not_an_expr" placeholder="…" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("@bind"), "{err}");
}

/// CI gate for the generator output. Replaces the old
/// `hand_written_button_matches_generated` test: now that
/// `Button` lives in the generated schema, we verify the
/// invariants the generator must maintain instead of comparing
/// against a hand-written copy.
#[test]
fn generated_schema_invariants() {
    use crate::schema::{ComponentKind, PropValue};

    let generated = crate::schema_generated::BUILTINS_GENERATED;

    // Every entry must have a recognised kind.
    for def in generated {
        match def.kind {
            ComponentKind::Container(_)
            | ComponentKind::Leaf(_)
            | ComponentKind::ControlFlow(_)
            | ComponentKind::RuntimeLeaf => {}
        }
    }

    // No unclassified props: every prop must have a concrete
    // PropValue so the codegen knows how to turn XML literals
    // into Rust expressions.
    for def in generated {
        let ComponentKind::Leaf(leaf) = def.kind else {
            continue;
        };
        for prop in leaf.props {
            assert!(
                !matches!(prop.value, PropValue::Unknown),
                "generated schema contains PropValue::Unknown for `{}::{}`; \
                 add an override or extend the classifier",
                def.tag,
                prop.name
            );
        }
    }

    // A representative set of leaf components must be present.
    for tag in [
        "Button",
        "Checkbox",
        "Switch",
        "TextInput",
        "Avatar",
        "Badge",
        "Card",
        "Icon",
        "Label",
        "Tag",
        "Progress",
        "Slider",
        "Radio",
        "ToggleButton",
        "GridView",
    ] {
        assert!(
            generated.iter().any(|c| c.tag == tag),
            "tag {tag:?} should be present in BUILTINS_GENERATED"
        );
    }
}

#[test]
fn generated_schema_preserves_typed_enum_categories() {
    // Guard against regressions where `impl Into<IconSource>` or
    // similar gets misclassified as a plain String/Variant.
    use crate::schema::{ComponentKind, ExtraArgKind, PropValue};

    fn leaf_props(tag: &str) -> &'static [crate::schema::PropDef] {
        let generated = crate::schema_generated::BUILTINS_GENERATED;
        let def = generated
            .iter()
            .find(|c| c.tag == tag)
            .unwrap_or_else(|| panic!("missing {tag}"));
        match def.kind {
            ComponentKind::Leaf(leaf) => leaf.props,
            _ => panic!("{tag} is not a leaf"),
        }
    }

    fn leaf_extra_args(tag: &str) -> &'static [crate::schema::ExtraArg] {
        let generated = crate::schema_generated::BUILTINS_GENERATED;
        let def = generated
            .iter()
            .find(|c| c.tag == tag)
            .unwrap_or_else(|| panic!("missing {tag}"));
        match def.kind {
            ComponentKind::Leaf(leaf) => leaf.extra_args,
            _ => panic!("{tag} is not a leaf"),
        }
    }

    // Setters that take `impl Into<IconSource>` must stay IconSource.
    assert_eq!(
        leaf_props("EmptyState")
            .iter()
            .find(|p| p.name == "icon")
            .unwrap()
            .value,
        PropValue::IconSource,
        "EmptyState::icon must remain IconSource"
    );
    assert_eq!(
        leaf_props("IconButton")
            .iter()
            .find(|p| p.name == "icon")
            .unwrap()
            .value,
        PropValue::IconSource,
        "IconButton::icon must remain IconSource"
    );
    assert_eq!(
        leaf_props("ToggleButton")
            .iter()
            .find(|p| p.name == "icon")
            .unwrap()
            .value,
        PropValue::IconSource,
        "ToggleButton::icon must remain IconSource"
    );

    // Factory positional args must keep their dedicated enum categories.
    assert_eq!(
        leaf_extra_args("Icon")
            .iter()
            .find(|e| e.attr == "source")
            .unwrap()
            .kind,
        ExtraArgKind::IconSource,
        "Icon::source must remain IconSource"
    );
    assert_eq!(
        leaf_extra_args("Image")
            .iter()
            .find(|e| e.attr == "source")
            .unwrap()
            .kind,
        ExtraArgKind::ImageSource,
        "Image::source must remain ImageSource"
    );

    // KeybindingInputMode setter.
    assert_eq!(
        leaf_props("KeybindingInput")
            .iter()
            .find(|p| p.name == "mode")
            .unwrap()
            .value,
        PropValue::KeybindingInputMode,
        "KeybindingInput::mode must remain KeybindingInputMode"
    );

    // TreeNodeId props must be categorised consistently (both Custom).
    assert_eq!(
        leaf_props("Tree")
            .iter()
            .find(|p| p.name == "selected")
            .unwrap()
            .value,
        PropValue::Custom,
        "Tree::selected must remain Custom (TreeNodeId)"
    );
    assert_eq!(
        leaf_extra_args("TreeItem")
            .iter()
            .find(|e| e.attr == "node_id")
            .unwrap()
            .kind,
        ExtraArgKind::Custom,
        "TreeItem::node_id must remain Custom (TreeNodeId)"
    );
}

/// The generated file must not contain any "review needed"
/// comments. Those comments indicate props the generator
/// couldn't classify; the --check mode of gen-schema now
/// rejects them, and this test makes sure none slipped into
/// the committed file.
#[test]
fn generated_schema_has_no_review_needed_comments() {
    let generated = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/schema_generated.rs"
    ));
    assert!(
        !generated.contains("review needed"),
        "schema_generated.rs contains 'review needed' comments; \
         run `cargo run -p yororen_ui_xml --bin gen-schema` and resolve them"
    );
}

#[test]
fn overrides_provide_containers_and_control_flow() {
    // `BUILTINS_OVERRIDES` (sourced from `overrides.toml`)
    // covers the XML-only tags that the headless source
    // doesn't have entries for: `Div` (the only Container
    // left in the hand-written schema) and the control-flow
    // pseudo-tags (If, ElseIf, Else, For, Fragment, …).
    //
    // `Column` / `Row` / `Stack` / `Center` / `Expanded` /
    // `Wrap` are now auto-generated as Leaf entries from
    // `headless/layout/*.rs` (see `BUILTINS_GENERATED`).
    for tag in ["Div", "If", "ElseIf", "Else", "For", "Fragment"] {
        assert!(
            BUILTINS_OVERRIDES.iter().any(|c| c.tag == tag),
            "tag {tag:?} should be in BUILTINS_OVERRIDES"
        );
    }
    // Layout components should be auto-generated as Leaves.
    for tag in ["Column", "Row", "Stack", "Center", "Expanded", "Wrap"] {
        assert!(
            crate::schema_generated::BUILTINS_GENERATED
                .iter()
                .any(|c| c.tag == tag),
            "tag {tag:?} should be in BUILTINS_GENERATED"
        );
    }
}

#[test]
fn if_branch_supports_multiple_children() {
    let s = render(
        r#"<Column>
    <If condition={show}>
        <Label id="a" text="A" />
        <Label id="b" text="B" />
    </If>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    // The branch body should be wrapped in a div that has
    // both labels as children.
    assert!(
        normalised.contains("ifshow"),
        "condition must be emitted; got {normalised}"
    );
    assert!(
        normalised.matches("child").count() >= 2,
        "multiple children should be wired; got {normalised}"
    );
}

#[test]
fn for_loop_supports_multiple_children_per_row() {
    let s = render(
        r#"<Column>
    <For each={items} let:item>
        <Label id="a" text={item.a} />
        <Label id="b" text={item.b} />
    </For>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalised.contains("foritemin"),
        "loop must be emitted; got {normalised}"
    );
    assert!(
        normalised.matches("child").count() >= 2,
        "multiple children per row should be wired; got {normalised}"
    );
}

#[test]
fn state_supports_multiple_children() {
    let s = render(
        r#"<Column>
    <State name="count" default="0">
        <Label id="a" text="A" />
        <Label id="b" text="B" />
    </State>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalised.contains("new(|_|0i64)"),
        "state entity must be created; got {normalised}"
    );
    assert!(
        normalised.matches("child").count() >= 2,
        "multiple state children should be wired; got {normalised}"
    );
}

#[test]
fn match_case_supports_multiple_children() {
    let s = render(
        r#"<Column>
    <Match on={status}>
        <Case pattern={Status::Ok}>
            <Label id="a" text="A" />
            <Label id="b" text="B" />
        </Case>
    </Match>
</Column>"#,
    );
    let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalised.contains("match"),
        "match must be emitted; got {normalised}"
    );
    assert!(
        normalised.matches("child").count() >= 2,
        "multiple case children should be wired; got {normalised}"
    );
}

#[test]
fn color_prop_hex_rgb() {
    let s = render(r##"<Switch id="s" custom_tone="#ff00ff" />"##);
    assert!(
        s.contains("gpui :: rgb (16711935u32)"),
        "hex #ff00ff should become gpui::rgb(0xff00ff); got {s}"
    );
}

#[test]
fn color_prop_hex_rgba() {
    let s = render(r##"<Switch id="s" custom_tone="#ff00ff80" />"##);
    assert!(
        s.contains("gpui :: rgba (4278255488u32)"),
        "hex #ff00ff80 should become gpui::rgba; got {s}"
    );
}

#[test]
fn color_prop_brace_expression_passes_through() {
    let s = render(r#"<Switch id="s" custom_tone={gpui::hsla(0.5, 1.0, 0.5, 1.0)} />"#);
    assert!(
        s.contains("hsla (0.5 , 1.0 , 0.5 , 1.0)"),
        "brace colour expression should pass through; got {s}"
    );
}

#[test]
fn unknown_leaf_attribute_lists_accepted_attrs() {
    let err = codegen(
        r#"<Button id="b" href="https://example.com" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(
        err.message.contains("unknown attribute `href`"),
        "{}",
        err.message
    );
    assert!(
        err.message.contains("accepted:"),
        "error should list accepted attrs; got {}",
        err.message
    );
    assert!(
        err.message.contains("on_click"),
        "accepted list should mention on_click; got {}",
        err.message
    );
}

#[test]
fn unknown_leaf_attribute_suggests_did_you_mean() {
    let err = codegen(
        r#"<Button id="b" on_clik={|| {}} />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(
        err.message.contains("did you mean `on_click`"),
        "error should suggest on_click for on_clik; got {}",
        err.message
    );
}

#[test]
fn unknown_container_attribute_suggests_did_you_mean() {
    // `Div` is the remaining Container — use it to test
    // the did-you-mean suggestion for unknown container
    // attributes.
    let err = codegen(r#"<Div flx col />"#, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        err.message.contains("did you mean `flex`"),
        "error should suggest flex for flx; got {}",
        err.message
    );
}

#[test]
fn unknown_tag_without_id_suggests_did_you_mean() {
    let err = codegen(r#"<Buton />"#, Span::call_site(), None, None, &[]).unwrap_err();
    assert!(
        err.message.contains("did you mean `<Button>`"),
        "error should suggest Button for Buton; got {}",
        err.message
    );
}

// ===================== VirtualList =====================

#[test]
fn virtual_list_requires_id() {
    let err = codegen(
        r#"<VirtualList item_count={9}><Label id="x" text="hi" /></VirtualList>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("`id"), "{err}");
}

#[test]
fn virtual_list_requires_item_count_or_children() {
    // No item_count AND no children → error.
    let err = codegen(
        r#"<VirtualList id="x" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("item_count"), "{err}");
    assert!(err.message.contains("child row"), "{err}");
}

#[test]
fn virtual_list_requires_children() {
    let err = codegen(
        r#"<VirtualList id="x" item_count={9} />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("at least one child"), "{err}");
}

#[test]
fn virtual_list_rejects_row_with_children() {
    // `row=` provides an explicit row closure; children would be
    // silently ignored by the codegen. Reject instead.
    let err = codegen(
        r#"<VirtualList id="x" item_count={9} row={|_| {}}><Label id="y" text="hi" /></VirtualList>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("row"), "{err}");
    assert!(err.message.contains("children"), "{err}");
}

#[test]
fn virtual_list_emits_keyed_state_and_row_closure() {
    let s = render(
        r#"<VirtualList id="gallery-sections" item_count={9} let:index={i}>
    <Label id="x" text="hi" />
</VirtualList>"#,
    );
    // Auto-persisted controller via use_keyed_state.
    assert!(s.contains("use_keyed_state"), "{s}");
    // The heterogeneous factory.
    assert!(
        s.contains("headless :: virtual_list :: virtual_list"),
        "{s}"
    );
    // Default overdraw.
    assert!(s.contains("px (16.)"), "{s}");
    // Per-frame count sync via controller.reset.
    assert!(s.contains(". reset"), "{s}");
    // Row closure bound to the user's index name `i`.
    assert!(s.contains("move | i : usize"), "{s}");
    // Rendered into an AnyElement.
    assert!(s.contains("into_any_element"), "{s}");
}

#[test]
fn virtual_list_let_item_binds_alias() {
    // `let:item` introduces a `let item: usize = index;`
    // alias inside the row body (parity with `<For>`).
    let s = render(
        r#"<VirtualList id="vl" item_count={3} let:item let:index={i}>
    <Label id="x" text={item} />
</VirtualList>"#,
    );
    assert!(s.contains("let item : usize = i"), "{s}");
}

#[test]
fn virtual_list_default_index_binding() {
    // Without `let:index`, the row index binds to `index`.
    let s = render(
        r#"<VirtualList id="vl" item_count={3}>
    <Label id="x" text="hi" />
</VirtualList>"#,
    );
    assert!(s.contains("move | index : usize"), "{s}");
}

#[test]
fn virtual_list_custom_overdraw_and_alignment() {
    let s = render(
        r#"<VirtualList id="vl" item_count={3} overdraw={px(40.)} alignment="bottom">
    <Label id="x" text="hi" />
</VirtualList>"#,
    );
    assert!(s.contains("px (40.)"), "{s}");
    assert!(s.contains("ListAlignment :: Bottom"), "{s}");
    // Custom overdraw must NOT also emit the default.
    assert!(!s.contains("px (16.)"), "{s}");
}

#[test]
fn virtual_list_rejects_bad_alignment() {
    let err = codegen(
        r#"<VirtualList id="vl" item_count={3} alignment="sideways">
    <Label id="x" text="hi" />
</VirtualList>"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("\"top\" or \"bottom\""), "{err}");
}

#[test]
fn virtual_list_forwards_on_visible_range_change() {
    let s = render(
        r#"<VirtualList id="vl" item_count={3} on_visible_range_change={controller.on_range}>
    <Label id="x" text="hi" />
</VirtualList>"#,
    );
    assert!(s.contains("on_visible_range_change"), "{s}");
    assert!(s.contains("__auto_clone . on_range"), "{s}");
}

// ===================== UniformVirtualList =====================

#[test]
fn uniform_virtual_list_emits_uniform_factory() {
    let s = render(
        r#"<UniformVirtualList id="uvl" item_count={1000} let:index={i}>
    <Label id="y" text="u" />
</UniformVirtualList>"#,
    );
    // Uniform factory + controller.
    assert!(
        s.contains("headless :: virtual_list :: uniform_virtual_list"),
        "{s}"
    );
    assert!(s.contains("UniformVirtualListController :: new"), "{s}");
    // No reset (uniform lists take count at the call site).
    assert!(!s.contains(". reset"), "{s}");
    // No on_visible_range_change support on uniform lists.
    assert!(!s.contains("on_visible_range_change"), "{s}");
    // Item count passed positionally at the call site.
    assert!(s.contains("1000 as usize"), "{s}");
}

#[test]
fn uniform_virtual_list_requires_item_count_or_children() {
    // Neither item_count nor children → error.
    let err = codegen(
        r#"<UniformVirtualList id="uvl" />"#,
        Span::call_site(),
        None,
        None,
        &[],
    )
    .unwrap_err();
    assert!(err.message.contains("item_count"), "{err}");
    assert!(err.message.contains("child row"), "{err}");
}

// ============= children-as-rows mode =============

#[test]
fn virtual_list_children_as_rows_counts_children() {
    // No `item_count`: each direct child is a row. The
    // emitted count literal equals the number of children.
    let s = render(
        r#"<VirtualList id="vl">
    <Label id="a" text="1" />
    <Label id="b" text="2" />
    <Label id="c" text="3" />
</VirtualList>"#,
    );
    // item_count = 3 children.
    assert!(s.contains("3usize"), "{s}");
    // Row closure dispatches by index via a match.
    assert!(s.contains("match"), "{s}");
    assert!(s.contains("0usize =>"), "{s}");
    assert!(s.contains("2usize =>"), "{s}");
    // Wildcard arm returns an empty div.
    assert!(s.contains("_ =>"), "{s}");
}

#[test]
fn virtual_list_children_as_rows_supports_single_child() {
    let s = render(
        r#"<VirtualList id="vl">
    <Label id="a" text="1" />
</VirtualList>"#,
    );
    assert!(s.contains("1usize"), "{s}");
    assert!(s.contains("0usize =>"), "{s}");
}

#[test]
fn virtual_list_style_passthrough_applies_to_rendered_element() {
    // `size_full` and `h` are not VL attrs → they become
    // style calls on the rendered element.
    let s = render(
        r#"<VirtualList id="vl" item_count={3} size_full h={px(200.)}>
    <Label id="a" text="1" />
</VirtualList>"#,
    );
    assert!(s.contains("Styled :: size_full"), "{s}");
    assert!(s.contains("Styled :: h"), "{s}");
    assert!(s.contains("px (200.)"), "{s}");
}

#[test]
fn virtual_list_children_as_rows_style_passthrough() {
    // Style passthrough works in children-as-rows mode too.
    let s = render(
        r#"<VirtualList id="vl" size_full>
    <Label id="a" text="1" />
</VirtualList>"#,
    );
    assert!(s.contains("Styled :: size_full"), "{s}");
}
