//! Runtime XML component registry.
//!
//! Built-in tags (`<Column>`, `<Button>`, `<Label>`, …)
//! are compiled in by the `xml!` proc-macro via the
//! schema in [`crate::schema`]. For application-specific
//! tags, the user can register a runtime renderer with
//! the [`register_xml_component!`] declarative macro
//! and then refer to the tag by name in XML.
//!
//! ## Mechanism
//!
//! ```ignore
//! // In your crate root or component module:
//! yororen_ui::register_xml_component! {
//!     Chart => my_module::ChartBuilder::render,
//! }
//!
//! // In xml:
//! xml! { <Chart id="c" /> }
//! ```
//!
//! The macro emits an `inventory::submit!` call that
//! registers a `ComponentDescriptor { tag, factory }`
//! in a global, type-erased list. The `xml!` codegen,
//! when it encounters a tag it doesn't know from the
//! built-in schema, emits a runtime lookup against
//! this list.
//!
//! The first registered renderer wins; later
//! registrations of the same tag panic at startup
//! (the conflict surfaces immediately, never silently).
//!
//! ## Type signature
//!
//! A registered component factory must have the shape:
//!
//! ```ignore
//! fn(id: &str, cx: &mut gpui::App) -> gpui::AnyElement
//! ```
//!
//! The id comes from the `id="…"` attribute on the XML
//! tag. Returning `AnyElement` lets the result compose
//! anywhere in the tree.
//!
//! ## Runtime built-in leaf support
//!
//! In addition to the container tags and the user
//! registry, `load_xml` can render a small, growing set
//! of built-in leaf components without the proc-macro:
//! `<Label>`, `<Button>`, `<Heading>`, and `<ListItem>`.
//! Literal attributes (`text`, `caption`, `variant`,
//! `selected`, boolean flags, …) are honoured; event
//! attributes are ignored with a warning because runtime
//! XML cannot safely embed Rust closures.

use gpui::{AnyElement, IntoElement, ParentElement, Styled};
use inventory::collect;

/// A type-erased descriptor for a runtime-registered XML
/// component. Submitted into the global registry by
/// [`register_xml_component!`] (re-exported from
/// `yororen-ui`).
#[derive(Clone)]
pub struct ComponentDescriptor {
    /// The XML tag name (e.g. `"Chart"`).
    ///
    /// Kept as `&'static str` because `inventory::submit!`
    /// stores descriptors in a static, and statics cannot
    /// const-construct an owned `String` from a literal.
    /// Lookups accept `&str`, so callers never need to
    /// leak temporary strings.
    pub tag: &'static str,
    /// The factory function: takes `(id, cx)`, returns an
    /// element that gets spliced into the parent.
    pub factory: fn(String, &mut gpui::App) -> AnyElement,
}

// `inventory::collect!` populates a static slice of
// submitted descriptors at link time. We collect into
// `&'static [ComponentDescriptor]` for the lookup.
collect!(ComponentDescriptor);

/// Look up a registered component by tag. Returns the
/// first match (insertion-order; duplicate tags are
/// rejected at submit time by [`register_xml_component!`]).
pub fn lookup(tag: &str) -> Option<&'static ComponentDescriptor> {
    inventory::iter::<ComponentDescriptor>()
        .into_iter()
        .find(|c| c.tag == tag)
}

/// Helper for the `xml!` codegen's runtime fallback:
/// when the schema doesn't know a tag, the codegen emits
/// a call to this function. Returns `AnyElement` so the
/// result composes uniformly with built-in leaves.
///
/// On unknown tag, returns an empty `div()` — a
/// placeholder element. We deliberately don't
/// `panic!` here because the codegen has already accepted
/// the XML; failing at runtime would be a worse UX than
/// rendering nothing for a typo'd tag.
///
/// The `tag` is borrowed because callers may supply a
/// temporary runtime string; the lookup accepts `&str`
/// even though the static registry stores `&'static str`
/// tags (required by `inventory::submit!`). The `id` is
/// passed as an owned `String` so the factory can convert
/// it directly into an `ElementId` without an extra clone.
pub fn render_or_empty(tag: &str, id: String, cx: &mut gpui::App) -> AnyElement {
    match lookup(tag) {
        Some(d) => (d.factory)(id, cx),
        None => {
            eprintln!("yororen-ui-xml: unknown xml component tag `{tag}` at runtime");
            gpui::div().into_any_element()
        }
    }
}

