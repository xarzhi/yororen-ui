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

std::thread_local! {
    /// User-supplied component schema injected by the
    /// proc-macro entry point. The macro reads
    /// `yororen-ui-xml-components.toml` at the call site
    /// and stashes the definitions here for the duration
    /// of the codegen pass.
    static USER_SCHEMA: std::cell::RefCell<Vec<ComponentDef>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Look up a tag using the currently active user schema.
fn lookup_with_user_schema(tag: &str) -> Option<ComponentDef> {
    USER_SCHEMA.with(|s| {
        let borrowed = s.borrow();
        crate::schema::lookup_component_owned(tag, &*borrowed)
    })
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
    codegen_with_includes(xml_text, outer_span, cx_expr, source_file, user_schema)
        .map(|(ts, _)| ts)
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
    USER_SCHEMA.with(|s| *s.borrow_mut() = user_schema.to_vec());
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
    let body = codegen_element(&element, &cx_tokens, &location, source_file)?;
    // The generated body uses fully-qualified trait method
    // calls (`::gpui::Styled::#m(__el, …)`,
    // `::gpui::ParentElement::child(__el, …)`, etc.) so the
    // caller does not need to import any gpui traits. The
    // result is a plain block expression returning the root
    // element.
    Ok((quote! { { #body } }, included_paths))
}

/// Walk the AST and lift every `<Template name="…">`
/// (at any depth) into a name → body map. Returns
/// `Err` if a duplicate name is found — the user gets a
/// helpful error pointing at the second definition.
///
/// Templates are file-scoped: they can live anywhere in
/// the XML — at the root, nested inside a `<Column>`, or
/// alongside other content. The pre-pass walks the whole
/// tree so the user doesn't have to remember to put
/// them at the top.
fn collect_templates(
    root: &AstElement,
) -> Result<std::collections::HashMap<String, AstElement>, XmlError> {
    use std::collections::HashMap;
    let mut templates: HashMap<String, AstElement> = HashMap::new();
    walk_for_templates(root, &mut templates)?;
    Ok(templates)
}

/// Recursive helper for `collect_templates`. Visits
/// every element in the tree and registers any
/// `<Template name="…">` it finds.
fn walk_for_templates(
    el: &AstElement,
    out: &mut std::collections::HashMap<String, AstElement>,
) -> Result<(), XmlError> {
    if el.tag == "Template" {
        let name_attr = el.attributes.iter().find(|a| a.name == "name");
        let Some(name_attr) = name_attr else {
            return Err(XmlError::new(
                XmlErrorKind::UnknownAttribute,
                el.span,
                "<Template> requires a `name=\"…\"` attribute",
            )
            .at(el.byte_offset));
        };
        if name_attr.expr.is_some() {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                name_attr.span,
                "<Template name> requires a literal identifier, not a brace expression",
            )
            .at(name_attr.byte_offset));
        }
        let name = name_attr.raw.clone();
        if out.contains_key(&name) {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                el.span,
                format!("duplicate <Template name=\"{name}\"> — template names must be unique"),
            )
            .at(el.byte_offset));
        }
        out.insert(name, el.clone());
    }
    for child in &el.children {
        if let AstNode::Element(c) = child {
            walk_for_templates(c, out)?;
        }
    }
    Ok(())
}

/// Walk the AST in-place. Every `<X>` whose tag matches a
/// template name is replaced with the template body, with
/// each `<Slot>` substituted by the caller's matching
/// content. Children of `<X>` that aren't wrapped in
/// `<Slot name="…">` go to the *default* slot (any
/// unnamed `<Slot/>` in the template).
///
/// The transformation recurses into the template body so
/// nested template calls work; it also recurses into the
/// caller-side slot content so `<If>` / `<For>` inside
/// slots are handled normally.
fn expand_template_invocations(
    element: &mut AstElement,
    templates: &std::collections::HashMap<String, AstElement>,
    outer_span: Span,
) -> Result<(), XmlError> {
    // First, recurse into the existing children — they may
    // themselves contain template calls (nested templates).
    let mut new_children: Vec<AstNode> = Vec::new();
    for child in &mut element.children {
        if let AstNode::Element(e) = child {
            expand_template_invocations(e, templates, outer_span)?;
            // After recursion, drop `<Template>` definitions
            // (they're compile-time-only, never emitted).
            if e.tag == "Template" {
                continue;
            }
            // If the (post-recursion) element is a template
            // invocation, splice in the expansion.
            if let Some(template) = templates.get(&e.tag) {
                let expanded = instantiate_template(template, e, outer_span)?;
                new_children.push(expanded);
                continue;
            }
        }
        new_children.push(child.clone());
    }
    element.children = new_children;
    Ok(())
}

/// Materialise a single `<X>` invocation. The caller-side
/// children of `<X>` are split into named-slot content
/// (children of `<Slot name="…">` inside the call) and
/// the default slot (everything else). The template body
/// is then cloned and each `<Slot>` inside is replaced by
/// the matching caller-side content.
fn instantiate_template(
    template: &AstElement,
    call: &AstElement,
    outer_span: Span,
) -> Result<AstNode, XmlError> {
    // Split the call's children into named-slot content and
    // the default slot. A child is a "named slot" iff it's
    // an element whose tag is `Slot` AND has a `name=`
    // attribute; otherwise it contributes to the default
    // slot.
    let mut named_slots: std::collections::HashMap<String, Vec<AstNode>> =
        std::collections::HashMap::new();
    let mut default_slot: Vec<AstNode> = Vec::new();
    for child in &call.children {
        if let AstNode::Element(e) = child
            && e.tag == "Slot"
        {
            let name_attr = e.attributes.iter().find(|a| a.name == "name");
            if let Some(name_attr) = name_attr {
                if name_attr.expr.is_some() {
                    return Err(XmlError::new(
                        XmlErrorKind::Unsupported,
                        name_attr.span,
                        "<Slot name> requires a literal identifier, not a brace expression",
                    )
                    .at(name_attr.byte_offset));
                }
                let name = name_attr.raw.clone();
                named_slots.insert(name, e.children.clone());
                continue;
            }
            // Unnamed `<Slot>` at the call site: errors.
            // The user should put the default-slot content
            // directly inside `<X>`, not wrapped in `<Slot/>`.
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                e.span,
                "<Slot/> at the call site must have a `name` attribute — default-slot content goes directly inside the template call",
            )
            .at(e.byte_offset));
        }
        default_slot.push(child.clone());
    }

    // The template's body must be a single child (or a
    // Fragment-like sequence). Multiple top-level children are
    // wrapped in a synthetic `<Fragment>` so the result composes
    // uniformly.
    let mut body = template.children.clone();
    // If the template body is empty, treat it as a single
    // empty Fragment.
    if body.is_empty() {
        body.push(AstNode::Element(AstElement {
            tag: "Fragment".to_string(),
            span: outer_span,
            byte_offset: 0,
            attributes: Vec::new(),
            children: Vec::new(),
        }));
    }
    // Walk the body and replace every `<Slot>` with the
    // matching content. A `<Slot name="X">` (with explicit
    // name) is replaced by `named_slots["X"]`; `<Slot/>`
    // (no name) is replaced by the default slot. Missing
    // named slots → empty content (so the template's
    // structure is preserved).
    substitute_slots(&mut body, &named_slots, &default_slot)?;
    // If there's exactly one body child, return it directly;
    // otherwise wrap in a synthetic Fragment so the parent's
    // `.child(...)` chain works.
    if body.len() == 1 {
        Ok(body.into_iter().next().unwrap())
    } else {
        Ok(AstNode::Element(AstElement {
            tag: "Fragment".to_string(),
            span: outer_span,
            byte_offset: 0,
            attributes: Vec::new(),
            children: body,
        }))
    }
}

/// Replace every `<Slot name="…">` / `<Slot/>` in `body`
/// with the matching caller-side content. Recurses into
/// nested elements (a `<Slot>` inside an `<If>` is still
/// a slot). Multiple `<Slot>` with the same name in the
/// template are all replaced (one-to-many fan-out).
fn substitute_slots(
    body: &mut Vec<AstNode>,
    named_slots: &std::collections::HashMap<String, Vec<AstNode>>,
    default_slot: &[AstNode],
) -> Result<(), XmlError> {
    // Walk the body, building a new children list. When we
    // hit a `<Slot>`, we splice in the matching content
    // (replacing the Slot, not wrapping it). For non-slot
    // elements, recurse into their children so nested slots
    // get substituted too.
    let mut out: Vec<AstNode> = Vec::new();
    for node in body.drain(..) {
        if let AstNode::Element(e) = &node
            && e.tag == "Slot"
        {
            let name_attr = e.attributes.iter().find(|a| a.name == "name");
            let replacement = match name_attr {
                Some(n) => {
                    if n.expr.is_some() {
                        return Err(XmlError::new(
                            XmlErrorKind::Unsupported,
                            n.span,
                            "<Slot name> inside a template must be a literal",
                        )
                        .at(n.byte_offset));
                    }
                    named_slots.get(&n.raw).cloned().unwrap_or_default()
                }
                None => default_slot.to_vec(),
            };
            out.extend(replacement);
            continue;
        }
        // Non-slot: recurse into the element's own children
        // so a `<Slot>` nested inside (e.g. inside an
        // `<If>`) is also substituted. The element itself
        // passes through unchanged.
        if let AstNode::Element(mut e) = node {
            substitute_slots(&mut e.children, named_slots, default_slot)?;
            out.push(AstNode::Element(e));
        } else {
            out.push(node);
        }
    }
    *body = out;
    Ok(())
}

