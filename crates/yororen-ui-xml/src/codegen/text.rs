use proc_macro2::TokenStream;
use quote::quote;

use crate::ast::{AstAttribute, AstNode};
use crate::error::XmlError;

use crate::codegen::parse_ts;

/// Build the value for a "text-like" attribute. Supports
/// brace interpolation: `text="Count: {count}"` becomes
/// `format!("Count: {}", count).into()`.
///
/// String literals without `{` are emitted as
/// `(#raw).to_string()` (the same path as before).
pub(crate) fn text_attr_value(attr: &AstAttribute) -> Result<TokenStream, XmlError> {
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
#[derive(Debug, Clone)]
pub(crate) enum InterpPart {
    /// A literal fragment (no braces).
    Literal(String),
    /// A `{…}` expression.
    Expr(String),
}
pub(crate) fn parse_string_interpolation(text: &str) -> Option<Vec<InterpPart>> {
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
pub(crate) fn render_string_interpolation(
    parts: &[InterpPart],
    attr: &AstAttribute,
) -> TokenStream {
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
pub(crate) fn extract_text_content(children: &[AstNode]) -> Option<String> {
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
