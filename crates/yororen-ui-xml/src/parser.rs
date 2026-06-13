//! XML string → [`AstNode`] tree using `roxmltree`.
//!
//! The parser is lenient about whitespace at the top level but
//! strict about the XML itself. Errors carry the originating
//! `Span` so the macro can attach a useful diagnostic.
//!
//! ## Bool shorthand
//!
//! Users often write `<Column flex col items_center />` to
//! flip on a few boolean flags. Strict XML forbids
//! value-less attributes, so the [`normalise_bool_attrs`]
//! helper rewrites the input string before handing it to
//! `roxmltree`. Any attribute name that is **not** followed
//! by `=` (i.e. the user wrote `flex` instead of
//! `flex="true"`) gets `"true"` appended.
//!
//! The rewriter is conservative: it only fires inside
//! element start tags (between `<Tag` and `>`), never inside
//! attribute values, never inside CDATA, and never inside
//! `<?xml ... ?>` processing instructions.

use proc_macro2::Span;

use crate::ast::{AstAttribute, AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};

/// Parse a literal XML string into an [`AstElement`]. The input
/// must contain exactly one root element.
///
/// `outer_span` is the `Span` covering the entire literal
/// (typically the `xml!` invocation) and is used as the
/// fallback for any sub-element whose own span can't be
/// recovered. The `LocationTracker` carries the user's
/// original (pre-normalisation) line offsets so error
/// messages can include a `line:col` hint.
pub fn parse(
    xml: &str,
    outer_span: Span,
    location: &LocationTracker,
) -> Result<AstElement, XmlError> {
    // `@bind` is XML-illegal (the `@` is reserved). The
    // codegen arms recognise `bind` specially; we pre-rewrite
    // `@bind` to `bind` here so roxmltree is happy.
    let un_atd = rewrite_at_bind(xml);
    let preprocessed = rewrite_let_attrs(&un_atd);
    let normalized = normalise_bool_attrs(&preprocessed);
    let doc = roxmltree::Document::parse(&normalized).map_err(|e| {
        // The error from roxmltree is a textual message
        // without a precise position; try to extract the
        // "at line N, column M" hint and convert to an
        // offset. If that fails, fall back to offset 0
        // (the whole literal).
        let offset = parse_roxmltree_offset(&format!("{e}"), &normalized).unwrap_or(0);
        XmlError::new(XmlErrorKind::ParseError, outer_span, format!("{e}")).at(offset)
    })?;

    let root = doc.root_element();
    Ok(element_from(root, outer_span, location))
}

/// Parse `error_text` looking for "at line N, column M"
/// (the message format `roxmltree` emits) and convert it
/// to a byte offset using the line_starts table baked
/// into the normalised XML. Returns `None` if the error
/// format is unexpected.
fn parse_roxmltree_offset(error_text: &str, normalized_xml: &str) -> Option<usize> {
    // roxmltree formats errors roughly as:
    //   "<error message> at line N, column M"
    let line_marker = " at line ";
    let col_marker = ", column ";
    let line_idx = error_text.find(line_marker)? + line_marker.len();
    let after_line = &error_text[line_idx..];
    let line_end = after_line.find(|c: char| !c.is_ascii_digit())?;
    let line: usize = after_line[..line_end].parse().ok()?;
    let col_idx = after_line.find(col_marker)? + col_marker.len();
    let after_col = &after_line[col_idx..];
    let col_end = after_col.find(|c: char| !c.is_ascii_digit())?;
    let col: usize = after_col[..col_end].parse().ok()?;
    // Build a tiny line_starts table for the normalised
    // XML on the fly (we don't have access to the
    // tracker's table here, but the offsets line up
    // because the preprocessors are line-preserving).
    let starts = LocationTracker::compute(normalized_xml);
    let l_idx = line.saturating_sub(1).min(starts.len().saturating_sub(1));
    Some(starts[l_idx] + col.saturating_sub(1))
}

/// Convert `@bind` to `bind` so the attribute name is
/// XML-legal. The codegen looks for the `bind` name and
/// treats it as the special two-way binding sugar.
pub(crate) fn rewrite_at_bind(input: &str) -> String {
    input.replace("@bind", "bind")
}

