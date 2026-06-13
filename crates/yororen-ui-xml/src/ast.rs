//! Parsed XML tree consumed by the codegen.
//!
//! The parser produces an [`AstNode`] tree. Every node carries
//! the original `Span` so codegen errors can point back to the
//! call-site token. The tree mirrors the XML structure
//! 1-to-1: elements contain attributes and a list of children
//! (other elements or text), and text content lives on
//! [`AstNode::Text`].

use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct AstElement {
    pub tag: String,
    pub span: Span,
    pub attributes: Vec<AstAttribute>,
    pub children: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct AstAttribute {
    pub name: String,
    /// The literal text the user wrote (for diagnostics).
    pub raw: String,
    /// The `Span` of the attribute value, or the attribute name
    /// if the value couldn't be located.
    pub span: Span,
    /// The body of the attribute — either:
    /// - `Some(expr)` for brace expressions (`{...}`), or
    /// - `None` for string literals (`"..."`) — `raw` then
    ///   holds the literal contents.
    pub expr: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Element(AstElement),
    /// Plain text inside a leaf container. Today, text nodes
    /// are only meaningful for labels (`<Label>{value}</Label>`
    /// or `<Label text={value} />`).
    Text {
        text: String,
        span: Span,
    },
}
