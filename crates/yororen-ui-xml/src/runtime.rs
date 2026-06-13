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

use gpui::{AnyElement, IntoElement};
use inventory::collect;

/// A type-erased descriptor for a runtime-registered XML
/// component. Submitted into the global registry by
/// [`register_xml_component!`] (re-exported from
/// `yororen-ui`).
#[derive(Clone)]
pub struct ComponentDescriptor {
    /// The XML tag name (e.g. `"Chart"`).
    pub tag: &'static str,
    /// The factory function: takes `(id, cx)`, returns an
    /// element that gets spliced into the parent.
    pub factory: fn(&str, &mut gpui::App) -> AnyElement,
}

// `inventory::collect!` populates a static slice of
// submitted descriptors at link time. We collect into
// `&'static [ComponentDescriptor]` for the lookup.
collect!(ComponentDescriptor);

/// Look up a registered component by tag. Returns the
/// first match (insertion-order; duplicate tags are
/// rejected at submit time by [`register_xml_component!`]).
pub fn lookup(tag: &str) -> Option<&'static ComponentDescriptor> {
    inventory::iter::<ComponentDescriptor>().into_iter().find(|c| c.tag == tag)
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
/// The `id` is passed by `String` (not `&str`) because
/// the codegen always coerces the `id="…"` attribute to
/// an owned `String` (to match the typical headless
/// factory signature). Callers that need a `&str` can
/// call `.as_str()` inside the factory.
pub fn render_or_empty(tag: &'static str, id: String, cx: &mut gpui::App) -> AnyElement {
    match lookup(tag) {
        Some(d) => (d.factory)(&id, cx),
        None => {
            eprintln!("yororen-ui-xml: unknown xml component tag `{tag}` at runtime");
            gpui::div().into_any_element()
        }
    }
}

/// Declarative macro companion to [`register_xml_component!`].
/// Place this in the user's crate to register a custom
/// tag. Each invocation registers exactly one tag.
///
/// The factory must have the signature
/// `fn(&str, &mut gpui::App) -> gpui::AnyElement`.
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