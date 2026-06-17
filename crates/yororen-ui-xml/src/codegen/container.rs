use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::{AstAttribute, AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::{
    ContainerDef, is_known_shorthand_method, is_spacing_prefix, is_spacing_shorthand,
    is_stateful_interactive_method,
};

use crate::codegen::{
    attr::attr_value_tokens, codegen_child, control_flow::codegen_if_chain,
    diagnostics::did_you_mean, parse_ts,
};
use crate::schema::ComponentDef;

pub(crate) fn codegen_container(
    element: &AstElement,
    def: ContainerDef,
    cx: &TokenStream,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    // Build the container as a sequence of `let __el = ...;`
    // statements rather than one giant method chain. This keeps
    // the type-checker happy when the XML grows large and
    // contains many closures (workaround for a rustc quirk
    // where huge closure-bearing expressions can be mis-parsed
    // in `impl Trait` return positions).
    let mut stmts: Vec<TokenStream> = Vec::new();
    let mut stateful = false;
    stmts.push(quote! { let __el = gpui::div(); });

    for attr in &element.attributes {
        apply_container_attr(&mut stmts, attr, def, element, &mut stateful)?;
    }

    // Walk children, merging consecutive If/ElseIf/Else
    // into a single Rust if/else chain (which must be a
    // single block expression so it can be the argument
    // of `ParentElement::child`).
    let mut i = 0;
    while i < element.children.len() {
        let child = &element.children[i];
        if matches!(
            child,
            AstNode::Element(e)
                if matches!(e.tag.as_str(), "If" | "ElseIf" | "Else")
        ) {
            // Collect the chain.
            let mut j = i;
            while j < element.children.len() {
                if let AstNode::Element(e) = &element.children[j] {
                    if !matches!(e.tag.as_str(), "If" | "ElseIf" | "Else") {
                        break;
                    }
                } else {
                    break;
                }
                j += 1;
            }
            let chain_expr =
                codegen_if_chain(&element.children[i..j], cx, source_file, user_schema)?;
            stmts.push(quote! { let __el = ::gpui::ParentElement::child(__el, #chain_expr); });
            i = j;
        } else {
            let child_expr = codegen_child(child, cx, source_file, user_schema)?;
            stmts.push(quote! { let __el = ::gpui::ParentElement::child(__el, #child_expr); });
            i += 1;
        }
    }

    Ok(quote! {
        {
            #(#stmts)*
            __el
        }
    })
}
pub(crate) fn apply_container_attr(
    stmts: &mut Vec<TokenStream>,
    attr: &AstAttribute,
    def: ContainerDef,
    element: &AstElement,
    stateful: &mut bool,
) -> Result<(), XmlError> {
    if attr.name == "id" {
        // `id="my-button"` becomes
        // `::gpui::InteractiveElement::id(__el, "my-button".into())`.
        let value = attr_value_tokens(attr)?;
        stmts.push(quote! {
            let __el = ::gpui::InteractiveElement::id(__el, #value);
        });
        *stateful = true;
        return Ok(());
    }

    // Fixed boolean flag (e.g. `col` for Column → `flex_col`).
    if attr.expr.is_none() {
        let name = attr.name.as_str();
        if let Some((_, method)) = def
            .fixed_methods
            .iter()
            .find(|(attr_name, _)| *attr_name == name)
            .copied()
        {
            let m = format_ident!("{}", method);
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el);
            });
            return Ok(());
        }
    }

    // Brace expression on a container: pass through to the
    // matching Styled method.
    if let Some(expr) = &attr.expr {
        // `gap={pixels}` → `::gpui::Styled::gap(__el, pixels)`
        if is_spacing_prefix(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed = parse_ts(
                expr,
                attr.span,
                attr.byte_offset,
                &format!("expression for `{}`", attr.name),
            )?;
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el, #parsed);
            });
            return Ok(());
        }
        // `gap_3={...}` or `flex_grow={...}` →
        // `::gpui::Styled::gap_3(__el, expr)`
        if is_known_shorthand_method(&attr.name) || is_spacing_shorthand(&attr.name) {
            let m = format_ident!("{}", attr.name);
            let parsed = parse_ts(
                expr,
                attr.span,
                attr.byte_offset,
                &format!("expression for `{}`", attr.name),
            )?;
            stmts.push(quote! {
                let __el = ::gpui::Styled::#m(__el, #parsed);
            });
            return Ok(());
        }
    }

    // Literal value on a spacing / sizing prefix:
    //   - `gap="8"`     → `.gap(px(8.))`        (number = px)
    //   - `gap="8px"`   → `.gap(px(8.))`
    //   - `w="50%"`     → `.w(relative(0.5))`
    //   - `w="2rem"`    → `.w(px(32.))`
    //   - legacy tailwind suffixes like `gap="0p5"`, `p="full"`
    //     still map to `.gap_0p5()` / `.p_full()` for compatibility.
    if attr.expr.is_none() && is_spacing_prefix(&attr.name) {
        let value = attr.raw.as_str();
        let method = format_ident!("{}", attr.name);

        // Pure number → px (the unified default).
        if let Ok(n) = value.parse::<f32>() {
            stmts.push(quote! {
                let __el = ::gpui::Styled::#method(__el, ::gpui::px(#n));
            });
            return Ok(());
        }

        // Explicit unit suffixes.
        if let Some(body) = value.strip_suffix("px") {
            if let Ok(n) = body.parse::<f32>() {
                stmts.push(quote! {
                    let __el = ::gpui::Styled::#method(__el, ::gpui::px(#n));
                });
                return Ok(());
            }
        }
        if let Some(body) = value.strip_suffix("rem") {
            if let Ok(n) = body.parse::<f32>() {
                stmts.push(quote! {
                    let __el = ::gpui::Styled::#method(__el, ::gpui::rems(#n));
                });
                return Ok(());
            }
        }
        if let Some(body) = value.strip_suffix('%') {
            if let Ok(n) = body.parse::<f32>() {
                if !matches!(
                    attr.name.as_str(),
                    "w" | "h" | "size" | "min_w" | "min_h" | "max_w" | "max_h"
                ) {
                    return Err(XmlError::new(
                        XmlErrorKind::InvalidExpression,
                        attr.span,
                        format!(
                            "percentage values are only allowed on width/height/size attributes, not `{}`",
                            attr.name
                        ),
                    )
                    .at(attr.byte_offset));
                }
                let ratio = n / 100.0f32;
                stmts.push(quote! {
                    let __el = ::gpui::Styled::#method(__el, ::gpui::relative(#ratio));
                });
                return Ok(());
            }
        }

        // Legacy tailwind-style suffixes (`0p5`, `1p5`, `full`, `1_2`, …).
        if !is_valid_spacing_suffix(value) {
            return Err(XmlError::new(
                XmlErrorKind::InvalidExpression,
                attr.span,
                format!(
                    "invalid value `{value}` for `{}`; expected a number (px), `Npx`, `Nrem`, `N%`, or a tailwind suffix like `0p5`/`full`",
                    attr.name
                ),
            )
            .at(attr.byte_offset));
        }
        let method_with_suffix = format!("{}_{}", attr.name, value);
        let m = format_ident!("{}", method_with_suffix);
        stmts.push(quote! {
            let __el = ::gpui::Styled::#m(__el);
        });
        return Ok(());
    }

    // Bare method name on a container (`flex`, `flex_col`,
    // `w_full`, …). These are all zero-arg gpui flags in
    // [`is_known_shorthand_method`] / [`is_spacing_shorthand`],
    // so the only valid literal form is `"true"` (which the
    // normaliser adds to bare attributes). Anything else
    // would compile to `.method(__el, "<raw>")` and produce
    // a confusing "too many arguments" rustc error far from
    // the XML — so we reject it up front.
    if attr.expr.is_none()
        && (is_known_shorthand_method(&attr.name)
            || is_spacing_shorthand(&attr.name)
            || is_stateful_interactive_method(&attr.name))
    {
        if attr.raw == "true" {
            let m = format_ident!("{}", attr.name);
            if is_stateful_interactive_method(&attr.name) {
                ensure_stateful(stmts, stateful);
                stmts.push(quote! {
                    let __el = ::gpui::StatefulInteractiveElement::#m(__el);
                });
            } else {
                // gpui's `flex_col()` / `flex_row()` / `flex_wrap()`
                // only set `flex_direction` / `flex_wrap`; they do
                // NOT turn `display` into `flex`. Without `.flex()`
                // first, the container stays `display: Block` and
                // `gap`/`direction` are ignored, so children collapse
                // together. Auto-emit `.flex()` for any flex-layout
                // flag so `<Div flex_col gap="8">` just works.
                if is_flex_layout_flag(&attr.name) {
                    stmts.push(quote! {
                        let __el = ::gpui::Styled::flex(__el);
                    });
                }
                stmts.push(quote! {
                    let __el = ::gpui::Styled::#m(__el);
                });
            }
            return Ok(());
        }
        let name = attr.name.clone();
        let raw = attr.raw.clone();
        return Err(XmlError::new(
            XmlErrorKind::InvalidExpression,
            attr.span,
            format!(
                "attribute `{name}` is a flag (no value); drop `=\"{raw}\"` and use `<{name}>` instead"
            ),
        )
        .at(attr.byte_offset));
    }

    let accepted = accepted_container_attrs(&def);
    let suggestion = did_you_mean(
        &attr.name,
        &accepted
            .split(", ")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>(),
    );
    let hint = if let Some(s) = suggestion {
        format!(" — did you mean `{}`?", s)
    } else {
        String::new()
    };
    Err(XmlError::new(
        XmlErrorKind::UnknownAttribute,
        attr.span,
        format!(
            "unknown attribute `{}` on <{}>; containers only accept shorthand style attributes ({accepted}){hint}",
            attr.name, element.tag,
        ),
    ))
}

/// Promote a bare `gpui::Div` to `Stateful<gpui::Div>` so that
/// `StatefulInteractiveElement` methods (e.g. `overflow_y_scroll`)
/// can be called. Uses an internal id; a user-supplied `id="…"`
/// attribute later in the same tag will simply override it.
fn ensure_stateful(stmts: &mut Vec<TokenStream>, stateful: &mut bool) {
    if !*stateful {
        stmts.push(quote! {
            let __el = ::gpui::InteractiveElement::id(
                __el,
                ::gpui::ElementId::from("__yororen_xml_container"),
            );
        });
        *stateful = true;
    }
}

/// Numeric suffixes accepted after spacing prefixes (`gap`, `p`,
/// `m`, …). Each maps to a real gpui method like `gap_3` / `p_4`.
const NUMERIC_SPACING_SUFFIX: &[&str] = &[
    "0", "0p5", "1", "1p5", "2", "2p5", "3", "3p5", "4", "5", "6", "7", "8", "9", "10", "11", "12",
    "16", "20", "24", "32", "40", "48", "56", "64", "72", "80", "96",
];
/// Textual spacing suffixes. `full` is the only commonly used one;
/// the fractional entries are gpui's Tailwind-style shorthands.
const TEXTUAL_SPACING_SUFFIX: &[&str] = &[
    "full", "1_2", "1_3", "2_3", "1_4", "3_4", "1_5", "2_5", "3_5", "4_5", "1_6", "5_6", "1_12",
];

pub(crate) fn is_valid_spacing_suffix(s: &str) -> bool {
    NUMERIC_SPACING_SUFFIX.contains(&s) || TEXTUAL_SPACING_SUFFIX.contains(&s)
}

/// Flex-layout shorthand flags whose gpui method only mutates
/// `flex_direction` / `flex_wrap`. They require `display: flex`
/// to actually take effect, so the dispatcher emits `.flex()`
/// before them.
pub(crate) fn is_flex_layout_flag(name: &str) -> bool {
    matches!(
        name,
        "flex_col" | "flex_col_reverse" | "flex_row" | "flex_row_reverse" | "flex_wrap"
            | "flex_wrap_reverse" | "flex_nowrap"
    )
}
pub(crate) fn accepted_container_attrs(def: &ContainerDef) -> String {
    let mut parts = vec!["id".to_string()];
    parts.push("flex".to_string());
    for (attr, _) in def.fixed_methods {
        parts.push(attr.to_string());
    }
    parts.push("shorthand style methods (gap_3, p_4, w_full, ...)".to_string());
    parts.join(", ")
}