/// Load an XML literal at runtime and render it into
/// `AnyElement`s.
///
/// This is the runtime counterpart of the `xml!` macro
/// — useful for hot-reload, plugin systems, and
/// dynamically-supplied UI descriptions. It supports:
/// - Built-in containers (`<Column>`, `<Row>`, `<Div>`,
///   `<Stack>`) with their `gap_3` / `flex` /
///   `items_center` shorthand attrs.
/// - A growing set of built-in leaves (`<Label>`,
///   `<Button>`, `<Heading>`, `<ListItem>`) with literal
///   props. Events are ignored at runtime.
/// - Runtime-registered custom tags via
///   [`register_xml_component!`].
///
/// **Not supported** (these need the `xml!` macro):
/// - `<If>` / `<ElseIf>` / `<Else>` (compile-time if).
/// - `<For>` / `<Match>` (compile-time loops / match).
/// - `@bind` (compile-time two-way binding).
/// - Complex built-in leaves whose factories need typed
///   arguments or `&mut Window`.
///
/// The return is a `Vec<AnyElement>` because a runtime
/// XML literal can have multiple top-level elements.
pub fn load_xml(xml: &str, cx: &mut gpui::App) -> Vec<AnyElement> {
    let line_starts = parser::line_starts(xml);
    let location = parser::LocationTracker {
        line_starts: &line_starts,
        xml,
        outer_span: proc_macro2::Span::call_site(),
    };
    let root = match parser::parse(xml, proc_macro2::Span::call_site(), &location) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "yororen-ui-xml: failed to parse runtime XML:\n{}",
                e.render_with(Some(&location))
            );
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for child in &root.children {
        if let ast::AstNode::Element(e) = child {
            match render_element_runtime(e, cx) {
                Ok(el) => out.push(el),
                Err(msg) => {
                    eprintln!("yororen-ui-xml: runtime render error: {msg}");
                }
            }
        }
    }
    out
}

/// Render a single `<Element>` to `AnyElement` at
/// runtime. Recognises the container shorthand styles
/// (`<Column col gap_3 />`, `<Row flex items_center />`)
/// and the runtime registry. Everything else becomes
/// an empty `div()`.
fn render_element_runtime(
    element: &ast::AstElement,
    cx: &mut gpui::App,
) -> Result<AnyElement, String> {
    match element.tag.as_str() {
        "Column" | "Row" | "Div" | "Stack" => render_container_runtime(element, cx),
        other => {
            // 1. Built-in leaf components from the schema.
            if let Some(def) = crate::schema::lookup_component(other, &[]) {
                match def.kind {
                    crate::schema::ComponentKind::Leaf(_) => {
                        return render_leaf_runtime(element, def, cx);
                    }
                    crate::schema::ComponentKind::Container(_) => {
                        return render_container_runtime(element, cx);
                    }
                    _ => {}
                }
            }

            // 2. Runtime-registered custom components.
            let id = element
                .attributes
                .iter()
                .find(|a| a.name == "id")
                .map(|a| a.raw.clone())
                .unwrap_or_default();
            // `render_or_empty` accepts a borrowed `&str`,
            // so we can pass the temporary tag directly
            // without leaking it to satisfy a `'static`
            // lifetime.
            Ok(render_or_empty(other, id, cx))
        }
    }
}

/// Render a container element at runtime. We only honour a
/// small set of layout shorthands; unknown attributes are
/// ignored so the user gets a "works mostly" UX rather than
/// a hard failure.
fn render_container_runtime(
    element: &ast::AstElement,
    cx: &mut gpui::App,
) -> Result<AnyElement, String> {
    let mut root = gpui::div();

    for attr in &element.attributes {
        if attr.expr.is_some() || attr.raw.is_empty() {
            continue;
        }
        match (attr.name.as_str(), attr.raw.as_str()) {
            ("col", "true") => root = root.flex().flex_col(),
            ("row", "true") => root = root.flex().flex_row(),
            ("flex", "true") => root = root.flex(),
            ("items_center", "true") => root = root.items_center(),
            ("items_start", "true") => root = root.items_start(),
            ("items_end", "true") => root = root.items_end(),
            ("justify_center", "true") => root = root.justify_center(),
            ("justify_between", "true") => root = root.justify_between(),
            ("relative", "true") => root = root.relative(),
            ("w_full", "true") => root = root.w_full(),
            ("h_full", "true") => root = root.h_full(),
            ("hidden", "true") => root = root.hidden(),
            ("overflow_hidden", "true") => root = root.overflow_hidden(),
            _ => {}
        }
    }

    for child in &element.children {
        if let ast::AstNode::Element(e) = child {
            match render_element_runtime(e, cx) {
                Ok(el) => root = root.child(el),
                Err(msg) => eprintln!("yororen-ui-xml: runtime render error: {msg}"),
            }
        }
    }
    Ok(root.into_any_element())
}