/// Threaded-through context for the codegen recursion. Holds
/// the `cx` expression, the source-file path (used by
/// `<Include>` to resolve relative paths), and the
/// `LocationTracker` for diagnostics.
///
/// Not currently used as a parameter on every helper (most
/// only need `cx` + `location`); kept as a doc-level
/// reference for the source-file threading and as a
/// future-proofing hook if we want to push more state
/// through the recursion.
#[allow(dead_code)]
struct CodegenCtx<'a> {
    cx: &'a TokenStream,
    source_file: Option<&'a str>,
    location: &'a crate::parser::LocationTracker<'a>,
}

fn codegen_element(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    // Unknown tags fall through to the runtime registry
    // (see `crate::runtime` and the `register_xml_component!`
    // declarative macro). The user gets a working
    // render via inventory lookup — at the cost of
    // losing compile-time attribute / event validation
    // for that tag.
    let def = lookup_with_user_schema(&element.tag).unwrap_or(RUNTIME_LEAF_FALLBACK.clone());

    match def.kind {
        ComponentKind::Container(c) => codegen_container(element, c, cx, location, source_file),
        ComponentKind::Leaf(l) => codegen_leaf(element, l, cx, location, source_file, true),
        ComponentKind::ControlFlow(c) => {
            codegen_control_flow(element, c, cx, location, source_file)
        }
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
            let builtins = crate::schema::builtin_tags();
            let suggestion = did_you_mean(&tag, &builtins.iter().map(|s| *s).collect::<Vec<_>>());
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
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    // Build the container as a sequence of `let __el = ...;`
    // statements rather than one giant method chain. This keeps
    // the type-checker happy when the XML grows large and
    // contains many closures (workaround for a rustc quirk
    // where huge closure-bearing expressions can be mis-parsed
    // in `impl Trait` return positions).
    let mut stmts: Vec<TokenStream> = Vec::new();
    stmts.push(quote! { let __el = gpui::div(); });

    for attr in &element.attributes {
        apply_container_attr(&mut stmts, attr, def, element)?;
    }

    // Walk children, merging consecutive If/ElseIf/Else
    // into a single Rust if/else chain (which must be a
    // single block expression so it can be the argument
    // of `ParentElement::child`).
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
            let chain_expr = codegen_if_chain(&element.children[i..j], cx, location, source_file)?;
            stmts.push(quote! { let __el = ::gpui::ParentElement::child(__el, #chain_expr); });
            i = j;
        } else {
            let child_expr = codegen_child(child, cx, location, source_file)?;
            stmts.push(quote! { let __el = ::gpui::ParentElement::child(__el, #child_expr); });
            i += 1;
        }
    }

    Ok(quote! {
        {
            #(#stmts)*
            __el
        }
    })
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
    source_file: Option<&str>,
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
            source_file,
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
    if branches
        .last()
        .and_then(|b| match b {
            AstNode::Element(e) => Some(e.tag.as_str()),
            _ => None,
        })
        != Some("Else")
    {
        chain.append_all(quote! { else { ::gpui::IntoElement::into_any_element(gpui::div()) } });
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
    stmts: &mut Vec<TokenStream>,
    attr: &AstAttribute,
    def: ContainerDef,
    element: &AstElement,
) -> Result<(), XmlError> {
    if attr.name == "id" {
        // `id="my-button"` becomes
        // `::gpui::InteractiveElement::id(__el, "my-button".into())`.
        let value = attr_value_tokens(attr)?;
        stmts.push(quote! {
            let __el = ::gpui::InteractiveElement::id(__el, #value);
        });
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
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el);
            });
            return Ok(());
        }
    }

    // Brace expression on a container: pass through to the
    // matching Styled method.
    if let Some(expr) = &attr.expr {
        // `gap={pixels}` → `::gpui::Styled::gap(__el, pixels)`
        if is_spacing_prefix(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed = parse_ts(
                expr,
                attr.span,
                attr.byte_offset,
                &format!("expression for `{}`", attr.name),
            )?;
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el, #parsed);
            });
            return Ok(());
        }
        // `gap_3={...}` or `flex_grow={...}` →
        // `::gpui::Styled::gap_3(__el, expr)`
        if is_known_shorthand_method(&attr.name) || is_spacing_shorthand(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed = parse_ts(
                expr,
                attr.span,
                attr.byte_offset,
                &format!("expression for `{}`", attr.name),
            )?;
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el, #parsed);
            });
            return Ok(());
        }
    }

    // Literal value on a spacing prefix: `gap="3"` →
    // `::gpui::Styled::gap_3(__el)`.
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
        stmts.push(quote! {
            let __el = ::gpui::Styled::#m(__el);
        });
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
        let m = format_ident!("{}", attr.name);
        if attr.raw == "true" {
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el);
            });
            return Ok(());
        }
        // `flex_grow="0.5"` — odd but possible; we just
        // pass the value as a string to the method.
        let raw = attr.raw.as_str();
        stmts.push(quote! {
            let __el = ::gpui::Styled::#m(__el, #raw);
        });
        return Ok(());
    }

    let accepted = accepted_container_attrs(&def);
    let suggestion = did_you_mean(
        &attr.name,
        &accepted
            .split(", ")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>(),
    );
    let hint = if let Some(s) = suggestion {
        format!(" — did you mean `{}`?", s)
    } else {
        String::new()
    };
    Err(XmlError::new(
        XmlErrorKind::UnknownAttribute,
        attr.span,
        format!(
            "unknown attribute `{}` on <{}>; containers only accept shorthand style attributes ({accepted}){hint}",
            attr.name, element.tag,
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
    source_file: Option<&str>,
    wrap_to_any: bool,
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
            (ExtraArgKind::Callback, Some(a)) => {
                let expr = attr_value_tokens(a)?;
                auto_wrap_callback_expr(a, expr)
            }
            (ExtraArgKind::Callback, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::UInt, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    let value = raw.parse::<usize>().map_err(|_| {
                        XmlError::new(
                            XmlErrorKind::InvalidExpression,
                            a.span,
                            format!(
                                "attribute `{}` expects a usize literal, got `{raw}`",
                                a.name
                            ),
                        )
                        .at(a.byte_offset)
                    })?;
                    let lit = proc_macro2::Literal::usize_unsuffixed(value);
                    quote! { #lit }
                }
            }
            (ExtraArgKind::UInt, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::HeadingLevel, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    let variant = match raw {
                        "H1" | "h1" => "H1",
                        "H2" | "h2" => "H2",
                        "H3" | "h3" => "H3",
                        "H4" | "h4" => "H4",
                        "H5" | "h5" => "H5",
                        "H6" | "h6" => "H6",
                        other => {
                            return Err(XmlError::new(
                                XmlErrorKind::InvalidExpression,
                                a.span,
                                format!("attribute `{}` expects H1..H6, got `{other}`", a.name),
                            )
                            .at(a.byte_offset));
                        }
                    };
                    let variant = format_ident!("{variant}");
                    quote! { ::yororen_ui::headless::heading::HeadingLevel::#variant }
                }
            }
            (ExtraArgKind::HeadingLevel, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::IconSource, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    quote! {
                        ::yororen_ui::headless::icon::IconSource::Builtin((#raw).into())
                    }
                }
            }
            (ExtraArgKind::IconSource, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::ImageSource, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    quote! {
                        ::yororen_ui::headless::image::ImageSource::Resource((#raw).into())
                    }
                }
            }
            (ExtraArgKind::ImageSource, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::KeybindingInputMode, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    let variant = match raw {
                        "Idle" | "idle" => "Idle",
                        "Capturing" | "capturing" => "Capturing",
                        other => {
                            return Err(XmlError::new(
                                XmlErrorKind::InvalidExpression,
                                a.span,
                                format!(
                                    "attribute `{}` expects Idle or Capturing, got `{other}`",
                                    a.name
                                ),
                            )
                            .at(a.byte_offset));
                        }
                    };
                    let variant = format_ident!("{variant}");
                    quote! { ::yororen_ui::headless::keybinding_input::KeybindingInputMode::#variant }
                }
            }
            (ExtraArgKind::KeybindingInputMode, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::Color, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    parse_hex_color(a.raw.as_str(), a)?
                }
            }
            (ExtraArgKind::Color, None) => {
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

    // Build the leaf as a sequence of statements so trait
    // methods can be fully qualified and the final type is
    // `AnyElement` without relying on the call site to import
    // `IntoElement`, `ParentElement`, etc.
    let mut stmts: Vec<TokenStream> = Vec::new();
    stmts.push(quote! { let mut __el = #factory(#(#factory_args),*); });

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
                stmts.extend(emit_bind(&parsed, def, cx));
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
                    if attr.expr.is_some() {
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
                        stmts.push(quote! { __el = __el.#m(); });
                    }
                    // `raw == "false"` → skip the call (the
                    // default for unset).
                    continue;
                }
                _ => {
                    let value = prop_value_tokens(attr, prop.value)?;
                    stmts.push(quote! { __el = __el.#m(#value); });
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
            let expr = auto_wrap_event_expr(attr, expr, setter, &element.tag);
            // Component event setters are inherent methods on
            // the component builder (e.g. `ButtonProps::on_click`),
            // so a normal method call is enough and avoids
            // requiring `StatefulInteractiveElement` to be in
            // scope at the call site.
            stmts.push(quote! {
                __el = __el.#m(#expr);
            });
            continue;
        }
        // Event modifiers: `on_click.stop={...}` /
        // `on_key_down.enter={...}`. The base name is
        // the real event; the modifier list wraps the
        // user's closure in a filter / interceptor.
        if let Some((base_event, modifiers)) = split_event_modifiers(&attr.name)
            && let Some((_, setter)) = def.events.iter().find(|(n, _)| *n == base_event).copied()
        {
            let m = format_ident!("{}", setter);
            let expr = attr_expr_only(attr)?;
            // For modifiers we build the closure body inline
            // rather than wrapping an already-auto-wrapped
            // closure. This keeps the receiver clone outside
            // the `move` closure so the original binding
            // (e.g. `controller`) is not captured and can be
            // reused by other handlers.
            let (clone_stmt, call_expr) = auto_wrap_event_call(attr, expr);
            let body = wrap_event_body_with_modifiers(&modifiers, call_expr, attr.span)?;
            let closure = if let Some(stmt) = clone_stmt {
                quote! {
                    {
                        #stmt
                        move |__ev, __window, cx| { #body }
                    }
                }
            } else {
                quote! {
                    move |__ev, __window, cx| { #body }
                }
            };
            stmts.push(quote! {
                __el = __el.#m(#closure);
            });
            continue;
        }
        // Leaf style pass-through: attributes that look like gpui
        // `Styled` methods (`w_full`, `h={px(16)}`, `border_1`, …)
        // are deferred and applied to the rendered element, so callers
        // can size leaves the same way they size `<Div>` containers.
        if is_leaf_style_attr(&attr.name, &def) {
            continue;
        }

        let mut accepted = accepted_leaf_attrs(&def);
        if let Some(suggestion) = did_you_mean(
            &attr.name,
            &accepted
                .split(", ")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>(),
        ) {
            accepted.push_str(&format!(" — did you mean `{}`?", suggestion));
        }
        return Err(XmlError::new(
            XmlErrorKind::UnknownAttribute,
            attr.span,
            format!(
                "unknown attribute `{}` on <{}>; accepted: {accepted}",
                attr.name, element.tag
            ),
        )
        .at(attr.byte_offset));
    }

    // 4. Named slots (e.g. Popover trigger/content, Tooltip trigger).
    // These builder methods are called before `.render(cx)`.
    let mut slotted_children: Vec<(&str, AstElement)> = Vec::new();
    let mut remaining_children: Vec<AstNode> = Vec::new();
    for child in &element.children {
        if let AstNode::Element(child_el) = child {
            if let Some(slot_attr) = child_el.attributes.iter().find(|a| a.name == "slot") {
                let slot_name = slot_attr.raw.as_str();
                if let Some(slot_def) = def.slots.iter().find(|s| s.name == slot_name) {
                    let mut stripped = child_el.clone();
                    stripped.attributes.retain(|a| a.name != "slot");
                    slotted_children.push((slot_def.setter, stripped));
                    continue;
                }
            }
        }
        remaining_children.push(child.clone());
    }
    for (setter, child_el) in &slotted_children {
        let setter = format_ident!("{setter}");
        let child_expr = codegen_child(
            &AstNode::Element(child_el.clone()),
            cx,
            location,
            source_file,
        )?;
        stmts.push(quote! { __el = __el.#setter(#child_expr); });
    }

    // 5. Children consumed before render (ButtonGroup, Modal).
    if def.children_before_render {
        for child in &remaining_children {
            match child {
                AstNode::Text { .. } => {
                    return Err(XmlError::new(
                        XmlErrorKind::Unsupported,
                        element.span,
                        format!(
                            "<{tag}> does not support text content; use a <Label> child",
                            tag = element.tag
                        ),
                    )
                    .at(element.byte_offset));
                }
                AstNode::Expr { .. } | AstNode::Element(_) => {
                    let child_expr = if def.unwrap_children {
                        codegen_child_unwrapped(child, cx, location, source_file)?
                    } else {
                        codegen_child(child, cx, location, source_file)?
                    };
                    stmts.push(quote! {
                        __el = __el.child(#child_expr);
                    });
                }
            }
        }
    }

    // 6. Form submit button is built from the props before render,
    //    then attached after render.
    let is_form = element.tag == "Form";
    let has_submit = is_form && element.attributes.iter().any(|a| a.name == "submit");
    if has_submit {
        stmts.push(quote! { let __form_submit_btn = __el.submit_button(&mut *#cx); });
    }

    // 7. Apply render mode.
    // After `.render(...)` the type changes from the
    // component's props/builder to `AnyElement`, so we
    // shadow `__el` with a fresh `let` rather than
    // reassigning (which would fix the original type).
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
                stmts.push(quote! { let __el = __el.render(&mut *#cx, &mut *window); });
            } else {
                let app_ref = quote! { &*#cx };
                stmts.push(quote! { let __el = __el.render(#app_ref); });
            }
        }
        RenderMode::Apply => {
            // Caller is responsible for `.apply(div())` — for
            // now, do nothing.
        }
    }

    // 8. Style pass-through for known `Styled` attributes.
    //    These are applied after `.render(cx)` so they affect
    //    the final rendered element (e.g. `<Spacer w_full h={px(16)}/>`).
    if def.render == RenderMode::Default {
        let style_container_def = crate::schema::ContainerDef {
            fixed_methods: &[],
            style_hint: "the gpui Styled trait (`.w(...)`, `.h(...)`, `.border_1()`, …)",
        };
        for attr in &element.attributes {
            if !is_leaf_style_attr(&attr.name, &def) {
                continue;
            }
            apply_container_attr(&mut stmts, attr, style_container_def, element)?;
        }
    }

    // 9. Children added after render (the default path).
    // For Default-rendered leaves, the rendered element is
    // typically a `Stateful<Div>` that accepts `.child(...)`.
    // Text children are only allowed for components that explicitly
    // opt in (e.g. `<Button>Click me</Button>`); other text inside a
    // leaf is an error.
    if def.render == RenderMode::Default
        && !def.children_before_render
        && !remaining_children.is_empty()
    {
        let text_opt = extract_text_content(&remaining_children);
        let mut text_added = false;
        let mut child_stmts: Vec<TokenStream> = Vec::new();
        let is_if_element = |node: &AstNode| -> bool {
            matches!(
                node,
                AstNode::Element(e) if matches!(e.tag.as_str(), "If" | "ElseIf" | "Else")
            )
        };
        let mut i = 0;
        while i < remaining_children.len() {
            if is_if_element(&remaining_children[i]) {
                // Merge consecutive If/ElseIf/Else siblings into a
                // single Rust if/else chain, just like containers do.
                let mut j = i;
                while j < remaining_children.len() && is_if_element(&remaining_children[j]) {
                    j += 1;
                }
                let chain_expr =
                    codegen_if_chain(&remaining_children[i..j], cx, location, source_file)?;
                child_stmts.push(quote! {
                    let __el = ::gpui::ParentElement::child(__el, #chain_expr);
                });
                i = j;
            } else {
                let child = &remaining_children[i];
                match child {
                    AstNode::Text { .. } => {
                        if def.supports_text_child {
                            if let Some(text) = &text_opt {
                                if !text_added {
                                    text_added = true;
                                    child_stmts.push(quote! {
                                        let __el = ::gpui::ParentElement::child(__el, #text);
                                    });
                                }
                            }
                        } else {
                            return Err(XmlError::new(
                                XmlErrorKind::Unsupported,
                                element.span,
                                format!(
                                    "<{tag}> does not support text content; wrap text in a <Label>",
                                    tag = element.tag
                                ),
                            )
                            .at(element.byte_offset));
                        }
                    }
                    AstNode::Expr { .. } | AstNode::Element(_) => {
                        let child_expr = codegen_child(child, cx, location, source_file)?;
                        child_stmts.push(quote! {
                            let __el = ::gpui::ParentElement::child(__el, #child_expr);
                        });
                    }
                }
                i += 1;
            }
        }
        stmts.extend(child_stmts);
    }

    // 9. Attach the Form submit button after render.
    if has_submit {
        stmts.push(quote! {
            let __el = if let Some(__btn) = __form_submit_btn {
                ::gpui::ParentElement::child(__el, __btn)
            } else {
                __el
            };
        });
    }

    // 10. Wrap to AnyElement so the result composes into a parent.
    // When called for an unwrapped child (e.g. ButtonGroup children),
    // leave the concrete leaf type (`Stateful<Div>`) so the parent
    // builder's `.child()` receives the right argument.
    if wrap_to_any {
        stmts.push(quote! { ::gpui::IntoElement::into_any_element(__el) });
    } else {
        stmts.push(quote! { __el });
    }

    Ok(quote! {
        {
            #(#stmts)*
        }
    })
}

fn codegen_control_flow(
    element: &AstElement,
    def: ControlFlowDef,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    match def {
        ControlFlowDef::If | ControlFlowDef::ElseIf | ControlFlowDef::Else => {
            codegen_if_branch(element, def, cx, location, source_file)
        }
        ControlFlowDef::For => codegen_for(element, cx, location, source_file),
        ControlFlowDef::Fragment => codegen_fragment(element, cx, location, source_file),
        ControlFlowDef::Template => codegen_template(element, cx, location, source_file),
        ControlFlowDef::Slot => codegen_slot(element, cx),
        ControlFlowDef::Match => codegen_match(element, cx, location, source_file),
        ControlFlowDef::Case => Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "<Case> must appear directly inside a <Match>",
        )
        .at(element.byte_offset)),
        ControlFlowDef::State => codegen_state(element, cx, location, source_file),
        ControlFlowDef::VirtualList => {
            codegen_virtual_list(element, cx, location, source_file)
        }
        ControlFlowDef::UniformVirtualList => {
            codegen_uniform_virtual_list(element, cx, location, source_file)
        }
        ControlFlowDef::Include => Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            "internal error: <Include> should have been expanded before codegen",
        )),
    }
}

fn codegen_if_branch(
    element: &AstElement,
    kind: ControlFlowDef,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
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
    let child_expr = codegen_children_as_element(&element.children, cx, location, source_file)?;

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

fn codegen_for(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
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
    let child_expr = codegen_children_as_element(&element.children, cx, location, source_file)?;

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
fn emit_for_loop(
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

fn codegen_fragment(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    // A Fragment is just a transparent group of children.
    // When it has multiple children we wrap them in a plain
    // `gpui::div()` so the result is always a single
    // `impl IntoElement`.
    codegen_children_as_element(&element.children, cx, location, source_file)
}

/// Try to resolve a relative `path` against the
/// `source_file` (the path of the `.rs` file that
/// invoked the enclosing `xml!` macro). Absolute paths
/// pass through; relative paths are joined to the
/// source file's parent directory.
///
/// When `source_file` is `None` (the runtime loader /
/// unit-test path), we fall back to the current
/// working directory — this preserves the behaviour
/// tests rely on.
fn resolve_include_path(
    path: &str,
    source_file: Option<&str>,
) -> Result<std::path::PathBuf, XmlError> {
    use std::path::Path;
    let p = Path::new(path);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }
    match source_file {
        Some(src) => {
            let source = Path::new(src);
            // `proc_macro::Span::file()` returns a
            // forward-slash path on every platform
            // (proc-macros run on a host-agnostic layer),
            // but be defensive and strip any leading
            // junk that some toolchains prepend.
            let dir = source
                .parent()
                .filter(|d| !d.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            Ok(dir.join(path))
        }
        None => Ok(Path::new(".").join(path)),
    }
}

/// Read and parse the XML file referenced by an `<Include>` element.
///
/// Errors are reported against the included file itself and wrapped
/// in a pre-rendered diagnostic so the caller's [`LocationTracker`]
/// does not accidentally map offsets into the parent XML.
fn parse_include(
    element: &AstElement,
    source_file: Option<&str>,
) -> Result<(std::path::PathBuf, AstElement), XmlError> {
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
    if src_attr.expr.is_some() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            src_attr.span,
            "<Include src> requires a string literal, not a brace expression",
        )
        .at(src_attr.byte_offset));
    }
    let path = src_attr.raw.as_str();
    let resolved = resolve_include_path(path, source_file)?;
    let contents = std::fs::read_to_string(&resolved).map_err(|e| {
        XmlError::new(
            XmlErrorKind::ParseError,
            element.span,
            format!(
                "could not read `{}` (resolved to `{}`): {e}",
                path,
                resolved.display()
            ),
        )
    })?;

    let line_starts = crate::parser::line_starts(&contents);
    let included_location = crate::parser::LocationTracker {
        line_starts: &line_starts,
        xml: &contents,
        outer_span: element.span,
    };
    match crate::parser::parse(&contents, element.span, &included_location) {
        Ok(root) => Ok((resolved, root)),
        Err(err) => {
            let diagnostic = err.render_with(Some(&included_location));
            let rendered = format!(
                "{diagnostic}\n  = note: in included file `{}`",
                resolved.display()
            );
            Err(XmlError::new(
                XmlErrorKind::ParseError,
                element.span,
                format!("in included file `{}`", resolved.display()),
            )
            .rendered(rendered))
        }
    }
}

/// Recursively expand every `<Include src="…">` in the AST so
/// templates and other compile-time constructs can be shared
/// across XML files. Detects include cycles.
///
/// By default the included file's root element is preserved as a
/// child of the `<Include>`'s parent — this lets a section file
/// declare its own container (e.g. `<Column gap="4">`). If the
/// included root is a `<Fragment>`, its children are spliced in
/// place so shared layout files and template libraries stay
/// transparent.
///
/// Every resolved include path is pushed to `included_paths` so
/// the proc-macro can register the file with Cargo via
/// `include_str!`; otherwise Cargo has no way to know that
/// editing an included XML file should recompile the crate.
fn expand_includes(
    element: &mut AstElement,
    source_file: Option<&str>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
    included_paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), XmlError> {
    let mut i = 0;
    while i < element.children.len() {
        let include_info = if let AstNode::Element(child) = &element.children[i] {
            if child.tag == "Include" {
                let span = child.span;
                Some((span, parse_include(child, source_file)?))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((span, (path, mut included_root))) = include_info {
            if !visited.insert(path.clone()) {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    span,
                    format!(
                        "cyclic <Include> detected: `{}` is already being included",
                        path.display()
                    ),
                ));
            }
            included_paths.push(path.clone());
            // Recurse with the *original* `source_file` (the
            // macro call site), NOT the included file's path.
            // This keeps `<Include src>` resolution anchored to
            // the same base directory at every nesting level —
            // matching the `xml_file!` convention where paths
            // like `ui/shared/x.xml` resolve the same way no
            // matter how deeply the file is nested via Include.
            // (Cycle detection still uses each file's absolute
            // `path` via `visited`, so re-parenting the source
            // base does not weaken the cycle guard.)
            expand_includes(&mut included_root, source_file, visited, included_paths)?;
            visited.remove(&path);
            let nodes = if included_root.tag == "Fragment" {
                included_root.children
            } else {
                vec![AstNode::Element(included_root)]
            };
            element.children.splice(i..i + 1, nodes);
            // Re-process the newly spliced children from this index.
            continue;
        }

        if let AstNode::Element(child) = &mut element.children[i] {
            expand_includes(child, source_file, visited, included_paths)?;
        }
        i += 1;
    }
    Ok(())
}

/// `<Template name="X">…</Template>` for the MVP
/// simply emits its children in place (the template
/// "name" attribute is reserved for future
/// cross-references). Slots are no-ops.
fn codegen_template(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    codegen_fragment(element, cx, location, source_file)
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
/// The arm body supports multiple children; they are
/// automatically wrapped in a plain `gpui::div()`.
fn codegen_match(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
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
        if arm.children.is_empty() {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                arm.span,
                "<Case> must contain at least one child",
            )
            .at(arm.byte_offset));
        }
        let body = codegen_children_as_element(&arm.children, cx, location, source_file)?;
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

/// Resolve the `let:item` / `let:index={name}` bindings on an
/// element, mirroring `<For>`'s convention. The parser turns
/// `let:item` into a value-less `let_item` attribute and
/// `let:index={i}` into a `let_index` attribute whose `raw`
/// value is the requested identifier (the normaliser may
/// append `="true"`, so a `true`/empty value falls back to
/// the default `i`).
///
/// Returns `(item_ident, index_ident)`. `item_ident` is `None`
/// when no `let:item` is present (the virtual-list row body
/// rarely needs it, since the visible index is what gpui
/// hands us — but we support it for symmetry with `<For>`).
fn parse_let_bindings(
    element: &AstElement,
) -> (Option<proc_macro2::Ident>, proc_macro2::Ident) {
    // Resolve a `let:`-derived attribute's identifier. Brace
    // expressions (`let:index={i}`) carry the name in `expr`
    // (the de-braced body); value-less or string forms fall to
    // `raw`, which the normaliser may have set to `"true"` /
    // empty — those cases yield the default name.
    fn resolve_name(
        attr: Option<&AstAttribute>,
        default: &str,
    ) -> Option<String> {
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
        "item",
    )
    .map(|s| format_ident!("{}", s));
    let index_ident = resolve_name(
        element.attributes.iter().find(|a| a.name == "let_index"),
        "index",
    )
    .unwrap_or_else(|| "index".to_string());
    let index_ident = format_ident!("{}", index_ident);
    (item_ident, index_ident)
}

/// Extract a required attribute by name and parse its brace
/// expression. Errors are reported against the missing-attr /
/// non-brace cases so the user gets a clear message.
fn require_attr_expr<'a>(
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

/// `<VirtualList id="…" item_count={n} let:index={i}>…children…</VirtualList>`
/// — a heterogeneous-height gpui virtual list whose controller
/// is auto-persisted via `window.use_keyed_state(id, …)`.
///
/// The children become the row-template body: gpui's `list()`
/// re-invokes `FnMut(usize, &mut Window, &mut App) -> AnyElement`
/// per visible index, so off-screen rows are never built. This
/// is the difference from `<For>`, which eagerly builds every
/// row up front.
///
/// Because the controller lives in element-state keyed by the
/// element id, the host view never needs to carry a
/// `VirtualListController` field — declaring the tag is enough.
/// (Caveat: element state only persists while the element is
/// rendered in consecutive frames; a list unmounted via `<If>`
/// and remounted starts fresh.)
///
/// Emitted shape (conceptual):
/// ```text
/// {
///     let __ix: usize;
///     let __ctl = window.use_keyed_state(id, cx, |_, _|
///         VirtualListController::new(item_count, align, overdraw));
///     __ctl.update(cx, |c, _| { if c.state().item_count() != item_count { c.reset(item_count); } });
///     let __snap = __ctl.read(cx).clone();
///     virtual_list(id, &__snap, cx)
///         .row(move |i, window, cx| { let index = i; <children> })
///         .on_visible_range_change(...)  // if present
///         .render(cx)
///         .into_any_element()
/// }
/// ```
fn codegen_virtual_list(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    codegen_virtual_list_kind(element, cx, location, source_file, VirtualListKind::Heterogeneous)
}

/// `<UniformVirtualList …>` — the equal-height variant. Same
/// row-template mechanism, but the underlying `gpui::uniform_list`
/// measures only the first row and lays the rest in a straight
/// line. It has no `on_visible_range_change` (gpui's
/// `UniformList` has no scroll handler). Otherwise identical
/// to `<VirtualList>`, including the auto-persisted controller.
fn codegen_uniform_virtual_list(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    codegen_virtual_list_kind(
        element,
        cx,
        location,
        source_file,
        VirtualListKind::Uniform,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VirtualListKind {
    Heterogeneous,
    Uniform,
}

/// Shared body for `<VirtualList>` and `<UniformVirtualList>`.
/// The `kind` selects the headless factory (`virtual_list` vs
/// `uniform_virtual_list`), the controller type
/// (`VirtualListController` vs `UniformVirtualListController`),
/// and whether `on_visible_range_change` is accepted.
///
/// Two row modes are supported:
/// - **Explicit** (`item_count={n} let:index={i}`): the children
///   are a row *template*, re-invoked per visible index. The
///   row body references the bound `index` / `item` idents.
/// - **Children-as-rows** (no `item_count`): each direct child
///   *is* a row, and `item_count` is the number of children.
///   The codegen emits a `match ix { 0 => {child0}, … }` so
///   off-screen rows are never built. This is the mode that
///   composes with `<Include>` — e.g. a list of section files
///   becomes a virtualized section scroller with no Rust glue.
fn codegen_virtual_list_kind(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
    kind: VirtualListKind,
) -> Result<TokenStream, XmlError> {
    // Attributes that belong to the virtual-list itself; every
    // other attribute is treated as a style pass-through applied
    // to the rendered element (e.g. `size_full`, `w`, `h`).
    const VL_ATTRS: &[&str] = &[
        "id",
        "item_count",
        "overdraw",
        "alignment",
        "on_visible_range_change",
        "let_item",
        "let_index",
        "controller",
        "row",
    ];

    // --- required attributes ---------------------------------------
    let id_attr = require_attr_expr(element, "id")?;
    // `id` may be a literal (`id="foo"`) or a brace expr. We
    // support both: a literal becomes `("foo").to_string()` so
    // it coerces to `Into<ElementId>`; a brace expr passes
    // through verbatim.
    let id_expr = match &id_attr.expr {
        Some(e) => parse_ts(e, id_attr.span, id_attr.byte_offset, "<VirtualList> id")?,
        None => {
            let raw = &id_attr.raw;
            quote! { (#raw).to_string() }
        }
    };

    // --- detect row mode -------------------------------------------
    // Three modes:
    //   1. Explicit `row={closure}` + `item_count` — the caller
    //      supplies a ready-made row closure (useful when the row
    //      body needs controller state that must not be captured
    //      from the surrounding scope).
    //   2. `item_count` + children — the children become the row
    //      template, re-invoked per visible index.
    //   3. Children only — each direct child is one row.
    let count_attr = element.attributes.iter().find(|a| a.name == "item_count");
    let row_attr = element.attributes.iter().find(|a| a.name == "row");
    let (count_expr, row_closure, _item_bind, _index_ident) = match (row_attr, count_attr) {
        (Some(row_attr), Some(count_attr)) => {
            let count_expr = attr_expr_only(count_attr)?;
            let row_expr = attr_expr_only(row_attr)?;
            (count_expr, quote! { #row_expr }, quote! {}, format_ident!("index"))
        }
        (Some(_), None) => {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                element.span,
                format!(
                    "<{}> `row={{...}}` requires an `item_count={{...}}` attribute",
                    element.tag
                ),
            )
            .at(element.byte_offset));
        }
        (None, Some(count_attr)) => {
            // Explicit mode: row template re-invoked per index.
            let count_expr = attr_expr_only(count_attr)?;
            if element.children.is_empty() {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    element.span,
                    format!(
                        "<{}> with `item_count` must contain at least one child (the row template)",
                        element.tag
                    ),
                )
                .at(element.byte_offset));
            }
            let (item_ident, index_ident) = parse_let_bindings(element);
            let child_body =
                codegen_children_as_element(&element.children, cx, location, source_file)?;
            // Bind `let item = index;` when `let:item` is requested —
            // gpui hands the row closure a `usize`, so for parity with
            // `<For>` we make the item binding an alias of the index.
            let item_bind = item_ident
                .map(|it| quote! { let #it: usize = #index_ident; })
                .unwrap_or_default();
            let row_closure = quote! {
                move |#index_ident: usize, window: &mut ::gpui::Window, cx: &mut ::gpui::App| -> ::gpui::AnyElement {
                    #item_bind
                    ::gpui::IntoElement::into_any_element(#child_body)
                }
            };
            (count_expr, row_closure, quote! {}, index_ident)
        }
        (None, None) => {
            // Children-as-rows mode: each direct child is one row.
            if element.children.is_empty() {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    element.span,
                    format!(
                        "<{}> requires either an `item_count={{...}}` attribute or at least one child row",
                        element.tag
                    ),
                )
                .at(element.byte_offset));
            }
            // One row per child. The index ident is internal to
            // the generated match; the user's children don't see it.
            let arms: Vec<TokenStream> = element
                .children
                .iter()
                .enumerate()
                .map(|(i, child)| {
                    let body =
                        codegen_child(child, cx, location, source_file)?;
                    let lit = i;
                    Ok::<_, XmlError>(quote! { #lit => { #body } })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let n = element.children.len();
            let count_expr = quote! { #n };
            let index_ident = format_ident!("__ix");
            let child_body = quote! {
                match #index_ident {
                    #(#arms)*
                    _ => ::gpui::div(),
                }
            };
            let row_closure = quote! {
                move |#index_ident: usize, window: &mut ::gpui::Window, cx: &mut ::gpui::App| -> ::gpui::AnyElement {
                    ::gpui::IntoElement::into_any_element(#child_body)
                }
            };
            (count_expr, row_closure, quote! {}, index_ident)
        }
    };

    // --- optional attributes ---------------------------------------
    let overdraw_expr = match element.attributes.iter().find(|a| a.name == "overdraw") {
        Some(a) => attr_expr_only(a)?,
        None => quote! { ::gpui::px(16.) },
    };
    // `alignment="top" | "bottom"` (heterogeneous only);
    // uniform lists have no alignment prop on the headless
    // factory, so we ignore it for that kind.
    let alignment_tokens = if kind == VirtualListKind::Heterogeneous {
        match element.attributes.iter().find(|a| a.name == "alignment") {
            Some(a) => {
                let raw = a.raw.as_str();
                let variant = match raw {
                    "bottom" => quote! { ::gpui::ListAlignment::Bottom },
                    "top" | "" => quote! { ::gpui::ListAlignment::Top },
                    _ => {
                        return Err(XmlError::new(
                            XmlErrorKind::Unsupported,
                            a.span,
                            format!(
                                "<{}> alignment must be \"top\" or \"bottom\", got `{raw}`",
                                element.tag
                            ),
                        )
                        .at(a.byte_offset));
                    }
                };
                Some(variant)
            }
            None => Some(quote! { ::gpui::ListAlignment::Top }),
        }
    } else {
        None
    };

    let on_range_tokens = if kind == VirtualListKind::Heterogeneous {
        match element
            .attributes
            .iter()
            .find(|a| a.name == "on_visible_range_change")
        {
            Some(a) => {
                let expr = attr_expr_only(a)?;
                Some(auto_wrap_event_expr(a, expr, "on_visible_range_change", "VirtualList"))
            }
            None => None,
        }
    } else {
        None
    };

    let controller_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "controller");
    let controller_expr = match controller_attr {
        Some(a) => Some(attr_expr_only(a)?),
        None => None,
    };

    // --- style pass-through ----------------------------------------
    // Any attribute that isn't a virtual-list attribute is
    // applied to the rendered element (e.g. `size_full`, `h`,
    // `w`, `flex_grow`). This is the same vocabulary the
    // container codegen accepts, so we reuse its dispatcher.
    let style_container_def = crate::schema::ContainerDef {
        fixed_methods: &[],
        style_hint: "the gpui Styled trait (`.size_full()`, `.h(...)`, …)",
    };
    let mut style_stmts: Vec<TokenStream> = Vec::new();
    for attr in &element.attributes {
        if VL_ATTRS.contains(&attr.name.as_str()) {
            continue;
        }
        apply_container_attr(&mut style_stmts, attr, style_container_def, element)?;
    }

    // --- factory / controller paths --------------------------------
    let (factory, controller_ty, entity_state_init) = match kind {
        VirtualListKind::Heterogeneous => {
            let init = quote! {
                ::yororen_ui::headless::virtual_list::VirtualListController::new(
                    #count_expr as usize,
                    #alignment_tokens,
                    #overdraw_expr,
                )
            };
            (
                quote! { ::yororen_ui::headless::virtual_list::virtual_list },
                quote! { ::yororen_ui::headless::virtual_list::VirtualListController },
                init,
            )
        }
        VirtualListKind::Uniform => {
            // The uniform factory takes item_count positionally
            // each frame (it is not stored on the controller),
            // so we pass `count_expr` at the call site below
            // rather than in the init.
            let init = quote! {
                ::yororen_ui::headless::virtual_list::UniformVirtualListController::new()
            };
            (
                quote! { ::yororen_ui::headless::virtual_list::uniform_virtual_list },
                quote! { ::yororen_ui::headless::virtual_list::UniformVirtualListController },
                init,
            )
        }
    };

    // --- emit ------------------------------------------------------
    // Heterogeneous: the factory signature is
    //   virtual_list(id, &controller, cx) -> VirtualListProps,
    // and we sync item_count via controller.reset every frame.
    // Uniform: the factory signature is
    //   uniform_virtual_list(id, item_count, &controller, cx)
    //   -> UniformVirtualListProps,
    // so count goes in at the call site and no reset is needed.
    let (factory_call, count_sync) = match kind {
        VirtualListKind::Heterogeneous => (
            quote! { #factory(#id_expr, &__snap, #cx) },
            quote! {
                // Evaluate the target count *outside* `update` so the
                // closure does not immutably borrow `cx` while the
                // `Entity::update` call already holds a mutable borrow.
                let __target_count = (#count_expr as usize);
                __entity.update(#cx, |__c: &mut #controller_ty, _| {
                    let __current = __c.state().item_count();
                    if __target_count > __current {
                        // Grow via append so the scroll position is
                        // preserved (infinite-loading behaviour).
                        __c.append(__target_count - __current);
                    } else if __target_count < __current {
                        __c.reset(__target_count);
                    }
                });
            },
        ),
        VirtualListKind::Uniform => (
            quote! { #factory(#id_expr, #count_expr as usize, &__snap, #cx) },
            quote! {},
        ),
    };

    let on_range_chain = match on_range_tokens {
        Some(t) => quote! { .on_visible_range_change(#t) },
        None => quote! {},
    };

    // `use_keyed_state` is an inherent method on `gpui::Window`.
    // It persists the controller as an `Entity<T>` keyed by the
    // element id across consecutive frames and auto-observes
    // mutations (so a `.reset()` / `.scroll_to_*` triggers a
    // re-render of the owning view). We clone the controller
    // out of the entity for the per-frame factory call.
    //
    // Alternatively, the caller can supply an external entity via
    // `controller={...}` — useful when buttons outside the list need
    // to drive scroll position or when the controller is shared with
    // business state.
    let entity_init = match &controller_expr {
        Some(expr) => quote! { let __entity = (#expr).clone(); },
        None => quote! {
            let __entity = window.use_keyed_state(
                #id_expr,
                #cx,
                |_window, _cx| #entity_state_init,
            );
        },
    };

    Ok(quote! {
        {
            #entity_init
            #count_sync
            let __snap = __entity.read(#cx).clone();
            let mut __props = #factory_call;
            __props = __props.row(#row_closure);
            __props = __props #on_range_chain;
            let mut __el = __props.render(#cx);
            #(#style_stmts)*
            ::gpui::IntoElement::into_any_element(__el)
        }
    })
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
    source_file: Option<&str>,
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

    let body = codegen_children_as_element(&element.children, cx, location, source_file)?;

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
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    match node {
        AstNode::Element(e) => codegen_element(e, cx, location, source_file),
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
fn codegen_children_as_element(
    children: &[AstNode],
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    if children.is_empty() {
        Ok(quote! { gpui::div() })
    } else if children.len() == 1 {
        codegen_child(&children[0], cx, location, source_file)
    } else {
        let mut stmts: Vec<TokenStream> = Vec::new();
        stmts.push(quote! { let __el = gpui::div(); });
        for child in children {
            let child_expr = codegen_child(child, cx, location, source_file)?;
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
fn codegen_child_unwrapped(
    node: &AstNode,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
) -> Result<TokenStream, XmlError> {
    match node {
        AstNode::Element(e) => {
            let def = lookup_with_user_schema(&e.tag).unwrap_or(RUNTIME_LEAF_FALLBACK.clone());
            match def.kind {
                ComponentKind::Leaf(l) => codegen_leaf(e, l, cx, location, source_file, false),
                _ => codegen_element(e, cx, location, source_file),
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
            PropValue::String
            | PropValue::Variant
            | PropValue::BadgeVariant
            | PropValue::HeadingLevel
            | PropValue::IconSource
            | PropValue::ImageSource
            | PropValue::KeybindingInputMode
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
        PropValue::BadgeVariant => Ok(match raw {
            "neutral" => quote! { ::yororen_ui::headless::badge::BadgeVariant::Neutral },
            "success" => quote! { ::yororen_ui::headless::badge::BadgeVariant::Success },
            "warning" => quote! { ::yororen_ui::headless::badge::BadgeVariant::Warning },
            "danger" => quote! { ::yororen_ui::headless::badge::BadgeVariant::Danger },
            "info" => quote! { ::yororen_ui::headless::badge::BadgeVariant::Info },
            other => {
                return Err(XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    format!(
                        "attribute `{}` expects one of `neutral`, `success`, `warning`, `danger`, `info`, got `{other}`",
                        attr.name
                    ),
                )
                .at(attr.byte_offset));
            }
        }),
        PropValue::HeadingLevel => {
            let variant = match raw {
                "H1" | "h1" => "H1",
                "H2" | "h2" => "H2",
                "H3" | "h3" => "H3",
                "H4" | "h4" => "H4",
                "H5" | "h5" => "H5",
                "H6" | "h6" => "H6",
                other => {
                    return Err(XmlError::new(
                        XmlErrorKind::InvalidExpression,
                        attr.span,
                        format!("attribute `{}` expects H1..H6, got `{other}`", attr.name),
                    )
                    .at(attr.byte_offset));
                }
            };
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
            let variant = match raw {
                "Idle" | "idle" => "Idle",
                "Capturing" | "capturing" => "Capturing",
                other => {
                    return Err(XmlError::new(
                        XmlErrorKind::InvalidExpression,
                        attr.span,
                        format!(
                            "attribute `{}` expects Idle or Capturing, got `{other}`",
                            attr.name
                        ),
                    )
                    .at(attr.byte_offset));
                }
            };
            let variant = format_ident!("{variant}");
            Ok(quote! { ::yororen_ui::headless::keybinding_input::KeybindingInputMode::#variant })
        }
        PropValue::Color => {
            // Brace expressions are passed through verbatim above;
            // this arm only handles literal strings. Hex colours are
            // converted to `gpui::rgb` / `gpui::rgba` calls; anything
            // else is rejected with a helpful note.
            parse_hex_color(raw, attr)
        }
        PropValue::Unknown => Ok(quote! { (#raw).to_string() }),
        PropValue::Custom => {
            if attr.expr.is_some() {
                unreachable!("brace expressions are handled at the top of prop_value_tokens")
            } else {
                Ok(quote! { (#raw).into() })
            }
        }
        PropValue::Float64 => {
            let value = raw.parse::<f64>().map_err(|_| {
                XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    format!(
                        "attribute `{}` expects an f64 literal, got `{raw}`",
                        attr.name
                    ),
                )
                .at(attr.byte_offset)
            })?;
            let lit = proc_macro2::Literal::f64_suffixed(value);
            Ok(quote! { #lit })
        }
        PropValue::Float32 => {
            let value = raw.parse::<f32>().map_err(|_| {
                XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    format!(
                        "attribute `{}` expects an f32 literal, got `{raw}`",
                        attr.name
                    ),
                )
                .at(attr.byte_offset)
            })?;
            let lit = proc_macro2::Literal::f32_suffixed(value);
            Ok(quote! { #lit })
        }
        PropValue::UInt => {
            let value = raw.parse::<usize>().map_err(|_| {
                XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    format!(
                        "attribute `{}` expects a usize literal, got `{raw}`",
                        attr.name
                    ),
                )
                .at(attr.byte_offset)
            })?;
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
/// appends both calls to the props builder.
fn emit_bind(entity: &TokenStream, def: LeafDef, cx: &TokenStream) -> Vec<TokenStream> {
    // Pick the value prop. Prefer `value` (TextInput,
    // SearchInput, NumberInput, …); fall back to
    // `checked` (Checkbox, Switch, ToggleButton); then
    // `text` (Label-like). If none of these exist, the
    // read side is skipped — the entity's current value
    // is read on each render anyway.
    let value_prop = def
        .props
        .iter()
        .find(|p| p.name == "value")
        .or_else(|| def.props.iter().find(|p| p.name == "checked"))
        .or_else(|| def.props.iter().find(|p| p.name == "selected"))
        .or_else(|| def.props.iter().find(|p| p.name == "text"));
    // Pick the change event. Prefer `on_change`; fall
    // back to `on_toggle` for boolean-style components.
    let change_event = def
        .events
        .iter()
        .find(|(n, _)| *n == "on_change")
        .or_else(|| def.events.iter().find(|(n, _)| *n == "on_toggle"));

    let mut out: Vec<TokenStream> = Vec::new();
    if let Some(prop) = value_prop {
        let m = format_ident!("{}", prop.setter);
        // Read the current value via the `XmlBinding` trait
        // — the blanket `impl<T: Clone> XmlBinding<T> for
        // Entity<T>` handles the common case, and user
        // impls route through the same call site. We clone
        // the entity so the original binding in the user's
        // scope isn't moved.
        out.push(quote! {
            __el = __el.#m({
                let __bind = (#entity).clone();
                ::yororen_ui::headless::XmlBinding::xml_read(&__bind, #cx)
            });
        });
    }
    if let Some((event_attr, setter)) = change_event {
        let m = format_ident!("{}", setter);
        // Pick the closure signature based on the event
        // name. on_change takes `(&str, &mut Window,
        // &mut App)` for text inputs and `(f64, &mut Window,
        // &mut App)` for number inputs; on_toggle takes
        // `(bool, Option<&ClickEvent>, &mut Window,
        // &mut App)`. We use the value setter's type
        // (Float → f64, anything else → String) to pick
        // the right `XmlBinding<T>` instantiation.
        let event_name = *event_attr;
        let value_is_f32 = matches!(value_prop.map(|p| p.value), Some(PropValue::Float32));
        let value_is_f64 = matches!(value_prop.map(|p| p.value), Some(PropValue::Float64));
        let value_is_usize = matches!(value_prop.map(|p| p.value), Some(PropValue::UInt));
        let writeback = if event_name == "on_toggle" {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: bool, _ev: Option<&gpui::ClickEvent>, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<bool>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else if value_is_f64 {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: f64, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<f64>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else if value_is_f32 {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: f32, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<f32>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else if value_is_usize {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: usize, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<usize>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: &str, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        let __new: String = __v.to_string();
                        ::yororen_ui::headless::XmlBinding::<String>::xml_write(&__bind, __new, cx);
                    }
                });
            }
        };
        out.push(writeback);
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
/// **4-arg events**: `on_toggle` on Checkbox / Switch /
/// Radio uses `(bool, Option<&ClickEvent>, &mut Window,
/// &mut App)`. When `event_name` is `on_toggle`, the
/// wrapper emits a 4-arg closure.
fn auto_wrap_event_expr(
    attr: &AstAttribute,
    expr: TokenStream,
    event_name: &str,
    tag: &str,
) -> TokenStream {
    // Closure shape depends on the event and component tag.
    let (params, call_args): (TokenStream, TokenStream) = match event_name {
        "on_toggle" if matches!(tag, "Checkbox" | "Switch" | "Radio" | "ToggleButton") => (
            quote! { __arg0: bool, __ev: Option<&gpui::ClickEvent>, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __arg0, __ev, __w, __cx },
        ),
        "on_toggle" => (
            quote! { __ev: &gpui::ClickEvent, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __ev, __w, __cx },
        ),
        "on_clear" | "on_start_capture" | "on_cancel_capture" => (
            quote! { __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __w, __cx },
        ),
        "on_visible_range_change" => (
            quote! { __range: std::ops::Range<usize>, __total: usize, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __range, __total, __w, __cx },
        ),
        _ => (
            quote! { __arg0, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __arg0, __w, __cx },
        ),
    };
    auto_wrap_closure_expr(attr, expr, params, call_args)
}

/// Auto-wrap a bare callback expression (used for
/// closure-type positional factory arguments such as
/// `SplitButton`'s `primary`).
fn auto_wrap_callback_expr(attr: &AstAttribute, expr: TokenStream) -> TokenStream {
    let params = quote! { __arg0, __w: &mut gpui::Window, __cx: &mut gpui::App };
    let call_args = quote! { __arg0, __w, __cx };
    auto_wrap_closure_expr(attr, expr, params, call_args)
}

/// Shared implementation for auto-wrapping a bare path / field
/// reference into a `move` closure, pre-cloning the receiver
/// outside the closure so multiple handlers can share the same
/// controller binding.
fn auto_wrap_closure_expr(
    attr: &AstAttribute,
    expr: TokenStream,
    params: TokenStream,
    call_args: TokenStream,
) -> TokenStream {
    let Some(raw) = &attr.expr else {
        return expr;
    };
    let trimmed = raw.trim();
    // Decide whether to auto-wrap. We *never* auto-wrap
    // user-supplied closures (they have `{` or `|`),
    // and we *always* auto-wrap bare path expressions
    // (`controller.foo` with no args). The interesting
    // middle case is a call expression like
    // `controller.goto(Section::Actions)` — the user is
    // calling a method whose RETURN VALUE is the event
    // handler. That's a "factory" call (the controller
    // method produces the closure to wire up). The
    // auto-wrap should NOT fire here either; we just
    // pass the call result through and the compiler
    // checks that the result is the right closure type.
    //
    // Concretely: the only expressions we auto-wrap are
    // those that syntactically look like a path / field
    // reference with NO call or closure body. Anything
    // containing `(` / `{` / `|` is the user's code and
    // we pass it through verbatim.
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
            move |#params| {
                #expr(#call_args)
            }
        };
    };
    match parsed {
        // Associated function (`Module::function`) —
        // no receiver, no clone needed.
        syn::Expr::Path(_) => quote! {
            move |#params| {
                #expr(#call_args)
            }
        },
        // `controller.method(args)` — a method call whose
        // result is itself the event handler (a closure
        // factory: `goto(Section::Actions) -> impl Fn(...)`).
        // Pass the call result through verbatim; the
        // receiver is cloned inline so the closure can
        // move it. We don't auto-wrap into a 3-arg
        // closure because the call has its own argument
        // list and the resulting value IS already a
        // closure.
        syn::Expr::Call(call) => {
            let func = call.func;
            // The function being called: clone its
            // receiver once, so the inline call can use
            // the owned value.
            match &*func {
                syn::Expr::Field(field) => {
                    let receiver = &field.base;
                    let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
                    let member = &field.member;
                    let args = call.args.iter();
                    quote! {
                        {
                            let #clone_ident = (#receiver).clone();
                            #clone_ident.#member(#(#args),*)
                        }
                    }
                }
                _ => {
                    // Path-style call (`my_func(args)`).
                    // Pass the result through directly.
                    quote! { #expr }
                }
            }
        }
        // `controller.method` — bare field access. Wrap
        // into the right closure shape for the event.
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
                    move |#params| {
                        #clone_ident.#member(#call_args)
                    }
                }
            }
        }
        // Method call, deref, closure literal, etc. —
        // the user wrote their own expression; pass it
        // through verbatim. The compiler will reject it
        // if the type doesn't match the setter's bound.
        _ => quote! { #expr },
    }
}

/// Like [`auto_wrap_event_expr`], but for use with event
/// modifiers. Instead of returning a complete closure, it
/// returns:
///
/// 1. An optional statement that pre-clones the receiver
///    (e.g. `let __auto_clone = (controller).clone();`).
///    This statement must be placed *outside* the final
///    `move` closure so the closure only captures the clone.
/// 2. A call expression that invokes the user's handler
///    inside the closure body with the standard event args
///    (`__ev, __window, cx`).
fn auto_wrap_event_call(
    attr: &AstAttribute,
    expr: TokenStream,
) -> (Option<TokenStream>, TokenStream) {
    let Some(raw) = &attr.expr else {
        return (None, quote! { #expr(__ev, __window, cx) });
    };
    let trimmed = raw.trim();
    let looks_like_path = !trimmed.contains('(')
        && !trimmed.contains('{')
        && !trimmed.contains('|')
        && !trimmed.is_empty();
    if !looks_like_path {
        return (None, quote! { #expr(__ev, __window, cx) });
    }
    let Ok(parsed) = syn::parse_str::<syn::Expr>(trimmed) else {
        return (None, quote! { #expr(__ev, __window, cx) });
    };

    let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
    match parsed {
        // `controller.method(args)` — a method call that
        // returns an event handler closure. Clone the
        // receiver, then call the method and immediately
        // invoke the returned closure with the event args.
        syn::Expr::Call(call) => match &*call.func {
            syn::Expr::Field(field) => {
                let receiver = &field.base;
                let member = &field.member;
                let args = call.args.iter();
                let clone = quote! { let #clone_ident = (#receiver).clone(); };
                let call = quote! { #clone_ident.#member(#(#args),*)(__ev, __window, cx) };
                (Some(clone), call)
            }
            _ => (None, quote! { #expr(__ev, __window, cx) }),
        },
        // `controller.method` — bare field access. Clone the
        // receiver, then call the method with the event args.
        syn::Expr::Field(field) => {
            let receiver = field.base;
            let member = field.member;
            let clone = quote! { let #clone_ident = (#receiver).clone(); };
            let call = quote! { #clone_ident.#member(__ev, __window, cx) };
            (Some(clone), call)
        }
        // Associated function or bare path — no receiver.
        _ => (None, quote! { #expr(__ev, __window, cx) }),
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

/// Build the body of a 3-arg event closure for an event
/// with modifiers. `inner_call` is the expression that
/// invokes the user's handler (e.g.
/// `__auto_clone.show_toast(__ev, __window, cx)`). The
/// returned token stream is the body of a closure with
/// signature `|__ev, __window, cx|`, with all modifier
/// checks applied around the inner call.
///
/// Modifiers are applied right-to-left so the leftmost
/// modifier listed in XML becomes the outermost check.
fn wrap_event_body_with_modifiers(
    modifiers: &[&str],
    inner_call: TokenStream,
    span: Span,
) -> Result<TokenStream, XmlError> {
    if modifiers.is_empty() {
        return Ok(inner_call);
    }
    let mut body = inner_call;
    for modifier in modifiers.iter().rev() {
        body = match *modifier {
            // `.stop` — ask the platform not to propagate
            // the event further. gpui's `App::stop_propagation`
            // is a flag the dispatcher reads; calling it here
            // before the user's handler runs is the contract.
            "stop" => quote! {
                { cx.stop_propagation(); #body }
            },
            // `.prevent` — ask the platform to skip the
            // default action for the event.
            "prevent" => quote! {
                { __window.prevent_default(); #body }
            },
            // Modifier-key filters. Each maps to a boolean
            // field on `gpui::Modifiers` — the event arg's
            // `.modifiers()` accessor returns one. `.meta`
            // is accepted as an alias for `.platform` (the
            // macOS Command key) because "cmd" / "meta" is
            // the more familiar name on Windows / Linux.
            "ctrl" => wrap_modifier_flag_body(body, "control"),
            "shift" => wrap_modifier_flag_body(body, "shift"),
            "alt" => wrap_modifier_flag_body(body, "alt"),
            "platform" | "meta" | "cmd" => wrap_modifier_flag_body(body, "platform"),
            "secondary" => wrap_modifier_flag_body(body, "secondary"),
            "function" => wrap_modifier_flag_body(body, "function"),
            // Keyboard filters — gate on the keystroke key.
            // `__ev.keystroke().key` returns the printable
            // name (`"enter"`, `"escape"`, `"tab"`, …) which
            // is exactly what the user writes in the XML.
            key => {
                if !is_known_key_filter(key) {
                    return Err(XmlError::new(
                        XmlErrorKind::InvalidExpression,
                        span,
                        format!(
                            "unknown event modifier `{key}`; expected one of `stop`, `prevent`, `ctrl`, `shift`, `alt`, `platform` (alias `meta`/`cmd`), `secondary`, `function`, or a key name like `enter` / `escape` / `tab`"
                        ),
                    ));
                }
                let key_lit = format!("\"{key}\"");
                quote! {
                    if __ev.keystroke().key == #key_lit {
                        #body
                    }
                }
            }
        };
    }
    Ok(body)
}

/// Wrap a closure body with a single modifier-key gate.
/// Emits `if __ev.modifiers().<flag> { #body }` so the
/// filter only fires when the corresponding `Modifiers`
/// field is set. The flag is spliced as a Rust field-access
/// identifier (`control`, `shift`, `alt`, `platform`,
/// `secondary`, `function`).
fn wrap_modifier_flag_body(body: TokenStream, flag: &str) -> TokenStream {
    let flag_ident = format_ident!("{}", flag);
    quote! {
        if __ev.modifiers().#flag_ident {
            #body
        }
    }
}

/// The set of keyboard key names accepted as `.xxx`
/// modifiers on event attributes. Anything outside this
/// set is rejected so the user gets a clear error
/// instead of a typo silently never firing.
fn is_known_key_filter(name: &str) -> bool {
    matches!(
        name,
        // Whitespace / editing
        "enter"
        | "escape"
        | "tab"
        | "space"
        | "backspace"
        | "delete"
        // Arrow keys
        | "up"
        | "down"
        | "left"
        | "right"
        // Navigation
        | "home"
        | "end"
        | "pageup"
        | "pagedown"
        // Function keys (F1..F12)
        | "f1" | "f2" | "f3" | "f4" | "f5" | "f6"
        | "f7" | "f8" | "f9" | "f10" | "f11" | "f12"
    )
}

/// Parse a hex colour literal (`#rrggbb` or `#rrggbbaa`) and
/// emit the corresponding gpui constructor. Rejects other
/// literal forms and points the user toward a brace expression.
fn parse_hex_color(raw: &str, attr: &AstAttribute) -> Result<TokenStream, XmlError> {
    let hex = raw.strip_prefix('#').ok_or_else(|| {
        XmlError::new(
            XmlErrorKind::InvalidExpression,
            attr.span,
            format!(
                "attribute `{}` expects a hex colour (`#rrggbb` or `#rrggbbaa`) or a brace expression like `{{gpui::hsla(...)}}`; got `{raw}`",
                attr.name
            ),
        )
        .at(attr.byte_offset)
    })?;
    if hex.len() == 6 {
        let value = u32::from_str_radix(hex, 16).map_err(|_| {
            XmlError::new(
                XmlErrorKind::InvalidExpression,
                attr.span,
                format!(
                    "attribute `{}` expects a valid hex colour, got `{raw}`",
                    attr.name
                ),
            )
            .at(attr.byte_offset)
        })?;
        Ok(quote! { ::gpui::rgb(#value) })
    } else if hex.len() == 8 {
        let value = u32::from_str_radix(hex, 16).map_err(|_| {
            XmlError::new(
                XmlErrorKind::InvalidExpression,
                attr.span,
                format!(
                    "attribute `{}` expects a valid hex colour, got `{raw}`",
                    attr.name
                ),
            )
            .at(attr.byte_offset)
        })?;
        Ok(quote! { ::gpui::rgba(#value) })
    } else {
        Err(XmlError::new(
            XmlErrorKind::InvalidExpression,
            attr.span,
            format!(
                "attribute `{}` expects `#rrggbb` or `#rrggbbaa`, got `{raw}`",
                attr.name
            ),
        )
        .at(attr.byte_offset))
    }
}

// --- error-message helpers --------------------------------------------------

/// Compute a simple string-similarity score: number of
/// character insertions / deletions / substitutions needed
/// to turn `a` into `b` (Damerau-Levenshtein distance,
/// capped to avoid quadratic blow-up on long names).
fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let n = a.len();
    let m = b.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                curr[j] = curr[j].min(prev[j - 2] + cost);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Whether an attribute on a leaf should be treated as a
/// gpui `Styled` pass-through instead of a component prop.
/// We only allow the same vocabulary that containers accept,
/// and we skip attributes that are already part of the
/// component's schema (e.g. `w`/`h` on `Skeleton`).
fn is_leaf_style_attr(name: &str, def: &LeafDef) -> bool {
    let is_style = is_known_shorthand_method(name)
        || is_spacing_prefix(name)
        || is_spacing_shorthand(name);
    if !is_style {
        return false;
    }
    if name == "id" {
        return false;
    }
    if def.extra_args.iter().any(|e| e.attr == name) {
        return false;
    }
    if def.props.iter().any(|p| p.name == name) {
        return false;
    }
    if def.events.iter().any(|e| e.0 == name) {
        return false;
    }
    true
}

/// Pick the candidate most likely to be what the user
/// meant. Returns `None` when nothing is close enough.
fn did_you_mean<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let name = name.to_ascii_lowercase();
    let mut best: Option<(&'a str, usize)> = None;
    for c in candidates {
        // Exact prefix match beats edit distance.
        let lower = c.to_ascii_lowercase();
        let dist = if lower.starts_with(&name) || name.starts_with(&lower) {
            1
        } else {
            edit_distance(&name, &lower)
        };
        let threshold = (name.len() / 2).max(2);
        if dist <= threshold && best.map_or(true, |(_, d)| dist < d) {
            best = Some((*c, dist));
        }
    }
    best.map(|(c, _)| c)
}

/// Build a human-readable list of accepted XML attributes
/// for a leaf component (id + extra args + props + events + slots).
fn accepted_leaf_attrs(def: &LeafDef) -> String {
    let mut parts = vec!["id".to_string()];
    for e in def.extra_args {
        parts.push(e.attr.to_string());
    }
    for p in def.props {
        parts.push(p.name.to_string());
    }
    for (name, _) in def.events {
        parts.push(name.to_string());
    }
    for s in def.slots {
        parts.push(format!("slot=\"{}\"", s.name));
    }
    parts.join(", ")
}

/// Build a human-readable list of accepted XML attributes
/// for a container (id + shorthand style methods).
fn accepted_container_attrs(def: &ContainerDef) -> String {
    let mut parts = vec!["id".to_string()];
    parts.push("flex".to_string());
    for (attr, _) in def.fixed_methods {
        parts.push(attr.to_string());
    }
    parts.push("shorthand style methods (gap_3, p_4, w_full, ...)".to_string());
    parts.join(", ")
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
    use crate::schema_generated::{BUILTINS_GENERATED, BUILTINS_OVERRIDES};
    use proc_macro2::Span;

    fn render(xml: &str) -> String {
        let ts = codegen(xml, Span::call_site(), None, None, &[]).expect("codegen succeeds");
        ts.to_string()
    }

    #[test]
    fn empty_column() {
        let s = render(r#"<Column col />"#);
        // Must start with `gpui::div()` and contain `flex_col`.
        assert!(s.contains("gpui :: div ()"), "{s}");
        assert!(s.contains("flex_col"), "{s}");
    }

    #[test]
    fn column_with_gap_and_padding() {
        let s = render(r#"<Column flex col gap="3" p="4" />"#);
        assert!(s.contains("flex"), "{s}");
        assert!(s.contains("flex_col"), "{s}");
        assert!(s.contains("gap_3"), "{s}");
        assert!(s.contains("p_4"), "{s}");
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
    fn unknown_tag_falls_through_to_runtime_registry() {
        // Unknown tags used to be a hard error; with the
        // runtime registry (`register_xml_component!`)
        // they now compile and resolve at runtime via
        // `runtime::render_or_empty`. The codegen must
        // emit a call into the runtime module rather
        // than erroring.
        let ts = codegen(r#"<MyWidget id="x" />"#, Span::call_site(), None, None, &[])
            .expect("codegen ok");
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
        let err =
            codegen(r#"<Label text="hi" />"#, Span::call_site(), None, None, &[]).unwrap_err();
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
        // event, but the body generator should still
        // produce the right shape around an inner call.
        let inner_call: TokenStream =
            syn::parse_str("__inner(__ev, __window, cx)").expect("parse inner call");
        let body = wrap_event_body_with_modifiers(&["enter"], inner_call, Span::call_site())
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
            wrap_event_body_with_modifiers(&["ctrl", "enter"], inner_call, Span::call_site())
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
        let body = wrap_event_body_with_modifiers(&["stop"], inner_call, Span::call_site())
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
        let body = wrap_event_body_with_modifiers(&["prevent"], inner_call, Span::call_site())
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
        let body = wrap_event_body_with_modifiers(&["shift"], inner_call, Span::call_site())
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
            let body =
                wrap_event_body_with_modifiers(&[mod_name], inner_call.clone(), Span::call_site())
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
        let err = wrap_event_body_with_modifiers(&["stpo"], inner_call, Span::call_site())
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
        let xml = r#"<Button id="x" caption="x" on_click={move |ev, w, cx| controller.handle(ev, w, cx)} />"#;
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
        // surrounding Container's child chain is preserved.
        let s = render(r#"<Column><Slot/></Column>"#);
        assert!(s.contains("gpui :: div ()"), "{s}");
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
        let resolved = resolve_include_path("ui/header.xml", Some("/home/dev/proj/src/view.rs"))
            .expect("resolve");
        assert_eq!(resolved, PathBuf::from("/home/dev/proj/src/ui/header.xml"));
    }

    #[test]
    fn include_passes_absolute_paths_through() {
        // Absolute `src` paths skip the join and are
        // used verbatim.
        use std::path::PathBuf;
        let resolved = resolve_include_path("/etc/foo.xml", Some("/home/dev/proj/src/view.rs"))
            .expect("resolve");
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
        let err = codegen(r#"<Column flx col />"#, Span::call_site(), None, None, &[]).unwrap_err();
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
}
