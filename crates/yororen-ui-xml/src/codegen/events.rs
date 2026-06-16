use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use crate::ast::AstAttribute;
use crate::error::{XmlError, XmlErrorKind};

/// Auto-wrap a bare event expression into a closure that
/// adapts the three standard callback args
/// `(arg0, &mut Window, &mut App)` to the user's method
/// signature. This is the heart of the "XML stays pure"
/// convention: the user writes
///
/// ```xml
/// <Button on_click={controller.increment} />
/// ```
///
/// and the codegen emits
///
/// ```ignore
/// .on_click(move |__arg0, __w, __cx| {
///     controller.clone().increment(__arg0, __w, __cx)
/// })
/// ```
///
/// Detection: the raw expression is parsed with
/// `syn::Expr` and classified by AST shape. Bare paths
/// (`Module::function`), field accesses (`obj.method`),
/// and method/factory calls (`obj.method(args)`) are
/// auto-wrapped and have their receiver pre-cloned.
/// User-written closures, blocks, macros, references,
/// and already-invoked call chains are passed through
/// verbatim.
///
/// **Receiver cloning**: for `obj.method` and
/// `obj.method(args)` we emit the receiver clone once,
/// outside the generated closure, so multiple event
/// handlers in the same XML can share a single
/// `controller` instance without `move` conflicts.
/// The user's `controller` type must implement
/// `Clone` (cheap clones are typical — `Arc<_>`,
/// `Entity<_>`, or a small data struct).
///
/// **4-arg events**: `on_toggle` on Checkbox / Switch /
/// Radio / ToggleButton uses `(bool, Option<&ClickEvent>,
/// &mut Window, &mut App)`. For all other components
/// (including `TreeItem`), `on_toggle` is treated as a
/// click-style event: `(&ClickEvent, &mut Window,
/// &mut App)`. Callers writing bare controller methods
/// must ensure the method signature matches the emitted
/// closure shape.
pub(crate) fn auto_wrap_event_expr(
    attr: &AstAttribute,
    expr: TokenStream,
    event_name: &str,
    tag: &str,
) -> TokenStream {
    // Closure shape depends on the event and component tag.
    let (params, call_args): (TokenStream, TokenStream) = match event_name {
        "on_toggle" if matches!(tag, "Checkbox" | "Switch" | "Radio" | "ToggleButton") => (
            quote! { __arg0: bool, __ev: Option<&gpui::ClickEvent>, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __arg0, __ev, __w, __cx },
        ),
        "on_toggle" => (
            // TreeItem and any future non-toggle component:
            // on_toggle is conceptually a click on the row /
            // chevron, so it uses the standard 3-arg click
            // signature. Auto-wrapped bare methods receive
            // `&ClickEvent` as the first argument.
            quote! { __ev: &gpui::ClickEvent, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __ev, __w, __cx },
        ),
        "on_clear" | "on_start_capture" | "on_cancel_capture" => (
            quote! { __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __w, __cx },
        ),
        "on_visible_range_change" => (
            quote! { __range: std::ops::Range<usize>, __total: usize, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __range, __total, __w, __cx },
        ),
        _ => (
            quote! { __arg0, __w: &mut gpui::Window, __cx: &mut gpui::App },
            quote! { __arg0, __w, __cx },
        ),
    };
    auto_wrap_closure_expr(attr, expr, params, call_args)
}
pub(crate) fn auto_wrap_callback_expr(attr: &AstAttribute, expr: TokenStream) -> TokenStream {
    let params = quote! { __arg0, __w: &mut gpui::Window, __cx: &mut gpui::App };
    let call_args = quote! { __arg0, __w, __cx };
    auto_wrap_closure_expr(attr, expr, params, call_args)
}
pub(crate) fn auto_wrap_closure_expr(
    attr: &AstAttribute,
    expr: TokenStream,
    params: TokenStream,
    call_args: TokenStream,
) -> TokenStream {
    let Some(raw) = &attr.expr else {
        return expr;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return expr;
    }
    let Ok(parsed) = syn::parse_str::<syn::Expr>(trimmed) else {
        return quote! { #expr };
    };

    match parsed {
        // Associated function / bare path (`Module::function`) —
        // no receiver, no clone needed.
        syn::Expr::Path(_) => quote! {
            move |#params| {
                #expr(#call_args)
            }
        },
        // `controller.method(args)` — a method call whose result
        // is itself the event handler (a closure factory). Clone
        // the receiver outside the generated closure and pass the
        // factory result through.
        syn::Expr::MethodCall(call) => {
            let receiver = &call.receiver;
            let method = &call.method;
            let args = call.args.iter();
            let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
            quote! {
                {
                    let #clone_ident = (#receiver).clone();
                    #clone_ident.#method(#(#args),*)
                }
            }
        }
        // `(controller.method)(args)` — field access used as a
        // function (unusual syntax, kept for backwards
        // compatibility). Clone the receiver and call the field.
        syn::Expr::Call(call) => {
            if let Some(field) = call_as_field(&call) {
                let receiver = &field.base;
                let member = &field.member;
                let args = call.args.iter();
                let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
                quote! {
                    {
                        let #clone_ident = (#receiver).clone();
                        #clone_ident.#member(#(#args),*)
                    }
                }
            } else {
                // Path-style call (`my_func(args)`) or already
                // invoked chain — pass the result through directly.
                quote! { #expr }
            }
        }
        // `controller.method` — bare field access. Wrap into the
        // right closure shape for the event.
        syn::Expr::Field(field) => {
            let receiver = field.base;
            let member = field.member;
            let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
            quote! {
                {
                    let #clone_ident = (#receiver).clone();
                    move |#params| {
                        #clone_ident.#member(#call_args)
                    }
                }
            }
        }
        // User closures, blocks, macros, references, and other
        // complex expressions — pass through verbatim. The
        // compiler will reject them if the type doesn't match the
        // setter's bound.
        _ => quote! { #expr },
    }
}

pub(crate) fn auto_wrap_event_call(
    attr: &AstAttribute,
    expr: TokenStream,
) -> (Option<TokenStream>, TokenStream) {
    let Some(raw) = &attr.expr else {
        return (None, quote! { #expr(__ev, __window, cx) });
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (None, quote! { #expr(__ev, __window, cx) });
    }
    let Ok(parsed) = syn::parse_str::<syn::Expr>(trimmed) else {
        return (None, quote! { #expr(__ev, __window, cx) });
    };

    let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
    match parsed {
        // `controller.method(args)` — a method call returning a
        // closure factory. Clone the receiver, then invoke the
        // returned closure with the event args.
        syn::Expr::MethodCall(call) => {
            let receiver = &call.receiver;
            let method = &call.method;
            let args = call.args.iter();
            let clone = quote! { let #clone_ident = (#receiver).clone(); };
            let call = quote! { #clone_ident.#method(#(#args),*)(__ev, __window, cx) };
            (Some(clone), call)
        }
        // `(controller.method)(args)` — field access used as a
        // function. Clone receiver and invoke the result.
        syn::Expr::Call(call) => {
            if let Some(field) = call_as_field(&call) {
                let receiver = &field.base;
                let member = &field.member;
                let args = call.args.iter();
                let clone = quote! { let #clone_ident = (#receiver).clone(); };
                let call = quote! { #clone_ident.#member(#(#args),*)(__ev, __window, cx) };
                (Some(clone), call)
            } else if is_chain_call(&call) {
                // Already-invoked chain like
                // `controller.method(args)(extra)` — the user has
                // fully applied the expression, so pass it through
                // verbatim instead of appending event args.
                (None, quote! { #expr })
            } else {
                // Factory call (`some_fn()`) whose result is the
                // event handler. Invoke it with event args.
                (None, quote! { #expr(__ev, __window, cx) })
            }
        }
        // `controller.method` — bare field access.
        syn::Expr::Field(field) => {
            let receiver = field.base;
            let member = field.member;
            let clone = quote! { let #clone_ident = (#receiver).clone(); };
            let call = quote! { #clone_ident.#member(__ev, __window, cx) };
            (Some(clone), call)
        }
        // Associated function / bare path — no receiver.
        syn::Expr::Path(_) => (None, quote! { #expr(__ev, __window, cx) }),
        // User closures, blocks, macros, references, etc.
        _ => (None, quote! { #expr(__ev, __window, cx) }),
    }
}

/// Peel off `Expr::Paren` / `Expr::Group` wrappers so that
/// `(controller.method)(args)` is classified the same as the
/// underlying field access.
fn unwrap_paren_group(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Paren(p) => unwrap_paren_group(&p.expr),
        syn::Expr::Group(g) => unwrap_paren_group(&g.expr),
        _ => expr,
    }
}

/// If `call` is a direct call whose callee is a field access,
/// return that field. Example: `(controller.method)(args)`.
fn call_as_field(call: &syn::ExprCall) -> Option<&syn::ExprField> {
    match unwrap_paren_group(&call.func) {
        syn::Expr::Field(field) => Some(field),
        _ => None,
    }
}

/// True when `call` is part of a call chain whose callee has
/// already been invoked, e.g. `controller.method(args)(extra)`.
fn is_chain_call(call: &syn::ExprCall) -> bool {
    matches!(
        unwrap_paren_group(&call.func),
        syn::Expr::Call(_) | syn::Expr::MethodCall(_) | syn::Expr::Await(_)
    )
}

/// Split an attribute name like `on_click.stop.enter` into
/// `("on_click", vec!["stop", "enter"])`. Returns `None` for
/// names without a dot, signalling that no modifier is
/// present.
///
/// The base name (before the first dot) is what the schema
/// looks up to find the headless event setter; the
/// modifiers drive the runtime wrapper that the macro
/// emits (see [`wrap_event_with_modifiers`]).
pub(crate) fn split_event_modifiers(name: &str) -> Option<(&str, Vec<&str>)> {
    let (base, rest) = name.split_once('.')?;
    if rest.is_empty() {
        return None;
    }
    // Reject double dots and other garbage so the codegen
    // surface a sensible error later.
    if rest.contains("..") || rest.starts_with('.') || rest.ends_with('.') {
        return None;
    }
    let modifiers: Vec<&str> = rest.split('.').collect();
    Some((base, modifiers))
}
pub(crate) fn wrap_event_body_with_modifiers(
    modifiers: &[&str],
    inner_call: TokenStream,
    span: Span,
) -> Result<TokenStream, XmlError> {
    if modifiers.is_empty() {
        return Ok(inner_call);
    }
    let mut body = inner_call;
    for modifier in modifiers.iter().rev() {
        body = match *modifier {
            // `.stop` — ask the platform not to propagate
            // the event further. gpui's `App::stop_propagation`
            // is a flag the dispatcher reads; calling it here
            // before the user's handler runs is the contract.
            "stop" => quote! {
                { cx.stop_propagation(); #body }
            },
            // `.prevent` — ask the platform to skip the
            // default action for the event.
            "prevent" => quote! {
                { __window.prevent_default(); #body }
            },
            // Modifier-key filters. Each maps to a boolean
            // field on `gpui::Modifiers` — the event arg's
            // `.modifiers()` accessor returns one. `.meta`
            // is accepted as an alias for `.platform` (the
            // macOS Command key) because "cmd" / "meta" is
            // the more familiar name on Windows / Linux.
            "ctrl" => wrap_modifier_flag_body(body, "control"),
            "shift" => wrap_modifier_flag_body(body, "shift"),
            "alt" => wrap_modifier_flag_body(body, "alt"),
            "platform" | "meta" | "cmd" => wrap_modifier_flag_body(body, "platform"),
            "secondary" => wrap_modifier_flag_body(body, "secondary"),
            "function" => wrap_modifier_flag_body(body, "function"),
            // Keyboard filters — gate on the keystroke key.
            // `__ev.keystroke().key` returns the printable
            // name (`"enter"`, `"escape"`, `"tab"`, …) which
            // is exactly what the user writes in the XML.
            key => {
                if !is_known_key_filter(key) {
                    return Err(XmlError::new(
                        XmlErrorKind::InvalidExpression,
                        span,
                        format!(
                            "unknown event modifier `{key}`; expected one of `stop`, `prevent`, `ctrl`, `shift`, `alt`, `platform` (alias `meta`/`cmd`), `secondary`, `function`, or a key name like `enter` / `escape` / `tab`"
                        ),
                    ));
                }
                let key_lit = format!("\"{key}\"");
                quote! {
                    if __ev.keystroke().key == #key_lit {
                        #body
                    }
                }
            }
        };
    }
    Ok(body)
}
pub(crate) fn wrap_modifier_flag_body(body: TokenStream, flag: &str) -> TokenStream {
    let flag_ident = format_ident!("{}", flag);
    quote! {
        if __ev.modifiers().#flag_ident {
            #body
        }
    }
}
pub(crate) fn is_known_key_filter(name: &str) -> bool {
    matches!(
        name,
        // Whitespace / editing
        "enter"
        | "escape"
        | "tab"
        | "space"
        | "backspace"
        | "delete"
        // Arrow keys
        | "up"
        | "down"
        | "left"
        | "right"
        // Navigation
        | "home"
        | "end"
        | "pageup"
        | "pagedown"
        // Function keys (F1..F12)
        | "f1" | "f2" | "f3" | "f4" | "f5" | "f6"
        | "f7" | "f8" | "f9" | "f10" | "f11" | "f12"
    )
}