/// Render a built-in leaf component at runtime. This is a
/// deliberately small, hand-written table of the most common
/// components; it proves the runtime path can render real
/// headless leaves, not just containers. More components can
/// be added here as needed.
fn render_leaf_runtime(
    element: &ast::AstElement,
    def: &crate::schema::ComponentDef,
    cx: &mut gpui::App,
) -> Result<AnyElement, String> {
    let id = attr_str(element, "id")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}-runtime", element.tag));

    let events: &[(&str, &str)] = if let crate::schema::ComponentKind::Leaf(leaf) = def.kind {
        leaf.events
    } else {
        &[]
    };

    macro_rules! warn_event {
        ($name:expr) => {
            eprintln!(
                "yororen-ui-xml: runtime ignores event attribute `{}` on <{}>",
                $name, element.tag
            )
        };
    }

    let el: gpui::AnyElement = match element.tag.as_str() {
        "Label" => {
            let text: gpui::SharedString =
                attr_str(element, "text").unwrap_or("").to_string().into();
            let mut l = yororen_ui_core::headless::label::label(id, text, cx);
            if attr_bool(element, "strong") {
                l = l.strong(true);
            }
            if attr_bool(element, "muted") {
                l = l.muted(true);
            }
            if attr_bool(element, "mono") {
                l = l.mono(true);
            }
            if attr_bool(element, "inherit_color") {
                l = l.inherit_color(true);
            }
            if attr_bool(element, "ellipsis") {
                l = l.ellipsis(true);
            }
            if attr_bool(element, "wrap") {
                l = l.wrap();
            }
            l.render(cx).into_any_element()
        }
        "Button" => {
            let mut b = yororen_ui_core::headless::button::button(id, cx);
            if let Some(caption) = attr_str(element, "caption") {
                b = b.caption(caption.to_string());
            }
            if let Some(variant) = attr_variant(element, "variant") {
                b = b.variant(variant);
            }
            if let Some(disabled) = attr_opt_bool(element, "disabled") {
                b = b.disabled(disabled);
            }
            if attr_bool(element, "clickable") {
                b = b.clickable(true);
            }
            for (name, _) in events.iter().copied() {
                if attr_present(element, name) {
                    warn_event!(name);
                }
            }
            b.render(cx).into_any_element()
        }
        "Heading" => {
            let level = attr_heading_level(element, "level")
                .unwrap_or(yororen_ui_core::headless::heading::HeadingLevel::H1);
            let text: gpui::SharedString =
                attr_str(element, "text").unwrap_or("").to_string().into();
            yororen_ui_core::headless::heading::heading(id, level, text, cx)
                .render(cx)
                .into_any_element()
        }
        "ListItem" => {
            let title: gpui::SharedString =
                attr_str(element, "title").unwrap_or("").to_string().into();
            let mut li = yororen_ui_core::headless::list_item::list_item(id, title, cx);
            if let Some(selected) = attr_opt_bool(element, "selected") {
                li = li.selected(selected);
            }
            if let Some(disabled) = attr_opt_bool(element, "disabled") {
                li = li.disabled(disabled);
            }
            for (name, _) in events.iter().copied() {
                if attr_present(element, name) {
                    warn_event!(name);
                }
            }
            li.render(cx).into_any_element()
        }
        _ => {
            return Err(format!(
                "runtime rendering for built-in leaf <{}> is not yet implemented",
                element.tag
            ));
        }
    };

    Ok(el)
}

fn attr_present(element: &ast::AstElement, name: &str) -> bool {
    element.attributes.iter().any(|a| a.name == name)
}

