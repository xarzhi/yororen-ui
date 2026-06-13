//! Compile-time diagnostics for the `xml!` macro.
//!
//! Errors are collected as structured payloads by the parser /
//! codegen and converted into compiler errors by the macro entry
//! point. The split keeps the underlying crate
//! (`yororen-ui-xml`) free of `proc_macro::Diagnostic` so it can
//! be tested without the proc-macro harness.
//!
//! Each error carries an optional `offset: Option<usize>` —
//! a byte offset into the user's XML literal. The macro
//! entry point combines that with a `LocationTracker` (built
//! from the same XML) to render a `line:col` header and a
//! one-line code snippet with a caret, similar to `rustc`'s
//! own diagnostics.

use proc_macro2::Span;

use crate::parser::LocationTracker;

#[derive(Debug, Clone)]
pub struct XmlError {
    pub kind: XmlErrorKind,
    pub span: Span,
    /// Byte offset into the XML literal where the error
    /// originated. Used by [`XmlError::render_with`] to
    /// produce `line:col` + snippet. `None` means the offset
    /// isn't meaningful (e.g. a structural error that was
    /// caught at a higher level).
    pub offset: Option<usize>,
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
            offset: None,
            message: message.into(),
        }
    }

    /// Builder: attach a byte offset (into the XML literal)
    /// to this error. Used by the parser / codegen so the
    /// macro entry point can render `line:col` diagnostics.
    pub fn at(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Render the error to a string suitable for `panic!` /
    /// `compile_error!`. The single-line format is kept for
    /// backwards compatibility; prefer [`Self::render_with`]
    /// when the caller has a [`LocationTracker`].
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

    /// Render the error with a [`LocationTracker`] to produce
    /// a multi-line diagnostic:
    ///
    /// ```text
    /// xml!: unknown xml tag at line 3, column 5:
    ///     <BadTag />
    ///     ^
    /// ```
    ///
    /// The fallback (no offset, or out-of-range offset) is
    /// the same single-line message as [`Self::render`].
    pub fn render_with(&self, location: Option<&LocationTracker<'_>>) -> String {
        let kind = match self.kind {
            XmlErrorKind::ParseError => "XML parse error",
            XmlErrorKind::UnknownTag => "unknown xml tag",
            XmlErrorKind::UnknownAttribute => "unknown xml attribute",
            XmlErrorKind::InvalidExpression => "invalid attribute expression",
            XmlErrorKind::Unsupported => "unsupported xml construct",
        };
        let Some(loc) = location else {
            return self.render();
        };
        let Some(offset) = self.offset else {
            return self.render();
        };
        if offset > loc.xml.len() {
            return self.render();
        }
        let (line, col) = loc.line_col(offset);
        let snippet = loc.snippet(offset);
        format!(
            "xml!: {kind} at line {line}, column {col}:\n{snippet}\n  = note: {}",
            self.message
        )
    }
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

impl std::error::Error for XmlError {}