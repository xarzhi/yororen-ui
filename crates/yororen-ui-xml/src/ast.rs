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
    /// Byte offset into the original (pre-normalisation) XML
    /// where the element's tag starts. Used by the macro entry
    /// point to compute `line:col` diagnostics. For nested
    /// elements this is the position of the `<Tag` opener.
    pub byte_offset: usize,
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
    /// Byte offset into the original XML where the attribute
    /// name starts (used for `line:col` diagnostics).
    pub byte_offset: usize,
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
        /// Byte offset of the text content's first non-whitespace
        /// character in the original XML.
        byte_offset: usize,
    },
}