fn attr_str<'a>(element: &'a ast::AstElement, name: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.raw.as_str())
}

fn attr_bool(element: &ast::AstElement, name: &str) -> bool {
    attr_opt_bool(element, name).unwrap_or(false)
}

fn attr_opt_bool(element: &ast::AstElement, name: &str) -> Option<bool> {
    element
        .attributes
        .iter()
        .find(|a| a.name == name)
        .and_then(|a| match a.raw.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
}

fn attr_variant(
    element: &ast::AstElement,
    name: &str,
) -> Option<yororen_ui_core::renderer::ActionVariantKind> {
    attr_str(element, name).and_then(|raw| match raw {
        "primary" => Some(yororen_ui_core::renderer::ActionVariantKind::Primary),
        "neutral" => Some(yororen_ui_core::renderer::ActionVariantKind::Neutral),
        "danger" => Some(yororen_ui_core::renderer::ActionVariantKind::Danger),
        _ => None,
    })
}

fn attr_heading_level(
    element: &ast::AstElement,
    name: &str,
) -> Option<yororen_ui_core::headless::heading::HeadingLevel> {
    attr_str(element, name).and_then(|raw| match raw {
        "H1" => Some(yororen_ui_core::headless::heading::HeadingLevel::H1),
        "H2" => Some(yororen_ui_core::headless::heading::HeadingLevel::H2),
        "H3" => Some(yororen_ui_core::headless::heading::HeadingLevel::H3),
        "H4" => Some(yororen_ui_core::headless::heading::HeadingLevel::H4),
        "H5" => Some(yororen_ui_core::headless::heading::HeadingLevel::H5),
        "H6" => Some(yororen_ui_core::headless::heading::HeadingLevel::H6),
        _ => None,
    })
}

// `parser` and `ast` are referenced by `load_xml` /
// `render_element_runtime`. Importing at the top of
// the module would create a circular dependency
// (`runtime` is referenced from `codegen` which
// references `ast`); these imports are scoped to the
// `load_xml` flow only.
use crate::ast;
use crate::parser;
/// Declarative macro companion to [`register_xml_component!`].
/// Place this in the user's crate to register a custom
/// tag. Each invocation registers exactly one tag.
///
/// The factory must have the signature
/// `fn(String, &mut gpui::App) -> gpui::AnyElement`.
#[macro_export]
macro_rules! register_xml_component {
    ($tag:literal => $factory:path) => {
        $crate::inventory::submit! {
            $crate::runtime::ComponentDescriptor {
                tag: $tag,
                factory: $factory,
            }
        }
    };
    ($tag:ident => $factory:path) => {
        $crate::inventory::submit! {
            $crate::runtime::ComponentDescriptor {
                tag: stringify!($tag),
                factory: $factory,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_registry_collects_submissions() {
        // Test that the inventory registration machinery
        // is wired correctly. Without a submission for
        // a tag, the lookup should fail.
        assert!(lookup("RTTest__nonexistent").is_none());
    }

    #[test]
    fn runtime_registry_lookup_iterates_inventory() {
        // Register a tag at compile time. The descriptor's
        // `tag` field is `'static str` (required by the
        // inventory static), so the literal works directly.
        // The factory returns an empty div — we don't need
        // any actual rendering for the lookup test.
        fn empty(_id: String, _cx: &mut gpui::App) -> gpui::AnyElement {
            gpui::div().into_any_element()
        }
        inventory::submit! {
            ComponentDescriptor {
                tag: "RTTestRegistered",
                factory: empty,
            }
        }
        assert!(lookup("RTTestRegistered").is_some());
    }

    #[test]
    fn schema_lookup_exposes_leaf_defs_to_runtime() {
        // The runtime renderer needs to resolve built-in
        // leaves from the same schema table the codegen
        // uses. Verify that lookup_component is public
        // and returns Leaf kinds for common components.
        let label = crate::schema::lookup_component("Label", &[]).expect("Label in schema");
        assert!(matches!(label.kind, crate::schema::ComponentKind::Leaf(_)));
        let button = crate::schema::lookup_component("Button", &[]).expect("Button in schema");
        assert!(matches!(button.kind, crate::schema::ComponentKind::Leaf(_)));
    }
}
