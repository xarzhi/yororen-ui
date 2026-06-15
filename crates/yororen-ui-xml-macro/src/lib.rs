//! `xml!` / `xml_file!` proc-macros for yororen-ui.
//!
//! See `yororen-ui-xml` (the supporting crate) for the
//! schema / parser / codegen implementation. This crate is a
//! thin wrapper that hands the literal XML text off to the
//! library and converts the resulting `TokenStream` into the
//! `proc_macro::TokenStream` the compiler expects.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Parser, Result as SynResult};

/// Parsed form of `xml! { cx = expr, <Column>...</Column> }`.
///
/// The optional `cx = <expr>,` preamble lets the user thread a
/// `&mut gpui::App` into factory calls. When omitted we use
/// the bare identifier `cx`.
struct XmlArgs {
    cx: Option<syn::Expr>,
    xml: String,
}

impl Parse for XmlArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Optional `cx = <expr>,` preamble.
        let mut cx: Option<syn::Expr> = None;
        if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
            let ident: syn::Ident = input.parse()?;
            if ident != "cx" {
                return Err(syn::Error::new(ident.span(), "expected `cx`"));
            }
            let _eq: syn::Token![=] = input.parse()?;
            cx = Some(input.parse()?);
            let _comma: syn::Token![,] = input.parse()?;
        }
        // The XML literal — anything until the end of the input.
        // We slurp the remaining tokens as a string.
        let xml_tokens = input.to_string();
        Ok(XmlArgs {
            cx,
            xml: xml_tokens,
        })
    }
}

