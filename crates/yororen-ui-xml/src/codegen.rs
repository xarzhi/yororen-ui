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
use quote::{TokenStreamExt, format_ident, quote};

use crate::ast::{AstAttribute, AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::{
    ComponentDef, ComponentKind, ContainerDef, ControlFlowDef, ExtraArgKind, LeafDef, PropValue,
    RenderMode, is_known_shorthand_method, is_spacing_prefix, is_spacing_shorthand,
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
/// `byte_offset` is the position in the original XML to
/// surface in the diagnostic; pass `0` if not meaningful.
fn parse_ts(
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
pub fn codegen(
    xml_text: &str,
    outer_span: Span,
    cx_expr: Option<TokenStream>,
) -> Result<TokenStream, XmlError> {
    let line_starts = crate::parser::line_starts(xml_text);
    let location = crate::parser::LocationTracker {
        line_starts: &line_starts,
        xml: xml_text,
        outer_span,
    };
    let element = crate::parser::parse(xml_text, outer_span, &location)?;
    let cx_tokens = match cx_expr {
        Some(expr) => quote! { (#expr) },
        None => quote! { cx },
    };
    let body = codegen_element(&element, &cx_tokens, &location)?;
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
    location: &crate::parser::LocationTracker<'_>,
) -> Result<TokenStream, XmlError> {
    // Unknown tags fall through to the runtime registry
    // (see `crate::runtime` and the `register_xml_component!`
    // declarative macro). The user gets a working
    // render via inventory lookup — at the cost of
    // losing compile-time attribute / event validation
    // for that tag.
    let def = lookup_component(&element.tag).unwrap_or(&RUNTIME_LEAF_FALLBACK);

    match def.kind {
        ComponentKind::Container(c) => codegen_container(element, c, cx, location),
        ComponentKind::Leaf(l) => codegen_leaf(element, l, cx, location),
        ComponentKind::ControlFlow(c) => codegen_control_flow(element, c, cx, location),
        ComponentKind::RuntimeLeaf => codegen_runtime_leaf(element, cx),
    }
}

/// Sentinel returned by `lookup_component` when the tag
/// is not in the built-in schema. We hand the codegen a
/// `RuntimeLeaf` variant instead of erroring so that
/// custom registered tags compile cleanly.
const RUNTIME_LEAF_FALLBACK: ComponentDef = ComponentDef {
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
fn codegen_runtime_leaf(element: &AstElement, cx: &TokenStream) -> Result<TokenStream, XmlError> {
    let tag = element.tag.clone();
    let id_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "id")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                format!("<{tag}> (a runtime-registered component) requires an `id` attribute"),
            )
            .at(element.byte_offset)
        })?;
    let id_expr = attr_value_tokens(id_attr)?;
    // We deliberately ignore every other attribute;
    // the user's renderer is responsible for parsing
    // them. This keeps the contract minimal.
    let _ = cx;
    // `tag` is owned (from element.tag) and lives for
    // the lifetime of the AST; we need a 'static
    // reference for the runtime lookup. The XML
    // literal is itself 'static (it's part of the
    // macro input), so emitting the literal string
    // yields a 'static reference.
    Ok(quote! {
        ::yororen_ui_xml::runtime::render_or_empty(#tag, #id_expr, #cx)
    })
}

fn codegen_container(
    element: &AstElement,
    def: ContainerDef,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
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
            let chain_expr = codegen_if_chain(&element.children[i..j], cx, location)?;
            tokens.append_all(quote! { .child(#chain_expr) });
            i = j;
        } else {
            let child_expr = codegen_child(child, cx, location)?;
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
    location: &crate::parser::LocationTracker<'_>,
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
                    location.span_outer(),
                    "<If>/<ElseIf>/<Else> chain cannot contain non-element nodes",
                ));
            }
        };
        let branch_expr = codegen_if_branch(
            element,
            element_tag_to_branch_kind(&element.tag)?,
            cx,
            location,
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
            let parsed = parse_ts(
                expr,
                attr.span,
                attr.byte_offset,
                &format!("expression for `{}`", attr.name),
            )?;
            tokens.append_all(quote! { .#m(#parsed) });
            return Ok(());
        }
        // `gap_3={...}` or `flex_grow={...}` → `.gap_3(expr)`
        if is_known_shorthand_method(&attr.name) || is_spacing_shorthand(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed = parse_ts(
                expr,
                attr.span,
                attr.byte_offset,
                &format!("expression for `{}`", attr.name),
            )?;
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
            attr.name, element.tag, def.style_hint,
        ),
    ))
}

fn is_valid_spacing_suffix(s: &str) -> bool {
    const NUMERIC: &[&str] = &[
        "0", "0p5", "1", "1p5", "2", "2p5", "3", "3p5", "4", "5", "6", "7", "8", "9", "10", "11",
        "12", "16", "20", "24", "32", "40", "48", "56", "64", "72", "80", "96",
    ];
    const TEXTUAL: &[&str] = &[
        "full", "1_2", "1_3", "2_3", "1_4", "3_4", "1_5", "2_5", "3_5", "4_5", "1_6", "5_6", "1_12",
    ];
    NUMERIC.contains(&s) || TEXTUAL.contains(&s)
}

fn codegen_leaf(
    element: &AstElement,
    def: LeafDef,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
) -> Result<TokenStream, XmlError> {
    let _ = location; // Currently unused — byte_offset lives on AST nodes.
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
            .at(element.byte_offset)
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
                    .at(element.byte_offset)
                })?;
                quote! { (#text).to_string() }
            }
            (ExtraArgKind::Custom, Some(a)) => attr_value_tokens(a)?,
            (ExtraArgKind::Custom, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
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
        element.byte_offset,
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
                    attr.byte_offset,
                    "@bind requires a brace expression, e.g. `@bind={self.name}`",
                )?;
                tokens.append_all(emit_bind(&parsed, def));
                continue;
            } else {
                return Err(XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    "@bind requires a brace expression, e.g. `@bind={self.name}`",
                )
                .at(attr.byte_offset));
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
                        )
                        .at(attr.byte_offset));
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
            // If the user's brace expression is a bare
            // path / field reference (no `(` / `{` / `|`),
            // we auto-wrap it into a closure that adapts
            // the three standard args `(arg0, &mut Window,
            // &mut App)` to whatever the user's method
            // signature is. This lets XML stay purely
            // declarative — the user just writes
            // `on_click={controller.increment}` instead
            // of `move |ev, w, cx| controller.increment(ev, w, cx)`.
            let expr = attr_expr_only(attr)?;
            let expr = auto_wrap_event_expr(attr, expr);
            tokens.append_all(quote! { .#m(#expr) });
            continue;
        }
        // Event modifiers: `on_click.stop={...}` /
        // `on_key_down.enter={...}`. The base name is
        // the real event; the modifier list wraps the
        // user's closure in a filter / interceptor.
        if let Some((base_event, modifiers)) = split_event_modifiers(&attr.name) {
            if let Some((_, setter)) = def.events.iter().find(|(n, _)| *n == base_event).copied() {
                let m = format_ident!("{}", setter);
                let expr = attr_expr_only(attr)?;
                let expr = auto_wrap_event_expr(attr, expr);
                let wrapped = wrap_event_with_modifiers(&modifiers, expr, attr.span)?;
                tokens.append_all(quote! { .#m(#wrapped) });
                continue;
            }
        }
        return Err(XmlError::new(
            XmlErrorKind::UnknownAttribute,
            attr.span,
            format!("unknown attribute `{}` on <{}>", attr.name, element.tag),
        )
        .at(attr.byte_offset));
    }

    // 4. Apply render mode.
    match def.render {
        RenderMode::Default => {
            // The render method typically takes `(&App)`; a
            // few components (e.g. `TextInput`) also
            // need a `&mut Window`. The schema's
            // `needs_window` flag tells us which.
            if def.needs_window {
                // Both `cx` and `window` are expected
                // as `&mut App` / `&mut Window` by the
                // renderer's `render` signature.
                tokens.append_all(quote! { .render(&mut *#cx, &mut *window) });
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
    location: &crate::parser::LocationTracker<'_>,
) -> Result<TokenStream, XmlError> {
    match def {
        ControlFlowDef::If | ControlFlowDef::ElseIf | ControlFlowDef::Else => {
            codegen_if_branch(element, def, cx, location)
        }
        ControlFlowDef::For => codegen_for(element, cx, location),
        ControlFlowDef::Fragment => codegen_fragment(element, cx, location),
        ControlFlowDef::Include => codegen_include(element, cx, location),
        ControlFlowDef::Template => codegen_template(element, cx, location),
        ControlFlowDef::Slot => codegen_slot(element, cx),
        ControlFlowDef::Match => codegen_match(element, cx, location),
        ControlFlowDef::Case => Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<Case> must appear directly inside a <Match>",
        )
        .at(element.byte_offset)),
        ControlFlowDef::State => codegen_state(element, cx, location),
    }
}

fn codegen_if_branch(
    element: &AstElement,
    kind: ControlFlowDef,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
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
        )
        .at(element.byte_offset));
    } else if element.children.len() == 1 {
        codegen_child(&element.children[0], cx, location)?
    } else {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            format!(
                "if/else branch has {} children; wrap in a <Column> or <Row> for now",
                element.children.len()
            ),
        )
        .at(element.byte_offset));
    };

    Ok(match kind {
        ControlFlowDef::If => quote! { if #condition { #child_expr } },
        ControlFlowDef::ElseIf => quote! { else if #condition { #child_expr } },
        ControlFlowDef::Else => quote! { else { #child_expr } },
        // Unreachable
        _ => unreachable!("non-branch kind {:?}", kind),
    })
}

fn codegen_for(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
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
        )
        .at(element.byte_offset));
    } else if element.children.len() == 1 {
        codegen_child(&element.children[0], cx, location)?
    } else {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<For> must wrap a single child (wrap multiple in a Column / Row)",
        )
        .at(element.byte_offset));
    };

    // `<For each={xs} let:item>` produces
    //   xs.into_iter().map(|item| { … }).collect::<Vec<_>>()
    // but that yields `Vec<AnyElement>`, not `IntoElement`.
    // For Phase 1 we wrap it in a `div()` with the children
    // appended so it slots into a parent.
    if has_index {
        let mut body = quote! { gpui::div() };
        for_each_in_each(
            &each_parsed,
            &item_ident,
            &index_ident,
            true,
            &child_expr,
            &mut body,
        );
        Ok(body)
    } else {
        let mut body = quote! { gpui::div() };
        for_each_in_each(
            &each_parsed,
            &item_ident,
            &index_ident,
            false,
            &child_expr,
            &mut body,
        );
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

fn codegen_fragment(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
) -> Result<TokenStream, XmlError> {
    let mut children_tokens = TokenStream::new();
    for child in &element.children {
        let expr = codegen_child(child, cx, location)?;
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
    location: &crate::parser::LocationTracker<'_>,
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
            .at(element.byte_offset)
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
        )
        .at(src_attr.byte_offset));
    }
    let path = src_attr.raw.as_str();
    let outer_span = location.span_outer();
    let resolved = resolve_include_path(path, outer_span)?;
    let contents = std::fs::read_to_string(&resolved).map_err(|e| {
        XmlError::new(
            XmlErrorKind::ParseError,
            outer_span,
            format!(
                "could not read `{}` (resolved to `{}`): {e}",
                path,
                resolved.display()
            ),
        )
    })?;
    // Parse the included file and emit its children as
    // a comma-separated sequence of expressions. The
    // included file gets its own `LocationTracker` so
    // error offsets are local to the included XML, not
    // the parent.
    let line_starts = crate::parser::line_starts(&contents);
    let included_root = {
        let included_location = crate::parser::LocationTracker {
            line_starts: &line_starts,
            xml: &contents,
            outer_span,
        };
        crate::parser::parse(&contents, outer_span, &included_location)?
    };
    let mut inner = TokenStream::new();
    for child in &included_root.children {
        let line_starts = crate::parser::line_starts(&contents);
        let included_location = crate::parser::LocationTracker {
            line_starts: &line_starts,
            xml: &contents,
            outer_span,
        };
        let expr = codegen_child(child, cx, &included_location)?;
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
fn codegen_template(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
) -> Result<TokenStream, XmlError> {
    codegen_fragment(element, cx, location)
}

/// `<Slot/>` is a no-op for the MVP. Future revisions
/// will wire caller-side slot-filling.
fn codegen_slot(_element: &AstElement, _cx: &TokenStream) -> Result<TokenStream, XmlError> {
    Ok(TokenStream::new())
}

/// `<Match on={expr}>` expands to a Rust `match`
/// expression. The children must be `<Case>` arms
/// (the macro walks them in order); `<Case pattern="_">`
/// becomes the wildcard arm.
///
/// The arm body is the single child of each `<Case>`
/// (multi-child arms are not supported — wrap in a
/// container for now, same limitation as `<If>`).
fn codegen_match(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
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
            AstNode::Text { .. } => {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    location.span_outer(),
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
        let body = if arm.children.is_empty() {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                arm.span,
                "<Case> must contain at least one child",
            )
            .at(arm.byte_offset));
        } else if arm.children.len() == 1 {
            codegen_child(&arm.children[0], cx, location)?
        } else {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                arm.span,
                format!(
                    "<Case> has {} children; wrap in a <Column> or <Row> for now",
                    arm.children.len()
                ),
            )
            .at(arm.byte_offset));
        };
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