/// Compute the byte offset of every line start in `xml`.
/// Index 0 is always 0; subsequent entries are the byte
/// offset right after each `\n`. This is the table the
/// `LocationTracker` uses to convert a normalised offset
/// back to a `line:col` in the user's source.
pub fn line_starts(xml: &str) -> Vec<usize> {
    LocationTracker::compute(xml)
}

/// Tracks the user's source for error messages. The XML
/// parser sees a normalised version (bare attrs are
/// rewritten to `="true"`, brace expressions wrapped in
/// `"…"`, `let:` rewritten to `let_`). The line_starts
/// table is built from the **original** XML so that any
/// error can be mapped back to the user's source line/col.
pub struct LocationTracker<'a> {
    pub line_starts: &'a [usize],
    pub xml: &'a str,
    /// The `proc_macro2::Span` that covers the entire XML
    /// literal — the macro entry point passes its `call_site`
    /// span here. Used as the fallback when an error has
    /// no meaningful byte offset (e.g. an error raised by
    /// a structural validator).
    pub outer_span: Span,
}

impl<'a> LocationTracker<'a> {
    /// Build a line-starts table for the given XML.
    /// Index 0 is always 0; subsequent entries are the
    /// byte offset of the start of each line (i.e. the
    /// position right after each `\n`).
    fn compute(xml: &str) -> Vec<usize> {
        let mut starts = vec![0usize];
        for (i, b) in xml.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        starts
    }

    /// Compute the `line:col` for a given offset into the
    /// normalised XML. (We use the normalised form because
    /// that's what `roxmltree` hands us; line counts in
    /// the original and normalised form are identical
    /// because the preprocessors are line-preserving.)
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        if self.line_starts.is_empty() {
            return (1, 1);
        }
        let mut line = 0;
        for (i, start) in self.line_starts.iter().enumerate() {
            if *start <= offset {
                line = i;
            } else {
                break;
            }
        }
        (line + 1, offset - self.line_starts[line] + 1)
    }

    /// Render a one-line snippet of the XML around `offset`
    /// for inclusion in error messages. The line containing
    /// `offset` is returned with a caret pointing at the
    /// offending column.
    pub fn snippet(&self, offset: usize) -> String {
        let (line, col) = self.line_col(offset);
        let line_start = self
            .line_starts
            .get(line.saturating_sub(1))
            .copied()
            .unwrap_or(0);
        let line_end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.xml.len());
        let line_text = &self.xml[line_start..line_end.min(self.xml.len())];
        let line_text = line_text.trim_end();
        let caret = format!("\n{}^", " ".repeat(col.saturating_sub(1)));
        format!("{line_text}{caret}")
    }

    /// Convenience accessor for the proc-macro span that
    /// covers the whole XML literal. The codegen uses this
    /// for structural errors that have no meaningful byte
    /// offset (e.g. "macro invariants violated").
    pub fn span_outer(&self) -> Span {
        self.outer_span
    }
}

/// Convert `<For ... let:item>...</For>` and
/// `<For ... let:index={i}>...</For>` into forms that
/// roxmltree can parse. The transformation is intentionally
/// lossless from the codegen's perspective: we record the
/// original names so the codegen arm for `<For>` can rebuild
/// `let:item` and `let:index` in the generated Rust.
///
/// Operates on bytes to preserve UTF-8 sequences (the
/// preprocessor must never mangle multi-byte chars).
pub(crate) fn rewrite_let_attrs(input: &str) -> String {
    // We need to find `let:NAME` and `let:NAME={...}` inside
    // start tags and rewrite them. The codegen reads the
    // original name from a marker; here we only need to make
    // roxmltree happy. Use a stable encoding:
    //   let:item        → let_item
    //   let:index={i}   → let_index="{i}"
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 3 < bytes.len()
            && bytes[i] == b'l'
            && bytes[i + 1] == b'e'
            && bytes[i + 2] == b't'
            && bytes[i + 3] == b':'
        {
            // Replace the colon with an underscore. This is
            // a textual rewrite (not tag-aware) — the only
            // places we expect `let:` are inside `<For>`'s
            // start tag, so this is safe in practice.
            out.extend_from_slice(b"let_");
            i += 4;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                out.push(bytes[i]);
                i += 1;
            }
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).expect("rewrite_let_attrs: output is valid UTF-8")
}