#[proc_macro]
pub fn xml(input: TokenStream) -> TokenStream {
    let parser = |stream: ParseStream| XmlArgs::parse(stream);
    let args = match parser.parse2(input.into()) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    let outer_span = Span::call_site();
    let cx_expr = args.cx.map(|e| quote::quote! { #e });
    // Build the location tracker up-front so we can use it
    // both for parsing/codegen AND for rendering the
    // diagnostic in case either fails. The error returned
    // by `codegen` already carries the byte offset; we
    // just need the table to convert it back to line/col.
    let line_starts = yororen_ui_xml::parser::line_starts(&args.xml);
    let location = yororen_ui_xml::parser::LocationTracker {
        line_starts: &line_starts,
        xml: &args.xml,
        outer_span,
    };
    // The `proc_macro::Span::call_site().file()` call
    // gives the absolute path of the file that invoked
    // this macro. The codegen uses it to resolve relative
    // `<Include src="…">` paths the same way `xml_file!`
    // does (so XML-include and XML-file point to the
    // same file from the same source file).
    let source_file = proc_macro::Span::call_site().file();
    match yororen_ui_xml::codegen::codegen(&args.xml, outer_span, cx_expr, Some(&source_file)) {
        Ok(ts) => ts.into(),
        Err(e) => syn::Error::new(outer_span, e.render_with(Some(&location)))
            .to_compile_error()
            .into(),
    }
}

/// Form of `xml_file!`:
/// - `xml_file!("path.xml")` — implicit `cx`, path resolved
///   relative to the source file that contains the macro call
/// - `xml_file!(cx = &mut **cx, "path.xml")` — explicit `cx`
///   binding, same path resolution
/// - `xml_file!(cx = &mut **cx, window = window, "path.xml")`
///   — also pass `&mut Window` for components whose render
///   method needs it (e.g. `TextInput`, `NumberInput`)
#[proc_macro]
pub fn xml_file(input: TokenStream) -> TokenStream {
    // Parse the leading `cx = <expr>,` (optional) and
    // optional `window = <expr>,` and the trailing string
    // literal path.
    let parser = |stream: ParseStream| -> syn::Result<XmlFileArgs> {
        let mut cx: Option<syn::Expr> = None;
        let mut window: Option<syn::Expr> = None;
        // The leading key=… clauses: `cx`, `window`, in any order,
        // each followed by a comma.
        loop {
            if !stream.peek(syn::Ident) || !stream.peek2(syn::Token![=]) {
                break;
            }
            let ident: syn::Ident = stream.parse()?;
            let _eq: syn::Token![=] = stream.parse()?;
            let expr: syn::Expr = stream.parse()?;
            match ident.to_string().as_str() {
                "cx" => cx = Some(expr),
                "window" => window = Some(expr),
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("expected `cx` or `window`, got `{other}`"),
                    ));
                }
            }
            // Optional trailing comma before the next clause or
            // the path. We always parse a comma when one is
            // present, then peek to see if more clauses follow.
            if stream.peek(syn::Token![,]) {
                let _: syn::Token![,] = stream.parse()?;
                // Continue the loop if the next token is an
                // `ident = …` pair; otherwise fall through to
                // the path parse.
                continue;
            }
            break;
        }
        let path: syn::LitStr = stream.parse()?;
        let call_site = proc_macro::Span::call_site();
        let source_file = call_site.file();
        Ok(XmlFileArgs {
            cx,
            window,
            path: path.value(),
            path_span: path.span(),
            source_file,
        })
    };
    let args = match parser.parse2(input.into()) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    let outer_span = args.path_span;

    // Resolve the path. If it's absolute, use it directly.
    // Otherwise resolve relative to the directory of the
    // source file that invoked the macro (so that
    // `xml_file!("ui/counter.xml")` from
    // `crates/yororen-ui-demos/counter_xml/src/view.rs` finds
    // `crates/yororen-ui-demos/counter_xml/ui/counter.xml`).
    let resolved_path = if std::path::Path::new(&args.path).is_absolute() {
        std::path::PathBuf::from(&args.path)
    } else {
        let source = std::path::Path::new(&args.source_file);
        let dir = source.parent().unwrap_or_else(|| std::path::Path::new("."));
        dir.join(&args.path)
    };

    let contents = match std::fs::read_to_string(&resolved_path) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(
                outer_span,
                format!(
                    "could not read `{}` (resolved to `{}`): {e}",
                    args.path,
                    resolved_path.display()
                ),
            )
            .to_compile_error()
            .into();
        }
    };
    let cx_expr = args.cx.map(|e| quote::quote! { #e });
    let _window_expr = args.window.map(|e| quote::quote! { #e });
    // Build the location tracker from the *file's* content
    // so error messages point to the right file. (Even
    // though the proc-macro's outer span points at the
    // `xml_file!(...)` call, the diagnostic line/col must
    // match the file the user is editing.)
    let line_starts = yororen_ui_xml::parser::line_starts(&contents);
    let location = yororen_ui_xml::parser::LocationTracker {
        line_starts: &line_starts,
        xml: &contents,
        outer_span,
    };
    match yororen_ui_xml::codegen::codegen(
        &contents,
        outer_span,
        cx_expr,
        Some(&args.source_file),
    ) {
        Ok(ts) => ts.into(),
        Err(e) => syn::Error::new(outer_span, e.render_with(Some(&location)))
            .to_compile_error()
            .into(),
    }
}

/// Insert `let window = <expr>;` as the first statement
/// inside the codegen's leading `{ use …; … }` block.
///
/// `ts` is expected to be `{ #[allow(unused_imports)] use
/// …; <body> }`. We rebuild it as `{ use …; let window =
/// <expr>; <body> }` so the generated code can reference
/// `window` for components whose render method takes it.
///
/// Implementation note: we DON'T add another `{ … }` wrap
/// around `ts` — that would create a doubly-nested block
/// and confuse some rustc diagnostics into reading the
/// outer block as a closure.
#[allow(dead_code)]
fn splice_window_let(ts: TokenStream, w_expr: TokenStream) -> TokenStream {
    let ts2: proc_macro2::TokenStream = ts.into();
    let w_expr2: proc_macro2::TokenStream = w_expr.into();
    let block: proc_macro2::TokenStream = quote::quote! {
        {
            #[allow(unused_imports)]
            use ::gpui::{IntoElement, ParentElement, StatefulInteractiveElement, InteractiveElement, Styled};
            let window = #w_expr2;
            #ts2
        }
    };
    block.into()
}

struct XmlFileArgs {
    cx: Option<syn::Expr>,
    window: Option<syn::Expr>,
    path: String,
    path_span: Span,
    source_file: String,
}
