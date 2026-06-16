use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::{AstElement, AstNode};
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::{
    ExtraArgKind, LeafDef, PropValue, RenderMode, is_known_shorthand_method, is_spacing_prefix,
    is_spacing_shorthand,
};

use crate::codegen::{
    attr::{attr_expr_only, attr_value_tokens, prop_value_tokens},
    codegen_child, codegen_child_unwrapped,
    color::parse_hex_color,
    container::apply_container_attr,
    control_flow::codegen_if_chain,
    diagnostics::did_you_mean,
    events::{
        auto_wrap_callback_expr, auto_wrap_event_call, auto_wrap_event_expr, split_event_modifiers,
        wrap_event_body_with_modifiers,
    },
    parse_ts,
    text::{extract_text_content, text_attr_value},
};

pub(crate) fn codegen_leaf(
    element: &AstElement,
    def: LeafDef,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
    wrap_to_any: bool,
) -> Result<TokenStream, XmlError> {
    let _ = location; // Currently unused — byte_offset lives on AST nodes.
    // 1. Resolve the id (first factory arg).
    let id_attr = element
        .attributes
        .iter()
        .find(|a| a.name == "id")
        .ok_or_else(|| {
            XmlError::new(
                XmlErrorKind::UnknownAttribute,
                element.span,
                format!("<{}> requires an `id` attribute", element.tag),
            )
            .at(element.byte_offset)
        })?;
    let id_expr = attr_value_tokens(id_attr)?;

    // 2. Build the factory call head.
    //
    // The factory signature is always one of:
    //   `factory(id, [extra_args…], [cx])`
    // where `cx` is present iff `def.needs_app`. We build
    // the positional arg list by:
    //   1. Inserting the resolved `id` value.
    //   2. Resolving every entry in `def.extra_args` (in
    //      declaration order) and inserting the value.
    //   3. Optionally appending `cx`.
    let mut factory_args: Vec<TokenStream> = Vec::new();
    factory_args.push(id_expr);

    for extra in def.extra_args {
        let extra_attr = element.attributes.iter().find(|a| a.name == extra.attr);
        let extra_tokens: TokenStream = match (extra.kind, extra_attr) {
            (ExtraArgKind::Text, Some(a)) => text_attr_value(a)?,
            (ExtraArgKind::Text, None) => {
                // Fall back to inner text content.
                let text = extract_text_content(&element.children).ok_or_else(|| {
                    XmlError::new(
                        XmlErrorKind::UnknownAttribute,
                        element.span,
                        format!(
                            "<{}> needs a `{}` attribute or text content",
                            element.tag, extra.attr
                        ),
                    )
                    .at(element.byte_offset)
                })?;
                quote! { (#text).to_string() }
            }
            (ExtraArgKind::Borrow, Some(a)) => {
                // Borrowed positional arg (e.g. `FocusRing`'s
                // `handle: &FocusHandle`). Pass the expression
                // verbatim; the user is expected to supply a
                // reference such as `handle={&cx.focus_handle()}`.
                attr_value_tokens(a)?
            }
            (ExtraArgKind::Borrow, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::Custom, Some(a)) => attr_value_tokens(a)?,
            (ExtraArgKind::Custom, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::StringList, Some(a)) => {
                if a.expr.is_some() {
                    attr_value_tokens(a)?
                } else {
                    return Err(XmlError::new(
                        XmlErrorKind::InvalidExpression,
                        a.span,
                        format!(
                            "<{}> attribute `{}` requires a brace expression, e.g. `{}={{vec![\"Cmd\".into(), \"S\".into()]}}`",
                            element.tag, extra.attr, extra.attr
                        ),
                    )
                    .at(a.byte_offset));
                }
            }
            (ExtraArgKind::StringList, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::Callback, Some(a)) => {
                let expr = attr_value_tokens(a)?;
                auto_wrap_callback_expr(a, expr)
            }
            (ExtraArgKind::Callback, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::UInt, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    let value = raw.parse::<usize>().map_err(|_| {
                        XmlError::new(
                            XmlErrorKind::InvalidExpression,
                            a.span,
                            format!(
                                "attribute `{}` expects a usize literal, got `{raw}`",
                                a.name
                            ),
                        )
                        .at(a.byte_offset)
                    })?;
                    let lit = proc_macro2::Literal::usize_unsuffixed(value);
                    quote! { #lit }
                }
            }
            (ExtraArgKind::UInt, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::HeadingLevel, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    let variant = match raw {
                        "H1" | "h1" => "H1",
                        "H2" | "h2" => "H2",
                        "H3" | "h3" => "H3",
                        "H4" | "h4" => "H4",
                        "H5" | "h5" => "H5",
                        "H6" | "h6" => "H6",
                        other => {
                            return Err(XmlError::new(
                                XmlErrorKind::InvalidExpression,
                                a.span,
                                format!("attribute `{}` expects H1..H6, got `{other}`", a.name),
                            )
                            .at(a.byte_offset));
                        }
                    };
                    let variant = format_ident!("{variant}");
                    quote! { ::yororen_ui::headless::heading::HeadingLevel::#variant }
                }
            }
            (ExtraArgKind::HeadingLevel, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::IconSource, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    quote! {
                        ::yororen_ui::headless::icon::IconSource::Builtin((#raw).into())
                    }
                }
            }
            (ExtraArgKind::IconSource, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::ImageSource, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    quote! {
                        ::yororen_ui::headless::image::ImageSource::Resource((#raw).into())
                    }
                }
            }
            (ExtraArgKind::ImageSource, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::KeybindingInputMode, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    let raw = a.raw.as_str();
                    let variant = match raw {
                        "Idle" | "idle" => "Idle",
                        "Capturing" | "capturing" => "Capturing",
                        other => {
                            return Err(XmlError::new(
                                XmlErrorKind::InvalidExpression,
                                a.span,
                                format!(
                                    "attribute `{}` expects Idle or Capturing, got `{other}`",
                                    a.name
                                ),
                            )
                            .at(a.byte_offset));
                        }
                    };
                    let variant = format_ident!("{variant}");
                    quote! { ::yororen_ui::headless::keybinding_input::KeybindingInputMode::#variant }
                }
            }
            (ExtraArgKind::KeybindingInputMode, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
            (ExtraArgKind::Color, Some(a)) => {
                if let Some(expr) = &a.expr {
                    parse_ts(
                        expr,
                        a.span,
                        a.byte_offset,
                        &format!("attribute `{}`", a.name),
                    )?
                } else {
                    parse_hex_color(a.raw.as_str(), a)?
                }
            }
            (ExtraArgKind::Color, None) => {
                return Err(XmlError::new(
                    XmlErrorKind::UnknownAttribute,
                    element.span,
                    format!("<{}> requires a `{}` attribute", element.tag, extra.attr),
                )
                .at(element.byte_offset));
            }
        };
        factory_args.push(extra_tokens);
    }

    if def.needs_app {
        factory_args.push(quote! { #cx });
    }

    let factory: TokenStream = parse_ts(
        def.factory,
        element.span,
        element.byte_offset,
        &format!("factory path for <{}>", element.tag),
    )?;

    // Build the leaf as a sequence of statements so trait
    // methods can be fully qualified and the final type is
    // `AnyElement` without relying on the call site to import
    // `IntoElement`, `ParentElement`, etc.
    let mut stmts: Vec<TokenStream> = Vec::new();
    stmts.push(quote! { let mut __el = #factory(#(#factory_args),*); });

    // 3. Apply prop / event setters in declaration order.
    for attr in &element.attributes {
        if attr.name == "id" {
            continue;
        }
        // Attributes consumed by the factory call
        // (the `text` of a `Label`, the `state` of a
        // `Modal`, …) are NOT re-emitted as setters.
        if def.extra_args.iter().any(|e| e.attr == attr.name) {
            continue;
        }
        // Special attribute `@bind={entity}` — the codegen
        // expands it to a one-line two-way binding. The
        // current value is read from the entity, the
        // on_change callback writes back. The user writes:
        //
        //     <TextInput id="x" @bind={self.name} />
        //
        // …and the macro turns it into:
        //
        //     text_input("x", cx)
        //         .value(self.name.read(cx).clone())
        //         .on_change({ let e = self.name.clone();
        //                     move |v, _, cx| e.update(cx, |s, _| *s = v.to_string()) })
        //
        // The exact prop name / event name depend on the
        // component, so we look them up from the schema:
        // the value setter is the first prop of kind
        // `String` whose name is one of `value` / `text`,
        // the event is the first `on_change` event.
        if attr.name == "bind" {
            if let Some(expr) = &attr.expr {
                let parsed = parse_ts(
                    expr,
                    attr.span,
                    attr.byte_offset,
                    "@bind requires a brace expression, e.g. `@bind={self.name}`",
                )?;
                stmts.extend(emit_bind(&parsed, def, cx));
                continue;
            } else {
                return Err(XmlError::new(
                    XmlErrorKind::InvalidExpression,
                    attr.span,
                    "@bind requires a brace expression, e.g. `@bind={self.name}`",
                )
                .at(attr.byte_offset));
            }
        }
        if let Some(prop) = def.props.iter().find(|p| p.name == attr.name).copied() {
            let m = format_ident!("{}", prop.setter);
            match prop.value {
                PropValue::Flag => {
                    // Zero-arg setter (`fn X(self) -> Self`).
                    // The bare-attribute convention is the
                    // trigger: `<Label wrap />` enables it;
                    // `wrap="false"` is a no-op; `wrap={…}` is
                    // a type error (the user is on the hook).
                    if attr.expr.is_some() {
                        return Err(XmlError::new(
                            XmlErrorKind::InvalidExpression,
                            attr.span,
                            format!(
                                "attribute `{}` is a flag (no value) — drop the `={{…}}`",
                                attr.name
                            ),
                        )
                        .at(attr.byte_offset));
                    }
                    let raw = attr.raw.as_str();
                    if raw == "true" {
                        stmts.push(quote! { __el = __el.#m(); });
                    }
                    // `raw == "false"` → skip the call (the
                    // default for unset).
                    continue;
                }
                _ => {
                    let value = prop_value_tokens(attr, prop.value)?;
                    stmts.push(quote! { __el = __el.#m(#value); });
                }
            }
            continue;
        }
        if let Some((_, setter)) = def.events.iter().find(|(n, _)| *n == attr.name).copied() {
            let m = format_ident!("{}", setter);
            // Events take a closure — don't `.into()`.
            // If the user's brace expression is a bare
            // path / field reference (no `(` / `{` / `|`),
            // we auto-wrap it into a closure that adapts
            // the three standard args `(arg0, &mut Window,
            // &mut App)` to whatever the user's method
            // signature is. This lets XML stay purely
            // declarative — the user just writes
            // `on_click={controller.increment}` instead
            // of `move |ev, w, cx| controller.increment(ev, w, cx)`.
            let expr = attr_expr_only(attr)?;
            let expr = auto_wrap_event_expr(attr, expr, setter, &element.tag);
            // Component event setters are inherent methods on
            // the component builder (e.g. `ButtonProps::on_click`),
            // so a normal method call is enough and avoids
            // requiring `StatefulInteractiveElement` to be in
            // scope at the call site.
            stmts.push(quote! {
                __el = __el.#m(#expr);
            });
            continue;
        }
        // Event modifiers: `on_click.stop={...}` /
        // `on_key_down.enter={...}`. The base name is
        // the real event; the modifier list wraps the
        // user's closure in a filter / interceptor.
        if let Some((base_event, modifiers)) = split_event_modifiers(&attr.name)
            && let Some((_, setter)) = def.events.iter().find(|(n, _)| *n == base_event).copied()
        {
            let m = format_ident!("{}", setter);
            let expr = attr_expr_only(attr)?;
            // For modifiers we build the closure body inline
            // rather than wrapping an already-auto-wrapped
            // closure. This keeps the receiver clone outside
            // the `move` closure so the original binding
            // (e.g. `controller`) is not captured and can be
            // reused by other handlers.
            let (clone_stmt, call_expr) = auto_wrap_event_call(attr, expr);
            let body = wrap_event_body_with_modifiers(&modifiers, call_expr, attr.span)?;
            let closure = if let Some(stmt) = clone_stmt {
                quote! {
                    {
                        #stmt
                        move |__ev, __window, cx| { #body }
                    }
                }
            } else {
                quote! {
                    move |__ev, __window, cx| { #body }
                }
            };
            stmts.push(quote! {
                __el = __el.#m(#closure);
            });
            continue;
        }
        // Leaf style pass-through: attributes that look like gpui
        // `Styled` methods (`w_full`, `h={px(16)}`, `border_1`, …)
        // are deferred and applied to the rendered element, so callers
        // can size leaves the same way they size `<Div>` containers.
        if is_leaf_style_attr(&attr.name, &def) {
            continue;
        }

        let mut accepted = accepted_leaf_attrs(&def);
        if let Some(suggestion) = did_you_mean(
            &attr.name,
            &accepted
                .split(", ")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>(),
        ) {
            accepted.push_str(&format!(" — did you mean `{}`?", suggestion));
        }
        return Err(XmlError::new(
            XmlErrorKind::UnknownAttribute,
            attr.span,
            format!(
                "unknown attribute `{}` on <{}>; accepted: {accepted}",
                attr.name, element.tag
            ),
        )
        .at(attr.byte_offset));
    }

    // 4. Named slots (e.g. Popover trigger/content, Tooltip trigger).
    // These builder methods are called before `.render(cx)`.
    let mut slotted_children: Vec<(&str, AstElement)> = Vec::new();
    let mut remaining_children: Vec<AstNode> = Vec::new();
    for child in &element.children {
        if let AstNode::Element(child_el) = child
            && let Some(slot_attr) = child_el.attributes.iter().find(|a| a.name == "slot")
            && let Some(slot_def) = def.slots.iter().find(|s| s.name == slot_attr.raw.as_str())
        {
            let mut stripped = child_el.clone();
            stripped.attributes.retain(|a| a.name != "slot");
            slotted_children.push((slot_def.setter, stripped));
            continue;
        }
        remaining_children.push(child.clone());
    }
    for (setter, child_el) in &slotted_children {
        let setter = format_ident!("{setter}");
        let child_expr = codegen_child(
            &AstNode::Element(child_el.clone()),
            cx,
            location,
            source_file,
        )?;
        stmts.push(quote! { __el = __el.#setter(#child_expr); });
    }

    // 5. Children consumed before render (ButtonGroup, Modal).
    if def.children_before_render {
        for child in &remaining_children {
            match child {
                AstNode::Text { .. } => {
                    return Err(XmlError::new(
                        XmlErrorKind::Unsupported,
                        element.span,
                        format!(
                            "<{tag}> does not support text content; use a <Label> child",
                            tag = element.tag
                        ),
                    )
                    .at(element.byte_offset));
                }
                AstNode::Expr { .. } | AstNode::Element(_) => {
                    let child_expr = if def.unwrap_children {
                        codegen_child_unwrapped(child, cx, location, source_file)?
                    } else {
                        codegen_child(child, cx, location, source_file)?
                    };
                    stmts.push(quote! {
                        __el = __el.child(#child_expr);
                    });
                }
            }
        }
    }

    // 6. Form submit button is built from the props before render,
    //    then attached after render.
    let is_form = element.tag == "Form";
    let has_submit = is_form && element.attributes.iter().any(|a| a.name == "submit");
    if has_submit {
        stmts.push(quote! { let __form_submit_btn = __el.submit_button(&mut *#cx); });
    }

    // 7. Apply render mode.
    // After `.render(...)` the type changes from the
    // component's props/builder to `AnyElement`, so we
    // shadow `__el` with a fresh `let` rather than
    // reassigning (which would fix the original type).
    match def.render {
        RenderMode::Default => {
            // The render method typically takes `(&App)`; a
            // few components (e.g. `TextInput`) also
            // need a `&mut Window`. The schema's
            // `needs_window` flag tells us which.
            if def.needs_window {
                // Both `cx` and `window` are expected
                // as `&mut App` / `&mut Window` by the
                // renderer's `render` signature.
                stmts.push(quote! { let __el = __el.render(&mut *#cx, &mut *window); });
            } else {
                let app_ref = quote! { &*#cx };
                stmts.push(quote! { let __el = __el.render(#app_ref); });
            }
        }
        RenderMode::Apply => {
            // Caller is responsible for `.apply(div())` — for
            // now, do nothing.
        }
    }

    // 8. Style pass-through for known `Styled` attributes.
    //    These are applied after `.render(cx)` so they affect
    //    the final rendered element (e.g. `<Spacer w_full h={px(16)}/>`).
    if def.render == RenderMode::Default {
        let style_container_def = crate::schema::ContainerDef {
            fixed_methods: &[],
            style_hint: "the gpui Styled trait (`.w(...)`, `.h(...)`, `.border_1()`, …)",
        };
        for attr in &element.attributes {
            if !is_leaf_style_attr(&attr.name, &def) {
                continue;
            }
            apply_container_attr(&mut stmts, attr, style_container_def, element)?;
        }
    }

    // 9. Children added after render (the default path).
    // For Default-rendered leaves, the rendered element is
    // typically a `Stateful<Div>` that accepts `.child(...)`.
    // Text children are only allowed for components that explicitly
    // opt in (e.g. `<Button>Click me</Button>`); other text inside a
    // leaf is an error.
    if def.render == RenderMode::Default
        && !def.children_before_render
        && !remaining_children.is_empty()
    {
        let text_opt = extract_text_content(&remaining_children);
        let mut text_added = false;
        let mut child_stmts: Vec<TokenStream> = Vec::new();
        let is_if_element = |node: &AstNode| -> bool {
            matches!(
                node,
                AstNode::Element(e) if matches!(e.tag.as_str(), "If" | "ElseIf" | "Else")
            )
        };
        let mut i = 0;
        while i < remaining_children.len() {
            if is_if_element(&remaining_children[i]) {
                // Merge consecutive If/ElseIf/Else siblings into a
                // single Rust if/else chain, just like containers do.
                let mut j = i;
                while j < remaining_children.len() && is_if_element(&remaining_children[j]) {
                    j += 1;
                }
                let chain_expr =
                    codegen_if_chain(&remaining_children[i..j], cx, location, source_file)?;
                child_stmts.push(quote! {
                    let __el = ::gpui::ParentElement::child(__el, #chain_expr);
                });
                i = j;
            } else {
                let child = &remaining_children[i];
                match child {
                    AstNode::Text { .. } => {
                        if def.supports_text_child {
                            if let Some(text) = &text_opt
                                && !text_added
                            {
                                text_added = true;
                                child_stmts.push(quote! {
                                    let __el = ::gpui::ParentElement::child(__el, #text);
                                });
                            }
                        } else {
                            return Err(XmlError::new(
                                XmlErrorKind::Unsupported,
                                element.span,
                                format!(
                                    "<{tag}> does not support text content; wrap text in a <Label>",
                                    tag = element.tag
                                ),
                            )
                            .at(element.byte_offset));
                        }
                    }
                    AstNode::Expr { .. } | AstNode::Element(_) => {
                        let child_expr = codegen_child(child, cx, location, source_file)?;
                        child_stmts.push(quote! {
                            let __el = ::gpui::ParentElement::child(__el, #child_expr);
                        });
                    }
                }
                i += 1;
            }
        }
        stmts.extend(child_stmts);
    }

    // 9. Attach the Form submit button after render.
    if has_submit {
        stmts.push(quote! {
            let __el = if let Some(__btn) = __form_submit_btn {
                ::gpui::ParentElement::child(__el, __btn)
            } else {
                __el
            };
        });
    }

    // 10. Wrap to AnyElement so the result composes into a parent.
    // When called for an unwrapped child (e.g. ButtonGroup children),
    // leave the concrete leaf type (`Stateful<Div>`) so the parent
    // builder's `.child()` receives the right argument.
    if wrap_to_any {
        stmts.push(quote! { ::gpui::IntoElement::into_any_element(__el) });
    } else {
        stmts.push(quote! { __el });
    }

    Ok(quote! {
        {
            #(#stmts)*
        }
    })
}
/// appends both calls to the props builder.
pub(crate) fn emit_bind(entity: &TokenStream, def: LeafDef, cx: &TokenStream) -> Vec<TokenStream> {
    // Pick the value prop. Prefer `value` (TextInput,
    // SearchInput, NumberInput, …); fall back to
    // `checked` (Checkbox, Switch, ToggleButton); then
    // `text` (Label-like). If none of these exist, the
    // read side is skipped — the entity's current value
    // is read on each render anyway.
    let value_prop = def
        .props
        .iter()
        .find(|p| p.name == "value")
        .or_else(|| def.props.iter().find(|p| p.name == "checked"))
        .or_else(|| def.props.iter().find(|p| p.name == "selected"))
        .or_else(|| def.props.iter().find(|p| p.name == "text"));
    // Pick the change event. Prefer `on_change`; fall
    // back to `on_toggle` for boolean-style components.
    let change_event = def
        .events
        .iter()
        .find(|(n, _)| *n == "on_change")
        .or_else(|| def.events.iter().find(|(n, _)| *n == "on_toggle"));

    let mut out: Vec<TokenStream> = Vec::new();
    if let Some(prop) = value_prop {
        let m = format_ident!("{}", prop.setter);
        // Read the current value via the `XmlBinding` trait
        // — the blanket `impl<T: Clone> XmlBinding<T> for
        // Entity<T>` handles the common case, and user
        // impls route through the same call site. We clone
        // the entity so the original binding in the user's
        // scope isn't moved.
        out.push(quote! {
            __el = __el.#m({
                let __bind = (#entity).clone();
                ::yororen_ui::headless::XmlBinding::xml_read(&__bind, #cx)
            });
        });
    }
    if let Some((event_attr, setter)) = change_event {
        let m = format_ident!("{}", setter);
        // Pick the closure signature based on the event
        // name. on_change takes `(&str, &mut Window,
        // &mut App)` for text inputs and `(f64, &mut Window,
        // &mut App)` for number inputs; on_toggle takes
        // `(bool, Option<&ClickEvent>, &mut Window,
        // &mut App)`. We use the value setter's type
        // (Float → f64, anything else → String) to pick
        // the right `XmlBinding<T>` instantiation.
        let event_name = *event_attr;
        let value_is_f32 = matches!(value_prop.map(|p| p.value), Some(PropValue::Float32));
        let value_is_f64 = matches!(value_prop.map(|p| p.value), Some(PropValue::Float64));
        let value_is_usize = matches!(value_prop.map(|p| p.value), Some(PropValue::UInt));
        let writeback = if event_name == "on_toggle" {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: bool, _ev: Option<&gpui::ClickEvent>, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<bool>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else if value_is_f64 {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: f64, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<f64>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else if value_is_f32 {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: f32, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<f32>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else if value_is_usize {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: usize, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        ::yororen_ui::headless::XmlBinding::<usize>::xml_write(&__bind, __v, cx);
                    }
                });
            }
        } else {
            quote! {
                __el = __el.#m({
                    let __bind = (#entity).clone();
                    move |__v: &str, _window: &mut gpui::Window, cx: &mut gpui::App| {
                        let __new: String = __v.to_string();
                        ::yororen_ui::headless::XmlBinding::<String>::xml_write(&__bind, __new, cx);
                    }
                });
            }
        };
        out.push(writeback);
    }
    out
}
pub(crate) fn is_leaf_style_attr(name: &str, def: &LeafDef) -> bool {
    let is_style =
        is_known_shorthand_method(name) || is_spacing_prefix(name) || is_spacing_shorthand(name);
    if !is_style {
        return false;
    }
    if name == "id" {
        return false;
    }
    if def.extra_args.iter().any(|e| e.attr == name) {
        return false;
    }
    if def.props.iter().any(|p| p.name == name) {
        return false;
    }
    if def.events.iter().any(|e| e.0 == name) {
        return false;
    }
    true
}
pub(crate) fn accepted_leaf_attrs(def: &LeafDef) -> String {
    let mut parts = vec!["id".to_string()];
    for e in def.extra_args {
        parts.push(e.attr.to_string());
    }
    for p in def.props {
        parts.push(p.name.to_string());
    }
    for (name, _) in def.events {
        parts.push(name.to_string());
    }
    for s in def.slots {
        parts.push(format!("slot=\"{}\"", s.name));
    }
    parts.join(", ")
}
