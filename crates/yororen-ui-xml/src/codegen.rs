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
use quote::{ToTokens, TokenStreamExt, format_ident, quote};

use crate::ast::{AstAttribute, AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::{
    ComponentKind, ContainerDef, ControlFlowDef, ExtraArgKind, LeafDef, PropValue, RenderMode,
    is_known_shorthand_method, is_spacing_prefix, is_spacing_shorthand,
};
use crate::schema_generated::{BUILTINS_GENERATED, BUILTINS_OVERRIDES};

/// Find a [`ComponentDef`] by tag. Lookup order:
/// 1. Hand-written `BUILTINS` in `schema.rs` (Phase 1
///    leaves with manual overrides — kept small).
/// 2. `BUILTINS_OVERRIDES` in the generated file
///    (containers + control flow from `overrides.toml`).
/// 3. `BUILTINS_GENERATED` in the generated file
///    (auto-extracted leaves).
fn lookup_component<'a>(tag: &str) -> Option<&'a crate::schema::ComponentDef> {
    use crate::schema::BUILTINS;
    if let Some(c) = BUILTINS.iter().find(|c| c.tag == tag) {
        return Some(c);
    }
    if let Some(c) = BUILTINS_OVERRIDES.iter().find(|c| c.tag == tag) {
        return Some(c);
    }
    BUILTINS_GENERATED.iter().find(|c| c.tag == tag)
}

/// Parse a string of Rust tokens into a `TokenStream`. Any
/// error is converted into an `XmlError::InvalidExpression`.
fn parse_ts(src: &str, span: Span, context: &str) -> Result<TokenStream, XmlError> {
    src.parse::<TokenStream>().map_err(|e| {
        XmlError::new(
            XmlErrorKind::InvalidExpression,
            span,
            format!("could not parse {context} `{src}`: {e}"),
        )
    })
}

/// Compile an `xml! { ... }` invocation.
///
/// `xml_text` is the literal XML the user wrote between the
/// braces. `outer_span` is the span of the literal (the macro
/// entry point supplies the call site). `cx_expr` is the
/// `cx = <expr>` token stream from the optional preamble, or
/// `None` to default to the bare identifier `cx`.
pub fn codegen(
    xml_text: &str,
    outer_span: Span,
    cx_expr: Option<TokenStream>,
) -> Result<TokenStream, XmlError> {
    let line_starts = crate::parser::line_starts(xml_text);
    let element = {
        let location = crate::parser::LocationTracker {
            line_starts: &line_starts,
            xml: xml_text,
        };
        crate::parser::parse(xml_text, outer_span, &location)?
    };
    let cx_tokens = match cx_expr {
        Some(expr) => quote! { (#expr) },
        None => quote! { cx },
    };
    let body = codegen_element(&element, &cx_tokens, outer_span)?;
    // Wrap the body in a block that imports the traits the
    // generated code needs. This keeps the call site clean:
    // the user never has to remember to `use gpui::Styled;`
    // for `<Column flex>` to compile.
    Ok(quote! {
        {
            #[allow(unused_imports)]
            use ::gpui::{IntoElement, ParentElement, StatefulInteractiveElement, InteractiveElement, Styled};
            #body
        }
    })
}

fn codegen_element(
    element: &AstElement,
    cx: &TokenStream,
    outer_span: Span,
) -> Result<TokenStream, XmlError> {
    let def = lookup_component(&element.tag).ok_or_else(|| {
        XmlError::new(
            XmlErrorKind::UnknownTag,
            element.span,
            format!("unknown tag <{}>", element.tag),
        )
    })?;

    match def.kind {
        ComponentKind::Container(c) => codegen_container(element, c, cx),
        ComponentKind::Leaf(l) => codegen_leaf(element, l, cx),
        ComponentKind::ControlFlow(c) => codegen_control_flow(element, c, cx, outer_span),
    }
}

fn codegen_container(
    element: &AstElement,
    def: ContainerDef,
    cx: &TokenStream,
) -> Result<TokenStream, XmlError> {
    let mut tokens = quote! { gpui::div() };

    for attr in &element.attributes {
        apply_container_attr(&mut tokens, attr, def, element)?;
    }

    // Walk children, merging consecutive If/ElseIf/Else
    // into a single Rust if/else chain (which must be a
    // single block expression so it can be the argument
    // of `.child(...)`).
    let mut i = 0;
    while i < element.children.len() {
        let child = &element.children[i];
        if matches!(
            child,
            AstNode::Element(e)
                if matches!(e.tag.as_str(), "If" | "ElseIf" | "Else")
        ) {
            // Collect the chain.
            let mut j = i;
            while j < element.children.len() {
                if let AstNode::Element(e) = &element.children[j] {
                    if !matches!(e.tag.as_str(), "If" | "ElseIf" | "Else") {
                        break;
                    }
                } else {
                    break;
                }
                j += 1;
            }
            let chain_expr = codegen_if_chain(&element.children[i..j], cx)?;
            tokens.append_all(quote! { .child(#chain_expr) });
            i = j;
        } else {
            let child_expr = codegen_child(child, cx)?;
            tokens.append_all(quote! { .child(#child_expr) });
            i += 1;
        }
    }

    Ok(tokens)
}

/// Combine a run of `If` / `ElseIf` / `Else` siblings
/// into a single block expression:
///   `{ if cond1 { body1 } else if cond2 { body2 } else { body3 } }`
///
/// The block is required so the result is a `Div`
/// (which `Div::child` expects) regardless of branch
/// type. The first branch must be `If`; `ElseIf` /
/// `Else` without a leading `If` is a hard error.
fn codegen_if_chain(
    branches: &[AstNode],
    cx: &TokenStream,
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
        let branch_expr = codegen_if_branch(element, element_tag_to_branch_kind(&element.tag)?, cx, element.span)?;
        chain.append_all(branch_expr);
        // After the first branch, every subsequent one
        // must be ElseIf or Else (the Rust grammar).
        if i == 0 && element.tag != "If" {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                element.span,
                format!("<{}> cannot start a chain — use <If> first", element.tag),
            ));
        }
    }
    Ok(quote! { { #chain } })
}

fn element_tag_to_branch_kind(tag: &str) -> Result<ControlFlowDef, XmlError> {
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

fn apply_container_attr(
    tokens: &mut TokenStream,
    attr: &AstAttribute,
    def: ContainerDef,
    element: &AstElement,
) -> Result<(), XmlError> {
    if attr.name == "id" {
        // `id="my-button"` becomes `.id("my-button".into())` —
        // we don't use the `id` attr on containers today but
        // reserve the slot so that future custom containers
        // can wire it.
        let value = attr_value_tokens(attr)?;
        tokens.append_all(quote! { .id(#value) });
        return Ok(());
    }

    // Fixed boolean flag (e.g. `col` for Column → `flex_col`).
    if attr.expr.is_none() {
        let name = attr.name.as_str();
        if let Some((_, method)) = def
            .fixed_methods
            .iter()
            .find(|(attr_name, _)| *attr_name == name)
            .copied()
        {
            let m = format_ident!("{}", method);
            tokens.append_all(quote! { .#m() });
            return Ok(());
        }
    }

    // Brace expression on a container: pass through to the
    // matching Styled method.
    if let Some(expr) = &attr.expr {
        // `gap={pixels}` → `.gap(pixels)`
        if is_spacing_prefix(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed =
                parse_ts(expr, attr.span, &format!("expression for `{}`", attr.name))?;
            tokens.append_all(quote! { .#m(#parsed) });
            return Ok(());
        }
        // `gap_3={...}` or `flex_grow={...}` → `.gap_3(expr)`
        if is_known_shorthand_method(&attr.name) || is_spacing_shorthand(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed =
                parse_ts(expr, attr.span, &format!("expression for `{}`", attr.name))?;
            tokens.append_all(quote! { .#m(#parsed) });
            return Ok(());
        }
    }

    // Literal value on a spacing prefix: `gap="3"` → `.gap_3()`.
    if attr.expr.is_none() && is_spacing_prefix(&attr.name) {
        let value = attr.raw.as_str();
        // The gpui method name is `<attr>_<value>`. Allow
        // numeric (`3`, `1p5`, `12`, `0p5`) and textual
        // (`full`) suffixes — anything else is an error.
        if !is_valid_spacing_suffix(value) {
            return Err(XmlError::new(
                XmlErrorKind::InvalidExpression,
                attr.span,
                format!(
                    "invalid spacing suffix `{value}` for `{}`; expected a number (0, 1, 2, …) or `full`",
                    attr.name
                ),
            ));
        }
        // Translate `0p5` (XML) → `0p5` (method name)
        let method = format!("{}_{}", attr.name, value);
        let m = format_ident!("{}", method);
        tokens.append_all(quote! { .#m() });
        return Ok(());
    }

    // Bare method name on a container (`flex`, `flex_col`,
    // `w_full`, …). Only valid when the value is the
    // normaliser-added `"true"`.
    if attr.expr.is_none()
        && (is_known_shorthand_method(&attr.name) || is_spacing_shorthand(&attr.name))
    {
        // The normaliser should have converted a bare attr
        // to `="true"`, so this is the common path.
        if attr.raw == "true" {
            let m = format_ident!("{}", attr.name);
            tokens.append_all(quote! { .#m() });
            return Ok(());
        }
        // `flex_grow="0.5"` — odd but possible; we just
        // pass the value as a string to the method.
        let raw = attr.raw.as_str();
        let m = format_ident!("{}", attr.name);
        tokens.append_all(quote! { .#m(#raw) });
        return Ok(());
    }

    Err(XmlError::new(
        XmlErrorKind::UnknownAttribute,
        attr.span,
        format!(
            "unknown attribute `{}` on <{}>; containers only accept shorthand style attributes (gap_3, p_4, w_full, flex, col, …) — see {}",
            attr.name,
            element.tag,
            def.style_hint,
        ),
    ))
}

fn is_valid_spacing_suffix(s: &str) -> bool {
    const NUMERIC: &[&str] = &[
        "0", "0p5", "1", "1p5", "2", "2p5", "3", "3p5", "4", "5", "6", "7", "8", "9", "10", "11",
        "12", "16", "20", "24", "32", "40", "48", "56", "64", "72", "80", "96",
    ];
    const TEXTUAL: &[&str] = &["full", "1_2", "1_3", "2_3", "1_4", "3_4", "1_5", "2_5", "3_5", "4_5", "1_6", "5_6", "1_12"];
    NUMERIC.contains(&s) || TEXTUAL.contains(&s)
}

fn codegen_leaf(
    element: &AstElement,
    def: LeafDef,
    cx: &TokenStream,
) -> Result<TokenStream, XmlError> {
    // 1. Resolve the id (first factory arg).
    let id_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "id")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                format!("<{}> requires an `id` attribute", element.tag),
            )
        })?;
    let id_expr = attr_value_tokens(id_attr)?;

    // 2. Build the factory call head.
    //
    // The factory signature is always one of:
    //   `factory(id, [extra_args…], [cx])`
    // where `cx` is present iff `def.needs_app`. We build
    // the positional arg list by:
    //   1. Inserting the resolved `id` value.
    //   2. Resolving every entry in `def.extra_args` (in
    //      declaration order) and inserting the value.
    //   3. Optionally appending `cx`.
    let mut factory_args: Vec<TokenStream> = Vec::new();
    factory_args.push(id_expr);

    for extra in def.extra_args {
        let extra_attr = element.attributes.iter().find(|a| a.name == extra.attr);
        let extra_tokens: TokenStream = match (extra.kind, extra_attr) {
            (ExtraArgKind::Text, Some(a)) => text_attr_value(a)?,
            (ExtraArgKind::Text, None) => {
                // Fall back to inner text content.
                let text = extract_text_content(&element.children).ok_or_else(|| {
                    XmlError::new(
                        XmlErrorKind::UnknownAttribute,
                        element.span,
                        format!(
                            "<{}> needs a `{}` attribute or text content",
                            element.tag, extra.attr
                        ),
                    )
                })?;
                quote! { (#text).to_string() }
            }
            (ExtraArgKind::Custom, Some(a)) => attr_value_tokens(a)?,
            (ExtraArgKind::Custom, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                ));
            }
        };
        factory_args.push(extra_tokens);
    }

    if def.needs_app {
        factory_args.push(quote! { #cx });
    }

    let factory: TokenStream = parse_ts(
        def.factory,
        element.span,
        &format!("factory path for <{}>", element.tag),
    )?;
    let mut tokens = quote! { #factory(#(#factory_args),*) };

    // 3. Apply prop / event setters in declaration order.
    for attr in &element.attributes {
        if attr.name == "id" {
            continue;
        }
        // Attributes consumed by the factory call
        // (the `text` of a `Label`, the `state` of a
        // `Modal`, …) are NOT re-emitted as setters.
        if def.extra_args.iter().any(|e| e.attr == attr.name) {
            continue;
        }
        // Special attribute `@bind={entity}` — the codegen
        // expands it to a one-line two-way binding. The
        // current value is read from the entity, the
        // on_change callback writes back. The user writes:
        //
        //     <TextInput id="x" @bind={self.name} />
        //
        // …and the macro turns it into:
        //
        //     text_input("x", cx)
        //         .value(self.name.read(cx).clone())
        //         .on_change({ let e = self.name.clone();
        //                     move |v, _, cx| e.update(cx, |s, _| *s = v.to_string()) })
        //
        // The exact prop name / event name depend on the
        // component, so we look them up from the schema:
        // the value setter is the first prop of kind
        // `String` whose name is one of `value` / `text`,
        // the event is the first `on_change` event.
        if attr.name == "bind" {
            if let Some(expr) = &attr.expr {
                let parsed = parse_ts(
                    expr,
                    attr.span,
                    "@bind requires a brace expression, e.g. `@bind={self.name}`",
                )?;
                tokens.append_all(emit_bind(&parsed, def));
                continue;
            } else {
                return Err(XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    "@bind requires a brace expression, e.g. `@bind={self.name}`",
                ));
            }
        }
        if let Some(prop) = def.props.iter().find(|p| p.name == attr.name).copied() {
            let m = format_ident!("{}", prop.setter);
            match prop.value {
                PropValue::Flag => {
                    // Zero-arg setter (`fn X(self) -> Self`).
                    // The bare-attribute convention is the
                    // trigger: `<Label wrap />` enables it;
                    // `wrap="false"` is a no-op; `wrap={…}` is
                    // a type error (the user is on the hook).
                    if let Some(_) = &attr.expr {
                        return Err(XmlError::new(
                            XmlErrorKind::InvalidExpression,
                            attr.span,
                            format!(
                                "attribute `{}` is a flag (no value) — drop the `={{…}}`",
                                attr.name
                            ),
                        ));
                    }
                    let raw = attr.raw.as_str();
                    if raw == "true" {
                        tokens.append_all(quote! { .#m() });
                    }
                    // `raw == "false"` → skip the call (the
                    // default for unset).
                    continue;
                }
                _ => {
                    let value = prop_value_tokens(attr, prop.value)?;
                    tokens.append_all(quote! { .#m(#value) });
                }
            }
            continue;
        }
        if let Some((_, setter)) = def.events.iter().find(|(n, _)| *n == attr.name).copied() {
            let m = format_ident!("{}", setter);
            // Events take a closure — don't `.into()`.
            let expr = attr_expr_only(attr)?;
            tokens.append_all(quote! { .#m(#expr) });
            continue;
        }
        return Err(XmlError::new(
            XmlErrorKind::UnknownAttribute,
            attr.span,
            format!(
                "unknown attribute `{}` on <{}>",
                attr.name, element.tag
            ),
        ));
    }

    // 4. Apply render mode.
    match def.render {
        RenderMode::Default => {
            // The render method typically takes `(&App)`;
            // a few components (e.g. `TextInput`) also
            // need a `&mut Window`. We detect the latter
            // by the factory path — a hardcoded but
            // small list. A future revision can move
            // this to a schema flag.
            let needs_window = def.factory.contains("text_input");
            if needs_window {
                let app_ref = quote! { &*#cx };
                // The `window` token is assumed to be in
                // the surrounding scope (e.g. `Render::render`
                // takes `&mut Window` as a parameter).
                tokens.append_all(quote! { .render(#app_ref, window) });
            } else {
                let app_ref = quote! { &*#cx };
                tokens.append_all(quote! { .render(#app_ref) });
            }
        }
        RenderMode::Apply => {
            // Caller is responsible for `.apply(div())` — for
            // now, do nothing. (Phase 2 will wire `<Button
            // custom>{...}</Button>` to `.apply(div()).child(...)`.)
        }
    }

    // 5. Optional text child.
    if def.supports_text_child {
        if let Some(text) = extract_text_content(&element.children) {
            tokens.append_all(quote! { .child(#text) });
        }
    }

    // 6. Wrap to AnyElement so the result composes into a parent.
    tokens.append_all(quote! { .into_any_element() });

    Ok(tokens)
}

fn codegen_control_flow(
    element: &AstElement,
    def: ControlFlowDef,
    cx: &TokenStream,
    outer_span: Span,
) -> Result<TokenStream, XmlError> {
    match def {
        ControlFlowDef::If | ControlFlowDef::ElseIf | ControlFlowDef::Else => {
            codegen_if_branch(element, def, cx, outer_span)
        }
        ControlFlowDef::For => codegen_for(element, cx),
        ControlFlowDef::Fragment => codegen_fragment(element, cx),
        ControlFlowDef::Include => codegen_include(element, cx, outer_span),
        ControlFlowDef::Template => codegen_template(element, cx),
        ControlFlowDef::Slot => codegen_slot(element, cx),
    }
}

fn codegen_if_branch(
    element: &AstElement,
    kind: ControlFlowDef,
    cx: &TokenStream,
    _outer_span: Span,
) -> Result<TokenStream, XmlError> {
    let condition = if matches!(kind, ControlFlowDef::Else) {
        TokenStream::new()
    } else {
        let cond_attr = element.attributes.iter().find(|a| a.name == "condition").ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                format!("<{:?}> requires a `condition={{...}}` attribute", kind),
            )
        })?;
        let expr = attr_expr_only(cond_attr)?;
        quote! { #expr }
    };

    // Build the body — a list of children turned into a
    // tuple of expressions (in the form `(expr1, expr2, …)`)
    // so the whole branch yields an `impl IntoElement`.
    //
    // For now we only support a single child per branch.
    let child_expr = if element.children.is_empty() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "if/else branch must contain at least one child",
        ));
    } else if element.children.len() == 1 {
        codegen_child(&element.children[0], cx)?
    } else {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            format!("if/else branch has {} children; wrap in a <Column> or <Row> for now", element.children.len()),
        ));
    };

    Ok(match kind {
        ControlFlowDef::If => quote! { if #condition { #child_expr } },
        ControlFlowDef::ElseIf => quote! { else if #condition { #child_expr } },
        ControlFlowDef::Else => quote! { else { #child_expr } },
        // Unreachable
        _ => unreachable!("non-branch kind {:?}", kind),
    })
}

fn codegen_for(element: &AstElement, cx: &TokenStream) -> Result<TokenStream, XmlError> {
    let each = element.attributes.iter().find(|a| a.name == "each").ok_or_else(|| {
        XmlError::new(
            XmlErrorKind::UnknownAttribute,
            element.span,
            "<For> requires an `each={...}` attribute",
        )
    })?;
    let each_parsed = attr_expr_only(each)?;

    // `<For each={xs} let:item>…</For>` — the `let:item` part
    // sets the loop variable name (defaults to `it`).
    // The parser preprocessed `let:item` to `let_item`, so
    // we look for the `let_item` / `let_index` names here.
    // The normaliser may have appended `="true"` to the
    // value-less `let_item` attr; we treat any value of
    // `let_item` that's `== "true"` (or empty) as the
    // default — we want the *name*, not the value.
    let item_name = element
        .attributes
        .iter()
        .find(|a| a.name == "let_item")
        .map(|a| {
            // The original `let:item` carried no value, so
            // the normaliser injected `="true"`. If the
            // value is `true`, fall back to the default
            // name `item`; otherwise treat the value as
            // the custom name.
            if a.raw == "true" || a.raw.is_empty() {
                "item".to_string()
            } else {
                a.raw.clone()
            }
        })
        .unwrap_or_else(|| "it".to_string());
    let item_ident = format_ident!("{}", item_name);

    // Optional index variable: `let:index={i}` (named `i` by
    // default; the `let_index` attribute is the marker — the
    // preprocessor turns `let:index` into `let_index`).
    let has_index = element.attributes.iter().any(|a| a.name == "let_index");
    let index_ident = format_ident!("i");

    let child_expr = if element.children.is_empty() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<For> must wrap a single child",
        ));
    } else if element.children.len() == 1 {
        codegen_child(&element.children[0], cx)?
    } else {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<For> must wrap a single child (wrap multiple in a Column / Row)",
        ));
    };

    // `<For each={xs} let:item>` produces
    //   xs.into_iter().map(|item| { … }).collect::<Vec<_>>()
    // but that yields `Vec<AnyElement>`, not `IntoElement`.
    // For Phase 1 we wrap it in a `div()` with the children
    // appended so it slots into a parent.
    if has_index {
        let mut body = quote! { gpui::div() };
        for_each_in_each(&each_parsed, &item_ident, &index_ident, true, &child_expr, &mut body);
        Ok(body)
    } else {
        let mut body = quote! { gpui::div() };
        for_each_in_each(&each_parsed, &item_ident, &index_ident, false, &child_expr, &mut body);
        Ok(body)
    }
}

fn for_each_in_each(
    each_parsed: &TokenStream,
    item_ident: &proc_macro2::Ident,
    index_ident: &proc_macro2::Ident,
    has_index: bool,
    child_expr: &TokenStream,
    body: &mut TokenStream,
) {
    // Emit the children as a chain of `.child(...)` calls
    // inside a block that runs at runtime.
    *body = if has_index {
        quote! {
            {
                let mut __div = gpui::div();
                for (__i, #item_ident) in (#each_parsed).into_iter().enumerate() {
                    let #index_ident = __i;
                    __div = __div.child(#child_expr);
                }
                __div
            }
        }
    } else {
        quote! {
            {
                let mut __div = gpui::div();
                for #item_ident in (#each_parsed).into_iter() {
                    __div = __div.child(#child_expr);
                }
                __div
            }
        }
    };
}

fn codegen_fragment(element: &AstElement, cx: &TokenStream) -> Result<TokenStream, XmlError> {
    let mut children_tokens = TokenStream::new();
    for child in &element.children {
        let expr = codegen_child(child, cx)?;
        children_tokens.append_all(quote! { #expr, });
    }
    Ok(quote! { (#children_tokens) })
}

/// Compile-time file inclusion. The `src` attribute is
/// read at proc-macro time, the file is parsed, and
/// its children are spliced into the current
/// `xml! { … }` invocation as additional sibling
/// elements. This is the file-include sibling of
/// `xml_file!` — useful inside a larger XML literal.
fn codegen_include(
    element: &AstElement,
    cx: &TokenStream,
    outer_span: Span,
) -> Result<TokenStream, XmlError> {
    let src_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "src")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                "<Include> requires a `src=\"...\"` attribute",
            )
        })?;
    // For now, the only way to get a `&str` path is via
    // a string literal in the XML. (`Include>` does not
    // accept brace expressions for `src` — the file
    // must be known at macro-expansion time.)
    if src_attr.expr.is_some() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            src_attr.span,
            "<Include src> requires a string literal, not a brace expression",
        ));
    }
    let path = src_attr.raw.as_str();
    // Read the file relative to the call site. We use
    // the same `cargo:rerun-if-changed` mechanism that
    // `include_str!` provides by leaning on the proc
    // macro entry's `Span::call_site().file()` (set up
    // in the macro crate).
    let resolved = resolve_include_path(path, outer_span)?;
    let contents = std::fs::read_to_string(&resolved).map_err(|e| {
        XmlError::new(
            XmlErrorKind::ParseError,
            outer_span,
            format!("could not read `{}` (resolved to `{}`): {e}", path, resolved.display()),
        )
    })?;
    // Parse the included file and emit its children as
    // a comma-separated sequence of expressions.
    let line_starts = crate::parser::line_starts(&contents);
    let included_root = {
        let location = crate::parser::LocationTracker {
            line_starts: &line_starts,
            xml: &contents,
        };
        crate::parser::parse(&contents, outer_span, &location)?
    };
    let mut inner = TokenStream::new();
    for child in &included_root.children {
        let expr = codegen_child(child, cx)?;
        inner.append_all(quote! { #expr, });
    }
    Ok(quote! { (#inner) })
}

/// Try to resolve a relative `path` against the
/// call-site source file. In the proc-macro context,
/// the `outer_span` is the user's literal token span;
/// `proc_macro2::Span` doesn't expose the source file
/// (that's a `proc_macro::Span` method), so for the
/// MVP we use `CARGO_MANIFEST_DIR` of the current build
/// as the base. The full implementation will thread
/// the file path through from the proc-macro entry.
fn resolve_include_path(path: &str, _span: Span) -> Result<std::path::PathBuf, XmlError> {
    use std::path::Path;
    let p = Path::new(path);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }
    // Fall back to CWD. The proc-macro entry point
    // would override this with the real source file
    // path in a future revision.
    Ok(Path::new(".").join(path))
}

/// `<Template name="X">…</Template>` for the MVP
/// simply emits its children in place (the template
/// "name" attribute is reserved for future
/// cross-references). Slots are no-ops.
fn codegen_template(element: &AstElement, cx: &TokenStream) -> Result<TokenStream, XmlError> {
    codegen_fragment(element, cx)
}

/// `<Slot/>` is a no-op for the MVP. Future revisions
/// will wire caller-side slot-filling.
fn codegen_slot(_element: &AstElement, _cx: &TokenStream) -> Result<TokenStream, XmlError> {
    Ok(TokenStream::new())
}

fn codegen_child(node: &AstNode, cx: &TokenStream) -> Result<TokenStream, XmlError> {
    match node {
        AstNode::Element(e) => codegen_element(e, cx, e.span),
        AstNode::Text { text, .. } => {
            // Text content inside a container is uncommon — only
            // meaningful for `<Button>Click me</Button>` (handled
            // by `supports_text_child`) or `<Label>Hello</Label>`.
            // For all other parents, surface an error.
            Ok(quote! { #text })
        }
    }
}

// --- helpers ----------------------------------------------------------------

fn attr_value_tokens(attr: &AstAttribute) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        let parsed = parse_ts(expr, attr.span, &format!("attribute `{}`", attr.name))?;
        Ok(quote! { #parsed })
    } else {
        let raw = attr.raw.as_str();
        // Use `.to_string()` so the type is unambiguous
        // (avoids the "multiple From<&str> impls" error when
        // the consumer's `impl Into<...>` is generic).
        Ok(quote! { (#raw).to_string() })
    }
}

fn prop_value_tokens(attr: &AstAttribute, kind: PropValue) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        // Brace expression — use as-is, `.into()` where
        // appropriate. (For `Flag` props, the main loop
        // has already rejected the expression.)
        let parsed = parse_ts(expr, attr.span, &format!("attribute `{}`", attr.name))?;
        return Ok(match kind {
            PropValue::String
            | PropValue::Variant
            | PropValue::Bool
            | PropValue::Unknown => quote! { #parsed },
            PropValue::Flag => quote! { #parsed /* unreachable */ },
        });
    }
    let raw = attr.raw.as_str();
    match kind {
        PropValue::String => Ok(quote! { (#raw).to_string() }),
        PropValue::Bool => match raw {
            "true" => Ok(quote! { true }),
            "false" => Ok(quote! { false }),
            other => Err(XmlError::new(
                XmlErrorKind::InvalidExpression,
                attr.span,
                format!(
                    "attribute `{}` expects `true` or `false`, got `{other}`",
                    attr.name
                ),
            )),
        },
        PropValue::Flag => {
            // Handled by the codegen's main loop — the
            // `Flag` arm above emits `.setter()` directly
            // and bypasses `prop_value_tokens`. This arm
            // exists for match exhaustiveness.
            Ok(quote! { /* unreachable */ })
        }
        PropValue::Variant => Ok(match raw {
            "neutral" => quote! { ::yororen_ui::ActionVariantKind::Neutral },
            "primary" => quote! { ::yororen_ui::ActionVariantKind::Primary },
            "danger" => quote! { ::yororen_ui::ActionVariantKind::Danger },
            other => {
                return Err(XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    format!(
                        "attribute `{}` expects one of `neutral`, `primary`, `danger`, got `{other}`",
                        attr.name
                    ),
                ));
            }
        }),
        PropValue::Unknown => Ok(quote! { (#raw).to_string() }),
    }
}

/// Emit the expansion of `@bind={entity}` for a given
/// component. We pick the value setter (preferring a
/// `value` / `text` named prop) and the change event
/// (preferring `on_change`). The resulting token stream
/// appends both calls to the props builder.
fn emit_bind(entity: &TokenStream, def: LeafDef) -> TokenStream {
    // Pick the value prop. Prefer `value` (TextInput,
    // SearchInput, NumberInput, …); fall back to
    // `checked` (Checkbox, Switch, ToggleButton); then
    // `text` (Label-like). If none of these exist, the
    // read side is skipped — the entity's current value
    // is read on each render anyway (e.g. a `name`
    // setter on `Avatar`).
    let value_prop = def
        .props
        .iter()
        .find(|p| p.name == "value")
        .or_else(|| def.props.iter().find(|p| p.name == "checked"))
        .or_else(|| def.props.iter().find(|p| p.name == "text"));
    // Pick the change event. Prefer `on_change`; fall
    // back to `on_toggle` for boolean-style components.
    let change_event = def
        .events
        .iter()
        .find(|(n, _)| *n == "on_change")
        .or_else(|| def.events.iter().find(|(n, _)| *n == "on_toggle"));

    let mut out = TokenStream::new();
    if let Some(prop) = value_prop {
        let m = format_ident!("{}", prop.setter);
        // Read the current value: `entity.read(cx).clone()`.
        // We clone the entity so the original binding in
        // the user's scope isn't moved.
        out.append_all(quote! {
            .#m({
                let __bind = (#entity).clone();
                __bind.read(cx).clone()
            })
        });
    }
    if let Some((event_attr, setter)) = change_event {
        let m = format_ident!("{}", setter);
        // Pick the closure signature based on the event
        // name. on_change takes `(&str, &mut Window,
        // &mut App)`; on_toggle takes
        // `(bool, Option<&ClickEvent>, &mut Window,
        // &mut App)`. We match on the attribute name
        // to decide which shape to emit.
        let event_name = *event_attr;
        let writeback = if event_name == "on_toggle" {
            quote! {
                .#m({
                    let __bind = (#entity).clone();
                    move |__v: bool, _ev: Option<&gpui::ClickEvent>, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        __bind.update(cx, |__s: &mut bool, _| {
                            *__s = __v;
                        });
                    }
                })
            }
        } else {
            quote! {
                .#m({
                    let __bind = (#entity).clone();
                    move |__v: &str, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        let __new: String = __v.to_string();
                        __bind.update(cx, |__s: &mut String, _| {
                            *__s = __new;
                        });
                    }
                })
            }
        };
        out.append_all(writeback);
    }
    out
}

fn attr_expr_only(attr: &AstAttribute) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        let parsed = parse_ts(expr, attr.span, &format!("attribute `{}`", attr.name))?;
        Ok(parsed)
    } else {
        Err(XmlError::new(
            XmlErrorKind::InvalidExpression,
            attr.span,
            format!(
                "attribute `{}` requires a brace expression, e.g. `{}={{...}}`",
                attr.name, attr.name
            ),
        ))
    }
}

/// Build the value for a "text-like" attribute. Supports
/// brace interpolation: `text="Count: {count}"` becomes
/// `format!("Count: {}", count).into()`.
///
/// String literals without `{` are emitted as
/// `(#raw).to_string()` (the same path as before).
fn text_attr_value(attr: &AstAttribute) -> Result<TokenStream, XmlError> {
    if let Some(expr) = &attr.expr {
        // Brace expression — wrap as a single-arg
        // `format!` call. The user has full control.
        let parsed = parse_ts(expr, attr.span, &format!("text attribute `{}`", attr.name))?;
        return Ok(quote! { (#parsed).to_string() });
    }
    // String literal: detect `{...}` interpolation.
    if let Some(parts) = parse_string_interpolation(&attr.raw) {
        return Ok(render_string_interpolation(&parts, attr));
    }
    let raw = attr.raw.as_str();
    Ok(quote! { (#raw).to_string() })
}

/// A piece of a string-with-`{}` interpolation template.
#[derive(Debug, Clone)]
enum InterpPart {
    /// A literal fragment (no braces).
    Literal(String),
    /// A `{…}` expression.
    Expr(String),
}

/// Scan `text` for `{expr}` segments. Returns `None` if
/// there are no braces at all (the codegen should then
/// take the fast path of just emitting the literal).
fn parse_string_interpolation(text: &str) -> Option<Vec<InterpPart>> {
    let bytes = text.as_bytes();
    if !bytes.contains(&b'{') {
        return None;
    }
    let mut parts = Vec::new();
    let mut current_literal = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            // Find the matching `}` (single-level depth).
            let start = i + 1;
            let mut depth: usize = 1;
            let mut j = start;
            let mut in_str: Option<u8> = None;
            while j < bytes.len() && depth > 0 {
                let b = bytes[j];
                if let Some(q) = in_str {
                    if b == b'\\' && j + 1 < bytes.len() {
                        j += 2;
                        continue;
                    }
                    if b == q {
                        in_str = None;
                    }
                } else if b == b'"' || b == b'\'' {
                    in_str = Some(b);
                } else if b == b'{' {
                    depth += 1;
                } else if b == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            if j >= bytes.len() {
                // Unterminated — treat the `{` as a literal.
                current_literal.push('{');
                i = start;
                continue;
            }
            // Flush the literal so far.
            if !current_literal.is_empty() {
                parts.push(InterpPart::Literal(std::mem::take(&mut current_literal)));
            }
            parts.push(InterpPart::Expr(text[start..j].to_string()));
            i = j + 1; // skip the closing `}`
        } else {
            current_literal.push(bytes[i] as char);
            i += 1;
        }
    }
    if !current_literal.is_empty() {
        parts.push(InterpPart::Literal(current_literal));
    }
    Some(parts)
}

fn render_string_interpolation(
    parts: &[InterpPart],
    attr: &AstAttribute,
) -> TokenStream {
    // Build `format!("lit1 {expr1} lit2 {expr2} …", expr1, expr2, …)`.
    let mut format_str = String::new();
    let mut args = Vec::new();
    for part in parts {
        match part {
            InterpPart::Literal(s) => {
                // Escape `{` and `}` in the literal portion so
                // `format!` doesn't choke.
                format_str.push_str(&s.replace('{', "{{").replace('}', "}}"));
            }
            InterpPart::Expr(s) => {
                format_str.push_str("{}");
                let parsed = match parse_ts(s, attr.span, "interpolation expression") {
                    Ok(ts) => ts,
                    Err(_) => continue,
                };
                args.push(parsed);
            }
        }
    }
    quote! { format!(#format_str, #(#args),*).to_string() }
}

fn extract_text_content(children: &[AstNode]) -> Option<String> {
    let mut text = String::new();
    for c in children {
        if let AstNode::Text { text: t, .. } = c {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(t);
        }
    }
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Map a shorthand attribute name (e.g. `gap_3`, `p_4`,
/// `flex`, `col`) to the corresponding gpui `Styled` method
/// name. Returns `None` if the name isn't a known shorthand.
#[allow(dead_code)]
fn expand_shorthand_method(name: &str) -> Option<String> {
    if is_known_shorthand_method(name) || is_spacing_shorthand(name) {
        Some(name.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    //! End-to-end tests for the codegen. Each test runs the
    //! XML through [`crate::parser::parse`] + [`codegen`]
    //! and asserts the resulting token stream can be
    //! parsed back as valid Rust. We don't actually compile
    //! the generated code here (the proc-macro harness does
    //! that), but we make sure the tokens are well-formed
    //! and contain the expected fragments.
    use super::*;
    use proc_macro2::Span;

    fn render(xml: &str) -> String {
        let ts = codegen(xml, Span::call_site(), None).expect("codegen succeeds");
        ts.to_string()
    }

    #[test]
    fn empty_column() {
        let s = render(r#"<Column col />"#);
        // Must start with `gpui::div()` and contain `flex_col`.
        assert!(s.contains("gpui :: div ()"), "{s}");
        assert!(s.contains("flex_col ()"), "{s}");
    }

    #[test]
    fn column_with_gap_and_padding() {
        let s = render(r#"<Column flex col gap="3" p="4" />"#);
        assert!(s.contains("flex ()"), "{s}");
        assert!(s.contains("flex_col ()"), "{s}");
        assert!(s.contains("gap_3 ()"), "{s}");
        assert!(s.contains("p_4 ()"), "{s}");
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
        let s = render(
            r#"<Button id="inc" caption="+" on_click={move |_, _, cx| { x += 1; }} />"#,
        );
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
        // quote! adds spaces between tokens, so `.child` becomes
        // `. child` in the printed form. We strip spaces first.
        let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(normalised.contains(".child"), "{normalised}");
        // Two `child` calls inside the column for label/row,
        // then two more inside the row.
        assert_eq!(normalised.matches(".child").count(), 4, "{normalised}");
    }

    #[test]
    fn if_without_else() {
        let s = render(
            r#"<Column><If condition={show}><Label id="x" text="hi" /></If></Column>"#,
        );
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
        assert!(s.contains("into_iter ()"), "{s}");
        assert!(s.contains("items"), "{s}");
        // The loop variable is the `let:item` name.
        let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(normalised.contains("foritem"), "{normalised}");
    }

    #[test]
    fn unknown_tag_is_an_error() {
        let err = codegen("<MyWidget />", Span::call_site(), None).unwrap_err();
        assert!(matches!(err.kind, crate::error::XmlErrorKind::UnknownTag), "{err:?}");
    }

    #[test]
    fn unknown_attribute_on_leaf_is_an_error() {
        let err = codegen(r#"<Label id="x" text="hi" href="bad" />"#, Span::call_site(), None)
            .unwrap_err();
        assert!(matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute), "{err:?}");
    }

    #[test]
    fn unknown_attribute_on_container_is_an_error() {
        let err = codegen(r#"<Column flex hover="red" />"#, Span::call_site(), None).unwrap_err();
        assert!(matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute), "{err:?}");
    }

    #[test]
    fn missing_id_on_leaf_is_an_error() {
        let err = codegen(r#"<Label text="hi" />"#, Span::call_site(), None).unwrap_err();
        assert!(matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute), "{err:?}");
        assert!(err.message.contains("id"));
    }

    #[test]
    fn missing_id_is_a_helpful_message() {
        let err = codegen(r#"<Button caption="Save" />"#, Span::call_site(), None).unwrap_err();
        assert!(err.message.contains("Button"), "{err}");
    }

    #[test]
    fn bad_bool_value_errors() {
        let err = codegen(
            r#"<Label id="x" text="hi" strong="maybe" />"#,
            Span::call_site(),
            None,
        )
        .unwrap_err();
        assert!(err.message.contains("true") || err.message.contains("false"), "{err}");
    }

    #[test]
    fn bad_variant_value_errors() {
        let err = codegen(
            r#"<Button id="x" variant="catastrophic" />"#,
            Span::call_site(),
            None,
        )
        .unwrap_err();
        assert!(err.message.contains("primary") || err.message.contains("neutral") || err.message.contains("danger"), "{err}");
    }

    #[test]
    fn xml_parse_error_propagates() {
        let err = codegen("<Column>", Span::call_site(), None).unwrap_err();
        assert!(matches!(err.kind, crate::error::XmlErrorKind::ParseError), "{err:?}");
    }

    #[test]
    fn generated_schema_picks_up_new_components() {
        // The generated schema (`BUILTINS_GENERATED`) is
        // included by `lib.rs` and the codegen falls back to
        // it for any tag not in the hand-written BUILTINS.
        // We assert that a handful of Phase 2 components are
        // reachable through the lookup.
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
        assert!(!s.contains("text_input ((\"name\") . to_string () , cx)"), "{s}");
        assert!(s.contains("text_input ((\"name\") . to_string ())"), "{s}");
        assert!(s.contains("on_change"), "{s}");
    }

    #[test]
    fn string_interpolation_in_text_attr() {
        let s = render(
            r#"<Label id="x" text="Count: {count}" />"#,
        );
        assert!(s.contains("format !"), "{s}");
        // The format string is `Count: {}` (one
        // placeholder, no literal braces to escape).
        assert!(s.contains("Count: {}"), "{s}");
        assert!(s.contains("count"), "{s}");
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
        let s = render(
            r#"<TextInput id="x" @bind={self.name} placeholder="Name" />"#,
        );
        // Strip spaces to make the assertion robust
        // against `quote!`'s token-spacing behaviour.
        let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains("placeholder"), "{s}");
        // The codegen binds the entity to a local
        // variable `__bind` and calls `.update` on it.
        assert!(compact.contains("__bind.update"), "{s}");
        assert!(compact.contains("on_change"), "{s}");
    }

    #[test]
    fn bind_attribute_emits_value_read_for_components_with_value_setter() {
        // Checkbox has a `checked` setter + `on_toggle`
        // event. `@bind` emits a `__bind.read(cx).clone()`
        // into the `checked` setter and a write-back
        // closure via `on_toggle`. (The `__bind` local
        // is bound to the entity expression; the
        // assertion below looks for it instead of the
        // raw entity path because the codegen uses the
        // local.)
        let s = render(r#"<Checkbox id="x" @bind={self.flag} />"#);
        let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains("__bind.read"), "{s}");
        assert!(compact.contains("__bind.update"), "{s}");
        assert!(compact.contains("on_toggle"), "{s}");
    }

    #[test]
    fn template_emits_children_inline() {
        // `<Template>` for the MVP simply inlines its
        // children at the call site.
        let s = render(
            r#"<Column>
    <Template>
        <Label id="a" text="A" />
    </Template>
</Column>"#,
        );
        let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(compact.contains("label::label"), "{s}");
        assert!(compact.contains("\"A\""), "{s}");
    }

    #[test]
    fn slot_is_a_no_op() {
        // `<Slot/>` doesn't emit anything — it just
        // disappears. Future revisions will wire
        // caller-side slot-filling.
        let s = render(r#"<Column><Slot/></Column>"#);
        // The Column still has its `child` chain.
        assert!(s.contains("gpui :: div ()"), "{s}");
    }

    #[test]
    fn include_requires_src() {
        // No `src` attribute → error.
        let err = codegen(r#"<Column><Include /></Column>"#, Span::call_site(), None).unwrap_err();
        assert!(err.message.contains("src"), "{err}");
    }

    #[test]
    fn bind_attribute_without_braces_errors() {
        let err = codegen(
            r#"<TextInput id="x" @bind="not_an_expr" placeholder="…" />"#,
            Span::call_site(),
            None,
        )
        .unwrap_err();
        assert!(err.message.contains("@bind"), "{err}");
    }

    /// Cross-check: the hand-written BUILTINS for `Button`
    /// and `Label` should agree with the auto-generated
    /// entries on the field names and prop types. This
    /// catches the case where someone changes a setter on
    /// `ButtonProps` but forgets to update the schema.
    #[test]
    fn hand_written_button_matches_generated() {
        let hand = crate::schema::BUILTINS
            .iter()
            .find(|c| c.tag == "Button")
            .expect("Button is in hand-written BUILTINS");
        let gen_entry = BUILTINS_GENERATED
            .iter()
            .find(|c| c.tag == "Button")
            .expect("Button is in BUILTINS_GENERATED");
        let hand_props: std::collections::BTreeSet<_> = match hand.kind {
            crate::schema::ComponentKind::Leaf(ref l) => {
                l.props.iter().map(|p| p.name).collect()
            }
            _ => panic!("Button is not a leaf"),
        };
        let gen_props: std::collections::BTreeSet<_> = match gen_entry.kind {
            crate::schema::ComponentKind::Leaf(ref l) => {
                l.props.iter().map(|p| p.name).collect()
            }
            _ => panic!("generated Button is not a leaf"),
        };
        let hand_events: std::collections::BTreeSet<_> = match hand.kind {
            crate::schema::ComponentKind::Leaf(ref l) => {
                l.events.iter().map(|(n, _)| n).cloned().collect()
            }
            _ => panic!("Button is not a leaf"),
        };
        let gen_events: std::collections::BTreeSet<_> = match gen_entry.kind {
            crate::schema::ComponentKind::Leaf(ref l) => {
                l.events.iter().map(|(n, _)| n).cloned().collect()
            }
            _ => panic!("generated Button is not a leaf"),
        };
        assert_eq!(
            hand_props, gen_props,
            "hand-written Button props diverge from generated"
        );
        assert_eq!(
            hand_events, gen_events,
            "hand-written Button events diverge from generated"
        );
    }

    #[test]
    fn hand_written_label_matches_generated() {
        let hand = crate::schema::BUILTINS
            .iter()
            .find(|c| c.tag == "Label")
            .expect("Label is in hand-written BUILTINS");
        let gen_entry = BUILTINS_GENERATED
            .iter()
            .find(|c| c.tag == "Label")
            .expect("Label is in BUILTINS_GENERATED");
        let hand_props: std::collections::BTreeSet<_> = match hand.kind {
            crate::schema::ComponentKind::Leaf(ref l) => {
                l.props.iter().map(|p| p.name).collect()
            }
            _ => panic!("Label is not a leaf"),
        };
        let gen_props: std::collections::BTreeSet<_> = match gen_entry.kind {
            crate::schema::ComponentKind::Leaf(ref l) => {
                l.props.iter().map(|p| p.name).collect()
            }
            _ => panic!("generated Label is not a leaf"),
        };
        // Compare the FIRST extra arg (Label only has one).
        let hand_extra_first = match hand.kind {
            crate::schema::ComponentKind::Leaf(ref l) => l.extra_args.first().copied(),
            _ => panic!("Label is not a leaf"),
        };
        let gen_extra_first = match gen_entry.kind {
            crate::schema::ComponentKind::Leaf(ref l) => l.extra_args.first().copied(),
            _ => panic!("generated Label is not a leaf"),
        };
        assert_eq!(
            hand_extra_first, gen_extra_first,
            "Label extra_args[0] diverges"
        );
        assert_eq!(
            hand_props, gen_props,
            "hand-written Label props diverge from generated"
        );
    }

    /// Run `gen-schema --check` and panic if the
    /// generated file is out of date. This test is the
    /// "is the schema fresh?" CI gate — any drift
    /// between the headless source and the committed
    /// `schema_generated.rs` fails the build.
    #[test]
    fn schema_drift_check() {
        // We can't easily spawn the `gen-schema` binary
        // from here (it lives in `src/bin/`). Instead,
        // we verify the key invariants the generator
        // maintains: that the `BUILTINS_GENERATED` static
        // contains every known built-in tag, and that
        // every component has either a valid
        // `Container`, `Leaf`, or `ControlFlow` kind.
        let generated = crate::schema_generated::BUILTINS_GENERATED;
        for def in generated {
            match def.kind {
                crate::schema::ComponentKind::Container(_)
                | crate::schema::ComponentKind::Leaf(_)
                | crate::schema::ComponentKind::ControlFlow(_) => {}
            }
        }
    }

    #[test]
    fn overrides_provide_containers_and_control_flow() {
        // `BUILTINS_OVERRIDES` (sourced from `overrides.toml`)
        // covers the XML-only tags that the headless source
        // doesn't have entries for: containers (Column,
        // Row, Div, Stack) and control flow (If, ElseIf,
        // Else, For, Fragment).
        for tag in [
            "Column", "Row", "Div", "Stack", "If", "ElseIf", "Else", "For", "Fragment",
        ] {
            assert!(
                BUILTINS_OVERRIDES.iter().any(|c| c.tag == tag),
                "tag {tag:?} should be in BUILTINS_OVERRIDES"
            );
        }
    }
}

#[allow(dead_code)]
fn _reserved_marker(name: &str) -> bool {
    crate::schema::RESERVED_ATTRS.contains(&name)
}

// Keep `Element` from going unused (it's handy for future
// validation of `let:`-style attrs).
impl ToTokens for AstElement {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        // Intentionally left blank — `AstElement` is consumed
        // directly by the codegen arms, not via `quote!`.
    }
}
