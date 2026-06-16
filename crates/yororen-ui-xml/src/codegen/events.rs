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
/// Detection: if the brace expression has no `(`, `{`, or
/// `|`, it's a bare identifier path / field reference —
/// we wrap it. Otherwise we pass it through verbatim
/// (the user wrote their own closure).
///
/// **Receiver cloning**: for `obj.method` (an
/// `Expr::Field`), we inject `.clone()` between the
/// receiver and the method call so multiple event
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
    // Decide whether to auto-wrap. We *never* auto-wrap
    // user-supplied closures (they have `{` or `|`),
    // and we *always* auto-wrap bare path expressions
    // (`controller.foo` with no args). The interesting
    // middle case is a call expression like
    // `controller.goto(Section::Actions)` — the user is
    // calling a method whose RETURN VALUE is the event
    // handler. That's a "factory" call (the controller
    // method produces the closure to wire up). The
    // auto-wrap should NOT fire here either; we just
    // pass the call result through and the compiler
    // checks that the result is the right closure type.
    //
    // Concretely: the only expressions we auto-wrap are
    // those that syntactically look like a path / field
    // reference with NO call or closure body. Anything
    // containing `(` / `{` / `|` is the user's code and
    // we pass it through verbatim.
    let looks_like_path = !trimmed.contains('(')
        && !trimmed.contains('{')
        && !trimmed.contains('|')
        && !trimmed.is_empty();
    if !looks_like_path {
        return expr;
    }
    // Parse the expression so we can detect a
    // field-access (`controller.method`) and pre-clone
    // the receiver outside the closure. Pre-cloning
    // (rather than `.clone()` inside the body) lets
    // multiple event handlers in the same XML share a
    // single `controller` — each closure captures its
    // own clone and the original `controller` is left
    // available for the next handler.
    let Ok(parsed) = syn::parse_str::<syn::Expr>(trimmed) else {
        return quote! {
            move |#params| {
                #expr(#call_args)
            }
        };
    };
    match parsed {
        // Associated function (`Module::function`) —
        // no receiver, no clone needed.
        syn::Expr::Path(_) => quote! {
            move |#params| {
                #expr(#call_args)
            }
        },
        // `controller.method(args)` — a method call whose
        // result is itself the event handler (a closure
        // factory: `goto(Section::Actions) -> impl Fn(...)`).
        // Pass the call result through verbatim; the
        // receiver is cloned inline so the closure can
        // move it. We don't auto-wrap into a 3-arg
        // closure because the call has its own argument
        // list and the resulting value IS already a
        // closure.
        syn::Expr::Call(call) => {
            let func = call.func;
            // The function being called: clone its
            // receiver once, so the inline call can use
            // the owned value.
            match &*func {
                syn::Expr::Field(field) => {
                    let receiver = &field.base;
                    let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
                    let member = &field.member;
                    let args = call.args.iter();
                    quote! {
                        {
                            let #clone_ident = (#receiver).clone();
                            #clone_ident.#member(#(#args),*)
                        }
                    }
                }
                _ => {
                    // Path-style call (`my_func(args)`).
                    // Pass the result through directly.
                    quote! { #expr }
                }
            }
        }
        // `controller.method` — bare field access. Wrap
        // into the right closure shape for the event.
        syn::Expr::Field(field) => {
            let receiver = field.base;
            let member = field.member;
            // `Span::mixed_site()` yields a unique span
            // per call, so every auto-wrapped closure
            // gets a distinct `__auto_clone_N` ident
            // (proc-macro hygiene).
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
        // Method call, deref, closure literal, etc. —
        // the user wrote their own expression; pass it
        // through verbatim. The compiler will reject it
        // if the type doesn't match the setter's bound.
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
    let looks_like_path = !trimmed.contains('(')
        && !trimmed.contains('{')
        && !trimmed.contains('|')
        && !trimmed.is_empty();
    if !looks_like_path {
        return (None, quote! { #expr(__ev, __window, cx) });
    }
    let Ok(parsed) = syn::parse_str::<syn::Expr>(trimmed) else {
        return (None, quote! { #expr(__ev, __window, cx) });
    };

    let clone_ident = format_ident!("__auto_clone", span = Span::mixed_site());
    match parsed {
        // `controller.method(args)` — a method call that
        // returns an event handler closure. Clone the
        // receiver, then call the method and immediately
        // invoke the returned closure with the event args.
        syn::Expr::Call(call) => match &*call.func {
            syn::Expr::Field(field) => {
                let receiver = &field.base;
                let member = &field.member;
                let args = call.args.iter();
                let clone = quote! { let #clone_ident = (#receiver).clone(); };
                let call = quote! { #clone_ident.#member(#(#args),*)(__ev, __window, cx) };
                (Some(clone), call)
            }
            _ => (None, quote! { #expr(__ev, __window, cx) }),
        },
        // `controller.method` — bare field access. Clone the
        // receiver, then call the method with the event args.
        syn::Expr::Field(field) => {
            let receiver = field.base;
            let member = field.member;
            let clone = quote! { let #clone_ident = (#receiver).clone(); };
            let call = quote! { #clone_ident.#member(__ev, __window, cx) };
            (Some(clone), call)
        }
        // Associated function or bare path — no receiver.
        _ => (None, quote! { #expr(__ev, __window, cx) }),
    }
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
