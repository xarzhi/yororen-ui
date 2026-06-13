//! Compile-time diagnostics for the `xml!` macro.
//!
//! Errors are collected as structured payloads by the parser /
//! codegen and converted into `proc_macro2::TokenStream` panic
//! messages by the macro entry point. The split keeps the
//! underlying crate (`yororen-ui-xml`) free of
//! `proc_macro::Diagnostic` so it can be tested without the
//! proc-macro harness.

use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct XmlError {
    pub kind: XmlErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum XmlErrorKind {
    /// The XML string failed to parse (malformed, mismatched tag,
    /// unterminated element, etc.).
    ParseError,
    /// An XML element used a tag the schema does not know.
    UnknownTag,
    /// An XML element had an attribute the schema does not
    /// accept on that tag.
    UnknownAttribute,
    /// An attribute value could not be parsed as a Rust
    /// expression.
    InvalidExpression,
    /// The XML is structurally valid but unsupported by the
    /// current phase 1 MVP.
    Unsupported,
}

impl XmlError {
    pub fn new(kind: XmlErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }

    /// Render the error to a string suitable for `panic!` /
    /// `compile_error!`.
    pub fn render(&self) -> String {
        let kind = match self.kind {
            XmlErrorKind::ParseError => "XML parse error",
            XmlErrorKind::UnknownTag => "unknown xml tag",
            XmlErrorKind::UnknownAttribute => "unknown xml attribute",
            XmlErrorKind::InvalidExpression => "invalid attribute expression",
            XmlErrorKind::Unsupported => "unsupported xml construct",
        };
        format!("xml!: {kind}: {}", self.message)
    }
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}
