//! Compile-time class string parser.
//!
//! Parses Tailwind-like class strings (`"flex gap-md p-md"`)
//! into `Vec<LayoutClass>`. Used by both the `classes!` proc-
//! macro and the XML `class` attribute handler.
//!
//! Unknown tokens are reported with a span (when available)
//! so the user sees a clear compile error.

use yororen_ui_core::headless::layout::{
    Inset, LayoutClass, Length, Spacing,
};

#[derive(Debug)]
pub struct ClassParseError {
    pub token: String,
    pub message: String,
}

impl std::fmt::Display for ClassParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown class `{}`: {}", self.token, self.message)
    }
}

impl std::error::Error for ClassParseError {}

/// Parse a class string into a list of `LayoutClass` tokens.
///
/// Tokens are separated by whitespace. Each token maps to
/// either a fixed variant (e.g. `flex` → `LayoutClass::Flex`)
/// or a value-bearing variant (e.g. `gap-md` →
/// `LayoutClass::Gap(Spacing::Md)`).
pub fn parse_class_string(input: &str) -> Result<Vec<LayoutClass>, ClassParseError> {
    let mut out = Vec::new();
    for raw in input.split_whitespace() {
        if raw.is_empty() {
            continue;
        }
        out.push(parse_one_token(raw)?);
    }
    Ok(out)
}

fn parse_one_token(token: &str) -> Result<LayoutClass, ClassParseError> {
    // Fixed (no value) tokens.
    let fixed: &[(&str, LayoutClass)] = &[
        // flex
        ("flex", LayoutClass::Flex),
        ("flex-col", LayoutClass::FlexCol),
        ("flex-row", LayoutClass::FlexRow),
        ("flex-wrap", LayoutClass::FlexWrap),
        ("flex-1", LayoutClass::Flex1),
        // items
        ("items-start", LayoutClass::ItemsStart),
        ("items-end", LayoutClass::ItemsEnd),
        ("items-center", LayoutClass::ItemsCenter),
        ("items-baseline", LayoutClass::ItemsBaseline),
        ("items-stretch", LayoutClass::ItemsStretch),
        // justify
        ("justify-start", LayoutClass::JustifyStart),
        ("justify-end", LayoutClass::JustifyEnd),
        ("justify-center", LayoutClass::JustifyCenter),
        ("justify-between", LayoutClass::JustifyBetween),
        ("justify-around", LayoutClass::JustifyAround),
        ("justify-evenly", LayoutClass::JustifyEvenly),
        // size
        ("w-full", LayoutClass::WFull),
        ("h-full", LayoutClass::HFull),
        ("size-full", LayoutClass::SizeFull),
        // position
        ("relative", LayoutClass::Relative),
        ("absolute", LayoutClass::Absolute),
        ("top-0", LayoutClass::Top0),
        ("right-0", LayoutClass::Right0),
        ("bottom-0", LayoutClass::Bottom0),
        ("left-0", LayoutClass::Left0),
        ("inset-0", LayoutClass::Inset0),
        // overflow
        ("overflow-hidden", LayoutClass::OverflowHidden),
        ("overflow-scroll", LayoutClass::OverflowScroll),
        // border
        ("border", LayoutClass::Border),
        ("border-1", LayoutClass::Border1),
        // radius
        ("rounded", LayoutClass::Rounded),
        ("rounded-md", LayoutClass::RoundedMd),
        ("rounded-lg", LayoutClass::RoundedLg),
        // shadow
        ("shadow-md", LayoutClass::ShadowMd),
        ("shadow-lg", LayoutClass::ShadowLg),
    ];
    if let Some((_, v)) = fixed.iter().find(|(k, _)| *k == token) {
        return Ok(v.clone());
    }

    // Value-bearing prefixes: gap-*, p-*, px-*, py-*, m-*, mx-*, my-*.
    if let Some(rest) = token.strip_prefix("gap-") {
        let s = parse_spacing(rest)?;
        return Ok(LayoutClass::Gap(s));
    }
    if let Some(rest) = token.strip_prefix("p-") {
        let s = parse_inset(rest)?;
        return Ok(LayoutClass::P(s));
    }
    if let Some(rest) = token.strip_prefix("px-") {
        let s = parse_spacing(rest)?;
        return Ok(LayoutClass::Px(s));
    }
    if let Some(rest) = token.strip_prefix("py-") {
        let s = parse_spacing(rest)?;
        return Ok(LayoutClass::Py(s));
    }
    if let Some(rest) = token.strip_prefix("m-") {
        let s = parse_inset(rest)?;
        return Ok(LayoutClass::M(s));
    }
    if let Some(rest) = token.strip_prefix("mx-") {
        let s = parse_inset(rest)?;
        return Ok(LayoutClass::Mx(s));
    }
    if let Some(rest) = token.strip_prefix("my-") {
        let s = parse_inset(rest)?;
        return Ok(LayoutClass::My(s));
    }

    Err(ClassParseError {
        token: token.to_string(),
        message: "no matching class. Use `flex`, `gap-md`, `p-4`, etc. (see `layout-system-plan.md` §4.1)"
            .to_string(),
    })
}

