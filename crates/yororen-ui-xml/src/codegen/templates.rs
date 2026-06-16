use proc_macro2::{Span, TokenStream};

use crate::ast::{AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::ComponentDef;

use crate::codegen::control_flow::codegen_fragment;

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
pub(crate) fn collect_templates(
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
pub(crate) fn walk_for_templates(
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
pub(crate) fn expand_template_invocations(
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
pub(crate) fn instantiate_template(
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
pub(crate) fn substitute_slots(
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

/// `<Template name="X">…</Template>` for the MVP
/// simply emits its children in place (the template
/// "name" attribute is reserved for future
/// cross-references). Slots are no-ops.
pub(crate) fn codegen_template(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    codegen_fragment(element, cx, location, source_file, user_schema)
}

/// `<Slot/>` is a no-op for the MVP. Future revisions
/// will wire caller-side slot-filling.
pub(crate) fn codegen_slot(
    _element: &AstElement,
    _cx: &TokenStream,
) -> Result<TokenStream, XmlError> {
    Ok(TokenStream::new())
}
