use proc_macro2::TokenStream;
use quote::quote;

use crate::ast::AstAttribute;
use crate::error::{XmlError, XmlErrorKind};

/// Parse a hex colour literal (`#rrggbb` or `#rrggbbaa`) and
/// emit the corresponding gpui constructor. Rejects other
/// literal forms and points the user toward a brace expression.
pub(crate) fn parse_hex_color(raw: &str, attr: &AstAttribute) -> Result<TokenStream, XmlError> {
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