/// Rewrite `<Tag bare_attr bare_attr2="v" />` →
/// `<Tag bare_attr="true" bare_attr2="v" />`. Operates on a
/// single source string and is safe for typical XML used by
/// `xml!` (no CDATA sections, no comments that look like
/// attributes).
///
/// **UTF-8 note**: the output is a `Vec<u8>` rather than a
/// `String` so that the preprocessor can preserve multi-byte
/// UTF-8 sequences byte-for-byte. Pushing individual bytes
/// to a `String` and re-encoding as `u8 as char` would
/// convert every byte > 0x7F to a Latin-1 char, mangling
/// the result (the classic UTF-8 ↔ Latin-1 mojibake). We
/// only decode as UTF-8 at the very end of the pipeline.
pub(crate) fn normalise_bool_attrs(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        // Find the next `<` (start of a tag).
        if bytes[i] == b'<' {
            // Copy `<` and detect comment / CDATA / PI / doctype
            // so we don't mangle them.
            out.push(b'<');
            i += 1;
            // Comments
            if i + 3 < bytes.len()
                && bytes[i] == b'!'
                && bytes[i + 1] == b'-'
                && bytes[i + 2] == b'-'
            {
                out.extend_from_slice(b"!--");
                i += 3;
                while i < bytes.len() {
                    out.push(bytes[i]);
                    if bytes[i] == b'-'
                        && i + 2 < bytes.len()
                        && bytes[i + 1] == b'-'
                        && bytes[i + 2] == b'>'
                    {
                        out.push(b'-');
                        out.push(b'>');
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            // PI / doctype / CDATA: copy verbatim until the
            // matching `>` (no attribute processing).
            if i < bytes.len() && (bytes[i] == b'?' || bytes[i] == b'!') {
                while i < bytes.len() && bytes[i] != b'>' {
                    out.push(bytes[i]);
                    i += 1;
                }
                if i < bytes.len() {
                    out.push(b'>');
                    i += 1;
                }
                continue;
            }
            // End tag: copy verbatim.
            if i < bytes.len() && bytes[i] == b'/' {
                while i < bytes.len() && bytes[i] != b'>' {
                    out.push(bytes[i]);
                    i += 1;
                }
                if i < bytes.len() {
                    out.push(b'>');
                    i += 1;
                }
                continue;
            }
            // Start tag: walk attributes.
            // First, the tag name.
            while i < bytes.len() && !is_tag_name_end(bytes[i]) {
                out.push(bytes[i]);
                i += 1;
            }
            // Now: whitespace + attributes until `>` or `/>`.
            while i < bytes.len() && bytes[i] != b'>' {
                // Skip whitespace.
                if bytes[i].is_ascii_whitespace() {
                    out.push(bytes[i]);
                    i += 1;
                    continue;
                }
                if bytes[i] == b'/' {
                    // self-closing
                    out.push(b'/');
                    i += 1;
                    continue;
                }
                // Attribute: read name until `=`, whitespace, or `/`.
                let name_start = i;
                while i < bytes.len() && !is_attr_name_end(bytes[i]) {
                    i += 1;
                }
                let name = &input[name_start..i];
                out.extend_from_slice(name.as_bytes());
                if i < bytes.len() && bytes[i] == b'=' {
                    // Normal `attr="value"` or `attr={...}` —
                    // copy as-is until the matching closer.
                    out.push(b'=');
                    i += 1;
                    // Skip whitespace before the value.
                    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                        out.push(bytes[i]);
                        i += 1;
                    }
                    if i >= bytes.len() {
                        break;
                    }
                    let opener = bytes[i];
                    if opener == b'{' {
                        // Rust brace expression. Walk until
                        // the matching `}` (depth-tracked so
                        // closures with `{}` blocks survive).
                        // The whole value is wrapped in
                        // double quotes so the result is
                        // valid XML that roxmltree can
                        // parse; the `strip_brace_expression`
                        // helper later unwraps the quotes
                        // and the outer braces.
                        out.push(b'"');
                        out.push(b'{');
                        i += 1;
                        let mut depth: usize = 1;
                        while i < bytes.len() && depth > 0 {
                            let b = bytes[i];
                            if b == b'{' {
                                depth += 1;
                                out.push(b);
                                i += 1;
                            } else if b == b'}' {
                                depth -= 1;
                                out.push(b);
                                i += 1;
                                if depth == 0 {
                                    break;
                                }
                            } else if b == b'<' {
                                // `<` inside a brace
                                // expression would start
                                // an XML tag — escape as
                                // `&lt;` so the wrapping
                                // attribute stays valid.
                                out.extend_from_slice(b"&lt;");
                                i += 1;
                            } else if b == b'>' {
                                // `>` is technically
                                // allowed in attributes
                                // (no entity issue), but
                                // we escape it for
                                // symmetry / safety.
                                out.extend_from_slice(b"&gt;");
                                i += 1;
                            } else if b == b'&' {
                                // `&` is the start of an
                                // XML entity — escape as
                                // `&amp;` so the
                                // wrapping attribute
                                // stays valid.
                                out.extend_from_slice(b"&amp;");
                                i += 1;
                            } else if b == b'"' {
                                // Inner `"` inside a brace
                                // expression — escape with
                                // XML's `&quot;` so the
                                // wrapping quote doesn't
                                // terminate. The
                                // `strip_brace_expression`
                                // helper later converts it
                                // back to a plain `"` for
                                // Rust.
                                out.extend_from_slice(b"&quot;");
                                i += 1;
                            } else if b == b'\'' {
                                // Inner `'` — same
                                // treatment (XML attribute
                                // values use `&apos;`).
                                out.extend_from_slice(b"&apos;");
                                i += 1;
                            } else {
                                out.push(b);
                                i += 1;
                            }
                        }
                        out.push(b'"');
                    } else if opener == b'"' || opener == b'\'' {
                        out.push(opener);
                        i += 1;
                        while i < bytes.len() && bytes[i] != opener {
                            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                out.push(bytes[i]);
                                out.push(bytes[i + 1]);
                                i += 2;
                            } else {
                                // Copy the byte verbatim
                                // so UTF-8 multi-byte
                                // sequences pass through.
                                out.push(bytes[i]);
                                i += 1;
                            }
                        }
                        if i < bytes.len() {
                            out.push(opener);
                            i += 1;
                        }
                    } else {
                        // Unquoted value — copy until whitespace
                        // or `>` (XML spec forbids this but
                        // roxmltree tolerates it for some
                        // inputs). We don't expect it from
                        // the macro; treat conservatively.
                        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>'
                        {
                            out.push(bytes[i]);
                            i += 1;
                        }
                    }
                } else {
                    // Bare attribute (no `=value`) — inject `="true"`.
                    out.extend_from_slice(b"=\"true\"");
                }
            }
            if i < bytes.len() {
                out.push(b'>');
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    // The input was valid UTF-8 (Rust source), so the
    // output Vec<u8> is also valid UTF-8 — every byte
    // is preserved byte-for-byte and structural markers
    // are ASCII.
    String::from_utf8(out).expect("normalise_bool_attrs: output is valid UTF-8")
}

fn is_tag_name_end(b: u8) -> bool {
    b.is_ascii_whitespace() || b == b'>' || b == b'/'
}

fn is_attr_name_end(b: u8) -> bool {
    b.is_ascii_whitespace() || b == b'>' || b == b'/' || b == b'='
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_bare_attrs() {
        let s = normalise_bool_attrs(r#"<Column flex col gap="3" />"#);
        assert_eq!(s, r#"<Column flex="true" col="true" gap="3" />"#);
    }

    #[test]
    fn leaves_quoted_attrs_untouched() {
        let s = normalise_bool_attrs(r#"<Label text="hello" />"#);
        assert_eq!(s, r#"<Label text="hello" />"#);
    }

    #[test]
    fn wraps_brace_expressions() {
        let s = normalise_bool_attrs(r#"<Label text={value} />"#);
        assert_eq!(s, r#"<Label text="{value}" />"#);
    }

    #[test]
    fn escapes_inner_quotes_in_brace_expressions() {
        // The preprocessor wraps `{...}` in quotes so
        // roxmltree accepts the result. Inner string
        // literal quotes are escaped with the XML
        // entity sequences `&quot;` / `&apos;` so the
        // wrapping quote doesn't terminate early. The
        // `strip_brace_expression` helper reverses
        // the encoding before handing the expression
        // to the Rust parser.
        let s = normalise_bool_attrs(
            r#"<Label text={format!("hi {}", name)} />"#,
        );
        assert_eq!(
            s,
            r#"<Label text="{format!(&quot;hi {}&quot;, name)}" />"#
        );
    }

    #[test]
    fn escapes_xml_specials_in_brace_expressions() {
        // `<`, `>` and `&` would start / terminate
        // XML elements if left unescaped inside the
        // wrapping attribute. The preprocessor
        // encodes them as `&lt;`, `&gt;`, `&amp;`.
        let s = normalise_bool_attrs(
            r#"<Label text={if a < b { c } else { d }} />"#,
        );
        assert_eq!(
            s,
            r#"<Label text="{if a &lt; b { c } else { d }}" />"#
        );

        let s2 = normalise_bool_attrs(r#"<Label text={x & 0xFF} />"#);
        assert_eq!(s2, r#"<Label text="{x &amp; 0xFF}" />"#);
    }

    #[test]
    fn leaves_text_content_alone() {
        let s = normalise_bool_attrs(r#"<Button>Click me</Button>"#);
        assert_eq!(s, r#"<Button>Click me</Button>"#);
    }

    #[test]
    fn handles_brace_with_closure() {
        let s = normalise_bool_attrs(
            r#"<Button on_click={move |_, _, cx| { x.update(cx, |v, _| *v += 1); }} />"#,
        );
        // The result must be valid XML: the brace value is
        // wrapped in double quotes so roxmltree accepts it.
        let expected =
            r#"<Button on_click="{move |_, _, cx| { x.update(cx, |v, _| *v += 1); }}" />"#;
        assert_eq!(s, expected);
    }

    #[test]
    fn strips_brace_expression_with_quotes() {
        let (expr, raw) = strip_brace_expression(r#""{count.to_string()}""#);
        assert_eq!(expr.as_deref(), Some("count.to_string()"));
        assert_eq!(raw, r#""{count.to_string()}""#);
    }

    #[test]
    fn strips_brace_expression_decodes_xml_entities() {
        // `&quot;` inside the wrapped attribute value
        // is decoded back to a literal `"` so the
        // expression is valid Rust source.
        let (expr, _raw) =
            strip_brace_expression(r#""{format!(&quot;hi {name}&quot;)}""#);
        assert_eq!(expr.as_deref(), Some(r#"format!("hi {name}")"#));

        // `&amp;` and `&lt;` / `&gt;` also round-trip.
        let (expr, _raw) = strip_brace_expression(r#""{x &amp; 0xFF}""#);
        assert_eq!(expr.as_deref(), Some("x & 0xFF"));
        let (expr, _raw) = strip_brace_expression(r#""{if a &lt; b { 1 } else { 0 }}""#);
        assert_eq!(expr.as_deref(), Some("if a < b { 1 } else { 0 }"));
    }

    #[test]
    fn line_starts_table_works() {
        let starts = line_starts("a\nb\nc");
        assert_eq!(starts, vec![0, 2, 4]);

        // Single line.
        assert_eq!(line_starts("hello"), vec![0]);

        // Empty.
        assert_eq!(line_starts(""), vec![0]);
    }

    #[test]
    fn location_tracker_snippet() {
        let xml = "<Column>\n    <BadTag />\n</Column>";
        let starts = line_starts(xml);
        let loc = LocationTracker {
            line_starts: &starts,
            xml,
            outer_span: Span::call_site(),
        };
        let (line, _col) = loc.line_col(20); // inside <BadTag />
        assert_eq!(line, 2);
        let snippet = loc.snippet(20);
        // The snippet includes the offending line and a caret.
        assert!(snippet.contains("<BadTag"), "{snippet}");
        assert!(snippet.contains('^'), "{snippet}");
    }
}

fn element_from(
    node: roxmltree::Node,
    fallback_span: Span,
    location: &LocationTracker,
) -> AstElement {
    let tag = node.tag_name().name().to_string();
    let span = span_for_node(node, fallback_span);
    let byte_offset = byte_offset_for_node(node, location);

    let mut attributes = Vec::new();
    for attr in node.attributes() {
        let value = attr.value();
        let attr_span = span_for_attr(attr, span);
        let attr_offset = byte_offset_for_attr(attr, location);
        let (expr, raw) = strip_brace_expression(&value);
        attributes.push(AstAttribute {
            name: attr.name().to_string(),
            raw,
            span: attr_span,
            byte_offset: attr_offset,
            expr,
        });
    }

    let mut children = Vec::new();
    for child in node.children() {
        if child.is_element() {
            children.push(AstNode::Element(element_from(child, span, location)));
        } else if child.is_text() {
            let text = child.text().unwrap_or("");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let text_offset = byte_offset_for_text(child, location, trimmed);
                children.push(AstNode::Text {
                    text: trimmed.to_string(),
                    span: span_for_node(child, span),
                    byte_offset: text_offset,
                });
            }
        }
    }

    AstElement {
        tag,
        span,
        byte_offset,
        attributes,
        children,
    }
}

/// Map a roxmltree node range to a `proc_macro2::Span`. We don't
/// have byte-accurate source maps, so we fall back to the
/// parent span for any node whose range we can't cleanly
/// translate.
fn span_for_node(_node: roxmltree::Node, fallback: Span) -> Span {
    fallback
}

fn span_for_attr(_attr: roxmltree::Attribute, fallback: Span) -> Span {
    fallback
}

/// Byte offset of the `<Tag` opener for an element node.
/// `roxmltree::Node::range()` gives the span of the entire
/// start tag (including attributes and the closing `>`),
/// so we need to step back over those to land on the `<`.
fn byte_offset_for_node(node: roxmltree::Node, _location: &LocationTracker) -> usize {
    let range = node.range();
    let qname_len = node.tag_name().name().len();
    // `range.start` is the position of the `<`. (Verified
    // empirically with roxmltree 0.20 — `range_qname()`
    // excludes the angle bracket.)
    range.start.min(range.end.saturating_sub(qname_len))
}

/// Byte offset of the attribute name (the position of its
/// first character). Used to anchor diagnostics.
fn byte_offset_for_attr(attr: roxmltree::Attribute, _location: &LocationTracker) -> usize {
    attr.range().start
}

/// Byte offset of the first non-whitespace character of a
/// text node. Falls back to the node range start.
fn byte_offset_for_text(node: roxmltree::Node, location: &LocationTracker, trimmed: &str) -> usize {
    let raw = node.text().unwrap_or("");
    if let Some(rel) = raw.find(trimmed) {
        node.range().start + rel
    } else {
        byte_offset_for_node(node, location)
    }
}

/// Detect whether `s` is a brace expression (`{...}`) — with
/// or without surrounding `"..."` quotes (the normaliser
/// always wraps in quotes so that the input remains valid
/// XML). Returns the inner expression and the original
/// string if it is; otherwise returns `(None, s.to_string())`.
fn strip_brace_expression(s: &str) -> (Option<String>, String) {
    let trimmed = s.trim();
    // Strip surrounding quotes if any.
    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    if unquoted.starts_with('{') && unquoted.ends_with('}') {
        // Convert XML entity escapes back to literal
        // characters. The preprocessor encodes inner
        // `<`, `>`, `&`, `"` and `'` inside brace
        // expressions as their XML attribute escape
        // sequences so the wrapping XML attribute
        // doesn't terminate prematurely; we reverse
        // the encoding here before handing the
        // expression to the Rust parser.
        let inner = unquoted[1..unquoted.len() - 1]
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        (Some(inner), s.to_string())
    } else {
        (None, s.to_string())
    }
}

// Avoid the dead-code lint while keeping the helper for later
// use (it can later learn to parse the attribute *position*).
#[allow(dead_code)]
fn _mark_used(_: fn(roxmltree::Attribute, Span) -> Span) {}