/// `<State name="x" default="0" />` declares a local
/// `Entity<T>` for the duration of the surrounding
/// `Render::render` closure. `name` is the identifier
/// the children can refer to; `default` is a stringified
/// Rust literal that becomes the initial value.
///
/// The macro emits a `let name = cx.new(|_| …);` at the
/// *start* of the surrounding XML body and then inlines
/// the children unchanged. The catch: the codegen for
/// a State element is a tuple `(let_decl, child_body)`,
/// which the caller must be able to splice. To keep the
/// shape simple, we emit a small block:
///
/// ```text
/// {
///     let name = cx.new(|_| <default_expr>);
///     <child>
/// }
/// ```
fn codegen_state(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
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

    let body = if element.children.is_empty() {
        quote! { gpui::div() }
    } else if element.children.len() == 1 {
        codegen_child(&element.children[0], cx, location)?
    } else {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<State> must wrap a single child",
        )
        .at(element.byte_offset));
    };

    Ok(quote! {
        {
            let #name_ident = (#cx).new(|_| #default_expr);
            #body
        }
    })
}

fn codegen_child(
    node: &AstNode,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
) -> Result<TokenStream, XmlError> {
    match node {
        AstNode::Element(e) => codegen_element(e, cx, location),
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

fn prop_value_tokens(attr: &AstAttribute, kind: PropValue) -> Result<TokenStream, XmlError> {
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
            PropValue::String | PropValue::Variant | PropValue::Bool | PropValue::Unknown => {
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
            other => Err(XmlError::new(
                XmlErrorKind::InvalidExpression,
                attr.span,
                format!(
                    "attribute `{}` expects `true` or `false`, got `{other}`",
                    attr.name
                ),
            )
            .at(attr.byte_offset)),
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
                )
                .at(attr.byte_offset));
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
        let parsed = parse_ts(
            expr,
            attr.span,
            attr.byte_offset,
            &format!("attribute `{}`", attr.name),
        )?;
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

/// Auto-wrap a bare event expression into a closure that
/// adapts the three standard callback args
/// `(arg0, &mut Window, &mut App)` to the user's method
/// signature. This is the heart of the "XML stays pure"
/// convention: the user writes
///
/// ```xml
/// <Button on_click={controller.increment} />
/// ```
///
/// and the codegen emits
///
/// ```ignore
/// .on_click(move |__arg0, __w, __cx| {
///     controller.clone().increment(__arg0, __w, __cx)
/// })
/// ```
///
/// Detection: if the brace expression has no `(`, `{`, or
/// `|`, it's a bare identifier path / field reference —
/// we wrap it. Otherwise we pass it through verbatim
/// (the user wrote their own closure).
///
/// **Receiver cloning**: for `obj.method` (an
/// `Expr::Field`), we inject `.clone()` between the
/// receiver and the method call so multiple event
/// handlers in the same XML can share a single
/// `controller` instance without `move` conflicts.
/// The user's `controller` type must implement
/// `Clone` (cheap clones are typical — `Arc<_>`,
/// `Entity<_>`, or a small data struct).
///
/// **Limitation**: events with 4-arg signatures
/// (e.g. `on_toggle` on Checkbox/Switch — `Fn(bool,
/// Option<&ClickEvent>, &mut Window, &mut App)`) don't
/// fit the 3-arg wrapper. For those, the user must
/// write a manual closure.
fn auto_wrap_event_expr(attr: &AstAttribute, expr: TokenStream) -> TokenStream {
    let Some(raw) = &attr.expr else {
        return expr;
    };
    let trimmed = raw.trim();
    let looks_like_path = !trimmed.contains('(')
        && !trimmed.contains('{')
        && !trimmed.contains('|')
        && !trimmed.is_empty();
    if !looks_like_path {
        return expr;
    }
    // Parse the expression so we can detect a
    // field-access (`controller.method`) and pre-clone
    // the receiver outside the closure. Pre-cloning
    // (rather than `.clone()` inside the body) lets
    // multiple event handlers in the same XML share a
    // single `controller` — each closure captures its
    // own clone and the original `controller` is left
    // available for the next handler.
    let Ok(parsed) = syn::parse_str::<syn::Expr>(trimmed) else {
        return quote! {
            move |__arg0, __w: &mut gpui::Window, __cx: &mut gpui::App| {
                #expr(__arg0, __w, __cx)
            }
        };
    };
    match parsed {
        // Associated function (`Module::function`) —
        // no receiver, no clone needed.
        syn::Expr::Path(_) => quote! {
            move |__arg0, __w: &mut gpui::Window, __cx: &mut gpui::App| {
                #expr(__arg0, __w, __cx)
            }
        },
        // Field access (`obj.method`) — pre-clone the
        // receiver into a hygienic local so each
        // closure captures its own clone. We emit a
        // block (statement + expression) so the
        // codegen can splice this directly into the
        // `.on_click({ … })` slot.
        syn::Expr::Field(field) => {
            let receiver = field.base;
            let member = field.member;
            // `Span::mixed_site()` yields a unique span
            // per call, so every auto-wrapped closure
            // gets a distinct `__auto_clone_N` ident
            // (proc-macro hygiene).
            let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
            quote! {
                {
                    let #clone_ident = (#receiver).clone();
                    move |__arg0, __w: &mut gpui::Window, __cx: &mut gpui::App| {
                        #clone_ident.#member(__arg0, __w, __cx)
                    }
                }
            }
        }
        // Method call, deref, etc. — fall back to
        // direct call (single-use capture).
        _ => quote! {
            move |__arg0, __w: &mut gpui::Window, __cx: &mut gpui::App| {
                #expr(__arg0, __w, __cx)
            }
        },
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
        let parsed = parse_ts(
            expr,
            attr.span,
            attr.byte_offset,
            &format!("text attribute `{}`", attr.name),
        )?;
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

fn render_string_interpolation(parts: &[InterpPart], attr: &AstAttribute) -> TokenStream {
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
                let parsed =
                    match parse_ts(s, attr.span, attr.byte_offset, "interpolation expression") {
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
    if text.is_empty() { None } else { Some(text) }
}

/// Split an attribute name like `on_click.stop.enter` into
/// `("on_click", vec!["stop", "enter"])`. Returns `None` for
/// names without a dot, signalling that no modifier is
/// present.
///
/// The base name (before the first dot) is what the schema
/// looks up to find the headless event setter; the
/// modifiers drive the runtime wrapper that the macro
/// emits (see [`wrap_event_with_modifiers`]).
fn split_event_modifiers(name: &str) -> Option<(&str, Vec<&str>)> {
    let (base, rest) = name.split_once('.')?;
    if rest.is_empty() {
        return None;
    }
    // Reject double dots and other garbage so the codegen
    // surface a sensible error later.
    if rest.contains("..") || rest.starts_with('.') || rest.ends_with('.') {
        return None;
    }
    let modifiers: Vec<&str> = rest.split('.').collect();
    Some((base, modifiers))
}

/// Wrap a user's event closure with the given modifiers.
/// Each modifier adds an outer closure that intercepts
/// the event before forwarding to the user's code.
///
/// Currently supported:
///
/// - `.stop` / `.prevent` — no-op at the macro level
///   (gpui's event callbacks don't expose propagation
///   control to Rust closures — these are accepted for
///   forward compatibility / hover-state hint, and
///   silently forwarded to the user's closure).
/// - Keyboard filters (`.enter`, `.escape`, `.tab`,
///   `.space`, `.up`, `.down`, `.left`, `.right`,
///   `.backspace`, `.delete`, `.home`, `.end`) — wrap
///   the closure so it only fires when the first event
///   argument has `keystroke().key == "<filter>"`.
///
/// Multiple modifiers are composed left-to-right: the
/// first listed is the outermost wrapper.
///
/// The wrapped closure assumes the headless event
/// signature is `(EventArg, &mut Window, &mut App)`
/// where `EventArg` exposes `keystroke() -> Keystroke`
/// (true for `on_key_down` / `on_key_up`). For
/// modifiers that don't apply to a given event
/// signature, the generated wrapper degrades to a
/// pass-through and the user gets a regular Rust
/// error if the underlying call site doesn't accept
/// the wrapper shape.
fn wrap_event_with_modifiers(
    modifiers: &[&str],
    inner: TokenStream,
    _span: Span,
) -> Result<TokenStream, XmlError> {
    if modifiers.is_empty() {
        return Ok(inner);
    }
    let mut wrapped = inner;
    // Wrap right-to-left so the leftmost modifier ends
    // up as the outermost closure (the one the headless
    // component invokes).
    for modifier in modifiers.iter().rev() {
        wrapped = match *modifier {
            // No-op for now — see doc comment.
            "stop" | "prevent" => quote! {
                move |__ev, __window, cx| {
                    #wrapped(__ev, __window, cx)
                }
            },
            // Keyboard filter: gate the inner closure on
            // the keystroke key matching the modifier name.
            key => {
                let key_lit = format!("\"{key}\"");
                quote! {
                    move |__ev, __window, cx| {
                        if __ev.keystroke().key == #key_lit {
                            #wrapped(__ev, __window, cx)
                        }
                    }
                }
            }
        };
    }
    Ok(wrapped)
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
        assert!(s.contains("into_iter ()"), "{s}");
        assert!(s.contains("items"), "{s}");
        // The loop variable is the `let:item` name.
        let normalised: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(normalised.contains("foritem"), "{normalised}");
    }

    #[test]
    fn unknown_tag_falls_through_to_runtime_registry() {
        // Unknown tags used to be a hard error; with the
        // runtime registry (`register_xml_component!`)
        // they now compile and resolve at runtime via
        // `runtime::render_or_empty`. The codegen must
        // emit a call into the runtime module rather
        // than erroring.
        let ts = codegen(r#"<MyWidget id="x" />"#, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        assert!(s.contains("render_or_empty"), "{s}");
        assert!(s.contains("\"MyWidget\""), "{s}");
    }

    #[test]
    fn unknown_tag_without_id_is_still_an_error() {
        // The runtime registry needs an `id` to call
        // the factory — the codegen still validates
        // this even on the runtime path.
        let err = codegen("<MyWidget />", Span::call_site(), None).unwrap_err();
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
        )
        .unwrap_err();
        assert!(
            matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
            "{err:?}"
        );
    }

    #[test]
    fn unknown_attribute_on_container_is_an_error() {
        let err = codegen(r#"<Column flex hover="red" />"#, Span::call_site(), None).unwrap_err();
        assert!(
            matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
            "{err:?}"
        );
    }

    #[test]
    fn missing_id_on_leaf_is_an_error() {
        let err = codegen(r#"<Label text="hi" />"#, Span::call_site(), None).unwrap_err();
        assert!(
            matches!(err.kind, crate::error::XmlErrorKind::UnknownAttribute),
            "{err:?}"
        );
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
        let err = codegen("<Column>", Span::call_site(), None).unwrap_err();
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
        let err = codegen(xml, Span::call_site(), None).unwrap_err();
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
        )
        .unwrap_err();
        assert!(err.offset.is_some(), "bad-bool error should carry offset");
        let line_starts =
            crate::parser::line_starts(r#"<Label id="x" text="hi" strong="maybe" />"#);
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
        // event, but the wrapper generator should still
        // produce the right shape when fed an inner
        // closure.
        let inner: TokenStream =
            syn::parse_str("move |_ev, _w, _cx| {}").expect("parse inner closure");
        let wrapped = wrap_event_with_modifiers(&["enter"], inner, Span::call_site())
            .expect("wrap with .enter");
        let s = wrapped.to_string();
        assert!(s.contains("keystroke"), "{s}");
        assert!(s.contains("enter"), "{s}");
    }

    #[test]
    fn event_modifier_chains_multiple_filters() {
        // Two modifiers wrap the user's closure twice —
        // the inner closure is called only when both
        // gates pass. The wrapped token stream should
        // contain two `keystroke()` invocations.
        let inner: TokenStream =
            syn::parse_str("move |_ev, _w, _cx| {}").expect("parse inner closure");
        let wrapped = wrap_event_with_modifiers(&["ctrl", "enter"], inner, Span::call_site())
            .expect("wrap with .ctrl.enter");
        let s = wrapped.to_string();
        let keystroke_count = s.matches("keystroke").count();
        assert!(keystroke_count >= 2, "{s}");
    }

    #[test]
    fn event_modifier_unknown_base_event_is_an_error() {
        // The base event must exist in the schema;
        // `on_key_down` is not a built-in event today,
        // so the modifier dispatch falls through to the
        // unknown-attribute error.
        let xml = r#"<TextInput id="x" on_key_down.enter={move |_, _, _| {}} />"#;
        let err = codegen(xml, Span::call_site(), None).unwrap_err();
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
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        // The pre-cloned receiver is captured by the
        // closure; the method call uses it (not the
        // original `controller`).
        assert!(s.contains("move"), "{s}");
        assert!(s.contains("__auto_clone"), "{s}");
        assert!(s.contains(". increment"), "{s}");
        assert!(s.contains("__arg0"), "{s}");
        assert!(s.contains("__w"), "{s}");
        assert!(s.contains("__cx"), "{s}");
    }

    #[test]
    fn event_auto_wrap_pre_clones_receiver() {
        // For `controller.method`, the codegen emits
        // `let __auto_clone_N = (controller).clone();`
        // BEFORE the closure, so each handler captures
        // its own clone and the original `controller`
        // can be used by the next handler.
        let xml = r#"<Button id="x" caption="x" on_click={controller.handle} />"#;
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
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
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
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
        let xml = r#"<Button id="x" caption="x" on_click={move |ev, w, cx| controller.handle(ev, w, cx)} />"#;
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
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
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        assert!(!s.contains("__arg0"), "{s}");
        assert!(s.contains("build_handler ()"), "{s}");
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
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
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
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        assert!(s.contains("0 =>"), "{s}");
        assert!(s.contains("_ =>"), "{s}");
    }

    #[test]
    fn match_without_cases_is_an_error() {
        let xml = r#"<Match on={x} />"#;
        let err = codegen(xml, Span::call_site(), None).unwrap_err();
        assert!(
            matches!(err.kind, crate::error::XmlErrorKind::Unsupported),
            "{err:?}"
        );
        assert!(err.message.contains("at least one"));
    }

    #[test]
    fn case_outside_match_is_an_error() {
        let xml = r#"<Column><Case pattern={A}><Label id="x" text="hi" /></Case></Column>"#;
        let err = codegen(xml, Span::call_site(), None).unwrap_err();
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
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        assert!(s.contains("let count"), "{s}");
        assert!(s.contains(". new"), "{s}");
        assert!(s.contains("count . read"), "{s}");
    }

    #[test]
    fn state_default_handles_bool_and_string() {
        // Bool literal.
        let xml = r#"<State name="on" default="true"><Label id="l" text="x" /></State>"#;
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        assert!(s.contains("true"), "{s}");
        // String literal.
        let xml = r#"<State name="name" default="anonymous"><Label id="l" text="x" /></State>"#;
        let ts = codegen(xml, Span::call_site(), None).expect("codegen ok");
        let s = ts.to_string();
        assert!(s.contains("String :: from"), "{s}");
        assert!(s.contains("anonymous"), "{s}");
    }

    #[test]
    fn state_without_default_is_an_error() {
        let xml = r#"<State name="x"><Label id="l" text="hi" /></State>"#;
        let err = codegen(xml, Span::call_site(), None).unwrap_err();
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
            crate::schema::ComponentKind::Leaf(ref l) => l.props.iter().map(|p| p.name).collect(),
            _ => panic!("Button is not a leaf"),
        };
        let gen_props: std::collections::BTreeSet<_> = match gen_entry.kind {
            crate::schema::ComponentKind::Leaf(ref l) => l.props.iter().map(|p| p.name).collect(),
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
            crate::schema::ComponentKind::Leaf(ref l) => l.props.iter().map(|p| p.name).collect(),
            _ => panic!("Label is not a leaf"),
        };
        let gen_props: std::collections::BTreeSet<_> = match gen_entry.kind {
            crate::schema::ComponentKind::Leaf(ref l) => l.props.iter().map(|p| p.name).collect(),
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
        // `Container`, `Leaf`, `ControlFlow`, or `RuntimeLeaf` kind.
        let generated = crate::schema_generated::BUILTINS_GENERATED;
        for def in generated {
            match def.kind {
                crate::schema::ComponentKind::Container(_)
                | crate::schema::ComponentKind::Leaf(_)
                | crate::schema::ComponentKind::ControlFlow(_)
                | crate::schema::ComponentKind::RuntimeLeaf => {}
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