fn parse_spacing(s: &str) -> Result<Spacing, ClassParseError> {
    let named: &[(&str, Spacing)] = &[
        ("xs", Spacing::Xs),
        ("sm", Spacing::Sm),
        ("md", Spacing::Md),
        ("lg", Spacing::Lg),
        ("xl", Spacing::Xl),
        ("xxl", Spacing::Xxl),
    ];
    if let Some((_, v)) = named.iter().find(|(k, _)| *k == s) {
        return Ok(*v);
    }
    if let Some(rest) = s.strip_suffix("px") {
        let n: f32 = rest
            .parse()
            .map_err(|_| ClassParseError {
                token: format!("{s}px"),
                message: "expected a number before `px` (e.g. `gap-3px` → 3.0 px)".to_string(),
            })?;
        return Ok(Spacing::Px(n));
    }
    if let Ok(n) = s.parse::<f32>() {
        return Ok(Spacing::Px(n));
    }
    Err(ClassParseError {
        token: s.to_string(),
        message: "expected `xs`/`sm`/`md`/`lg`/`xl`/`xxl` or a number (e.g. `gap-3`)".to_string(),
    })
}

fn parse_inset(s: &str) -> Result<Inset, ClassParseError> {
    let named: &[(&str, Inset)] = &[
        ("xs", Inset::Xs),
        ("sm", Inset::Sm),
        ("md", Inset::Md),
        ("lg", Inset::Lg),
        ("xl", Inset::Xl),
    ];
    if let Some((_, v)) = named.iter().find(|(k, _)| *k == s) {
        return Ok(*v);
    }
    if let Ok(n) = s.parse::<f32>() {
        return Ok(Inset::Px(n));
    }
    Err(ClassParseError {
        token: s.to_string(),
        message: "expected `xs`/`sm`/`md`/`lg`/`xl` or a number (e.g. `p-4`)".to_string(),
    })
}

// We re-export `Length::Full` for `w-full` / `h-full` — those
// tokens are handled in the fixed table above, so this helper
// only needs to be used for explicit `Length`-typed values.
#[allow(dead_code)]
pub(crate) fn parse_length(s: &str) -> Result<Length, ClassParseError> {
    match s {
        "full" => Ok(Length::Full),
        "fit" => Ok(Length::Fit),
        "auto" => Ok(Length::Auto),
        _ => Err(ClassParseError {
            token: s.to_string(),
            message: "expected `full`, `fit`, or `auto`".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixed_classes() {
        let r = parse_class_string("flex flex-col items-center w-full").unwrap();
        assert_eq!(
            r,
            vec![
                LayoutClass::Flex,
                LayoutClass::FlexCol,
                LayoutClass::ItemsCenter,
                LayoutClass::WFull,
            ]
        );
    }

    #[test]
    fn parses_value_classes() {
        let r = parse_class_string("gap-md p-sm px-3 m-lg").unwrap();
        assert_eq!(
            r,
            vec![
                LayoutClass::Gap(Spacing::Md),
                LayoutClass::P(Inset::Sm),
                LayoutClass::Px(Spacing::Px(3.0)),
                LayoutClass::M(Inset::Lg),
            ]
        );
    }

    #[test]
    fn unknown_token_is_error() {
        let err = parse_class_string("flex totally-made-up").unwrap_err();
        // The Display impl wraps the token in backticks.
        assert_eq!(format!("{err}").contains("totally-made-up"), true);
        assert_eq!(err.token, "totally-made-up");
    }

    #[test]
    fn mixed_whitespace() {
        let r = parse_class_string("  flex   gap-md  ").unwrap();
        assert_eq!(r, vec![LayoutClass::Flex, LayoutClass::Gap(Spacing::Md)]);
    }
}