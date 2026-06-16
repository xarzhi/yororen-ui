use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::AstElement;
use crate::error::{XmlError, XmlErrorKind};
use crate::schema::ComponentDef;

use crate::codegen::{
    attr::{attr_expr_only, parse_let_bindings, require_attr_expr},
    codegen_child, codegen_children_as_element,
    container::apply_container_attr,
    events::auto_wrap_event_expr,
    parse_ts,
};

pub(crate) fn codegen_virtual_list(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    codegen_virtual_list_kind(
        element,
        cx,
        location,
        source_file,
        VirtualListKind::Heterogeneous,
        user_schema,
    )
}
pub(crate) fn codegen_uniform_virtual_list(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    codegen_virtual_list_kind(
        element,
        cx,
        location,
        source_file,
        VirtualListKind::Uniform,
        user_schema,
    )
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VirtualListKind {
    Heterogeneous,
    Uniform,
}

/// Shared body for `<VirtualList>` and `<UniformVirtualList>`.
/// The `kind` selects the headless factory (`virtual_list` vs
/// `uniform_virtual_list`), the controller type
/// (`VirtualListController` vs `UniformVirtualListController`),
/// and whether `on_visible_range_change` is accepted.
///
/// Two row modes are supported:
/// - **Explicit** (`item_count={n} let:index={i}`): the children
///   are a row *template*, re-invoked per visible index. The
///   row body references the bound `index` / `item` idents.
/// - **Children-as-rows** (no `item_count`): each direct child
///   *is* a row, and `item_count` is the number of children.
///   The codegen emits a `match ix { 0 => {child0}, … }` so
///   off-screen rows are never built. This is the mode that
///   composes with `<Include>` — e.g. a list of section files
///   becomes a virtualized section scroller with no Rust glue.
pub(crate) fn codegen_virtual_list_kind(
    element: &AstElement,
    cx: &TokenStream,
    location: &crate::parser::LocationTracker<'_>,
    source_file: Option<&str>,
    kind: VirtualListKind,
    user_schema: &[ComponentDef],
) -> Result<TokenStream, XmlError> {
    // Attributes that belong to the virtual-list itself; every
    // other attribute is treated as a style pass-through applied
    // to the rendered element (e.g. `size_full`, `w`, `h`).
    pub(crate) const VL_ATTRS: &[&str] = &[
        "id",
        "item_count",
        "overdraw",
        "alignment",
        "on_visible_range_change",
        "let_item",
        "let_index",
        "controller",
        "row",
    ];

    // --- required attributes ---------------------------------------
    let id_attr = require_attr_expr(element, "id")?;
    // `id` may be a literal (`id="foo"`) or a brace expr. We
    // support both: a literal becomes `("foo").to_string()` so
    // it coerces to `Into<ElementId>`; a brace expr passes
    // through verbatim.
    let id_expr = match &id_attr.expr {
        Some(e) => parse_ts(e, id_attr.span, id_attr.byte_offset, "<VirtualList> id")?,
        None => {
            let raw = &id_attr.raw;
            quote! { (#raw).to_string() }
        }
    };

    // --- detect row mode -------------------------------------------
    // Three mutually-exclusive modes:
    //   1. Explicit `row={closure}` + `item_count` — the caller
    //      supplies a ready-made row closure (useful when the row
    //      body needs controller state that must not be captured
    //      from the surrounding scope). Children are not allowed
    //      in this mode.
    //   2. `item_count` + children — the children become the row
    //      template, re-invoked per visible index.
    //   3. Children only — each direct child is one row.
    let count_attr = element.attributes.iter().find(|a| a.name == "item_count");
    let row_attr = element.attributes.iter().find(|a| a.name == "row");
    let (count_expr, row_closure, _item_bind, _index_ident) = match (row_attr, count_attr) {
        (Some(row_attr), Some(count_attr)) => {
            let count_expr = attr_expr_only(count_attr)?;
            let row_expr = attr_expr_only(row_attr)?;
            (
                count_expr,
                quote! { #row_expr },
                quote! {},
                format_ident!("index"),
            )
        }
        (Some(_), None) => {
            return Err(XmlError::new(
                XmlErrorKind::Unsupported,
                element.span,
                format!(
                    "<{}> `row={{...}}` requires an `item_count={{...}}` attribute",
                    element.tag
                ),
            )
            .at(element.byte_offset));
        }
        (None, Some(count_attr)) => {
            // Explicit mode: row template re-invoked per index.
            let count_expr = attr_expr_only(count_attr)?;
            if element.children.is_empty() {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    element.span,
                    format!(
                        "<{}> with `item_count` must contain at least one child (the row template)",
                        element.tag
                    ),
                )
                .at(element.byte_offset));
            }
            let (item_ident, index_ident) = parse_let_bindings(element, "item", "index");
            let child_body = codegen_children_as_element(
                &element.children,
                cx,
                location,
                source_file,
                user_schema,
            )?;
            // Bind `let item = index;` when `let:item` is requested —
            // gpui hands the row closure a `usize`, so for parity with
            // `<For>` we make the item binding an alias of the index.
            let item_bind = item_ident
                .map(|it| quote! { let #it: usize = #index_ident; })
                .unwrap_or_default();
            let row_closure = quote! {
                move |#index_ident: usize, window: &mut ::gpui::Window, cx: &mut ::gpui::App| -> ::gpui::AnyElement {
                    #item_bind
                    ::gpui::IntoElement::into_any_element(#child_body)
                }
            };
            (count_expr, row_closure, quote! {}, index_ident)
        }
        (None, None) => {
            // Children-as-rows mode: each direct child is one row.
            if element.children.is_empty() {
                return Err(XmlError::new(
                    XmlErrorKind::Unsupported,
                    element.span,
                    format!(
                        "<{}> requires either an `item_count={{...}}` attribute or at least one child row",
                        element.tag
                    ),
                )
                .at(element.byte_offset));
            }
            // One row per child. The index ident is internal to
            // the generated match; the user's children don't see it.
            let arms: Vec<TokenStream> = element
                .children
                .iter()
                .enumerate()
                .map(|(i, child)| {
                    let body = codegen_child(child, cx, location, source_file, user_schema)?;
                    let lit = i;
                    Ok::<_, XmlError>(quote! { #lit => { #body } })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let n = element.children.len();
            let count_expr = quote! { #n };
            let index_ident = format_ident!("__ix");
            let child_body = quote! {
                match #index_ident {
                    #(#arms)*
                    _ => ::gpui::div(),
                }
            };
            let row_closure = quote! {
                move |#index_ident: usize, window: &mut ::gpui::Window, cx: &mut ::gpui::App| -> ::gpui::AnyElement {
                    ::gpui::IntoElement::into_any_element(#child_body)
                }
            };
            (count_expr, row_closure, quote! {}, index_ident)
        }
    };

    // Explicit `row=` and children are mutually exclusive: mode 1
    // already provides the full row closure, so any children would
    // be silently ignored. Reject instead of dropping them.
    if row_attr.is_some() && !element.children.is_empty() {
        return Err(XmlError::new(
            XmlErrorKind::Unsupported,
            element.span,
            format!(
                "<{}> `row={{...}}` cannot be used together with children; \
                 remove the children or use `item_count={{...}}` with children instead",
                element.tag
            ),
        )
        .at(element.byte_offset));
    }

    // --- optional attributes ---------------------------------------
    let overdraw_expr = match element.attributes.iter().find(|a| a.name == "overdraw") {
        Some(a) => attr_expr_only(a)?,
        None => quote! { ::gpui::px(16.) },
    };
    // `alignment="top" | "bottom"` (heterogeneous only);
    // uniform lists have no alignment prop on the headless
    // factory, so we ignore it for that kind.
    let alignment_tokens = if kind == VirtualListKind::Heterogeneous {
        match element.attributes.iter().find(|a| a.name == "alignment") {
            Some(a) => {
                let raw = a.raw.as_str();
                let variant = match raw {
                    "bottom" => quote! { ::gpui::ListAlignment::Bottom },
                    "top" | "" => quote! { ::gpui::ListAlignment::Top },
                    _ => {
                        return Err(XmlError::new(
                            XmlErrorKind::Unsupported,
                            a.span,
                            format!(
                                "<{}> alignment must be \"top\" or \"bottom\", got `{raw}`",
                                element.tag
                            ),
                        )
                        .at(a.byte_offset));
                    }
                };
                Some(variant)
            }
            None => Some(quote! { ::gpui::ListAlignment::Top }),
        }
    } else {
        None
    };

    let on_range_tokens = if kind == VirtualListKind::Heterogeneous {
        match element
            .attributes
            .iter()
            .find(|a| a.name == "on_visible_range_change")
        {
            Some(a) => {
                let expr = attr_expr_only(a)?;
                Some(auto_wrap_event_expr(
                    a,
                    expr,
                    "on_visible_range_change",
                    "VirtualList",
                ))
            }
            None => None,
        }
    } else {
        None
    };

    let controller_attr = element.attributes.iter().find(|a| a.name == "controller");
    let controller_expr = match controller_attr {
        Some(a) => Some(attr_expr_only(a)?),
        None => None,
    };

    // --- style pass-through ----------------------------------------
    // Any attribute that isn't a virtual-list attribute is
    // applied to the rendered element (e.g. `size_full`, `h`,
    // `w`, `flex_grow`). This is the same vocabulary the
    // container codegen accepts, so we reuse its dispatcher.
    let style_container_def = crate::schema::ContainerDef {
        fixed_methods: &[],
        style_hint: "the gpui Styled trait (`.size_full()`, `.h(...)`, …)",
    };
    let mut style_stmts: Vec<TokenStream> = Vec::new();
    for attr in &element.attributes {
        if VL_ATTRS.contains(&attr.name.as_str()) {
            continue;
        }
        apply_container_attr(&mut style_stmts, attr, style_container_def, element)?;
    }

    // --- factory / controller paths --------------------------------
    let (factory, controller_ty, entity_state_init) = match kind {
        VirtualListKind::Heterogeneous => {
            let init = quote! {
                ::yororen_ui::headless::virtual_list::VirtualListController::new(
                    #count_expr as usize,
                    #alignment_tokens,
                    #overdraw_expr,
                )
            };
            (
                quote! { ::yororen_ui::headless::virtual_list::virtual_list },
                quote! { ::yororen_ui::headless::virtual_list::VirtualListController },
                init,
            )
        }
        VirtualListKind::Uniform => {
            // The uniform factory takes item_count positionally
            // each frame (it is not stored on the controller),
            // so we pass `count_expr` at the call site below
            // rather than in the init.
            let init = quote! {
                ::yororen_ui::headless::virtual_list::UniformVirtualListController::new()
            };
            (
                quote! { ::yororen_ui::headless::virtual_list::uniform_virtual_list },
                quote! { ::yororen_ui::headless::virtual_list::UniformVirtualListController },
                init,
            )
        }
    };

    // --- emit ------------------------------------------------------
    // Heterogeneous: the factory signature is
    //   virtual_list(id, &controller, cx) -> VirtualListProps,
    // and we sync item_count via controller.reset every frame.
    // Uniform: the factory signature is
    //   uniform_virtual_list(id, item_count, &controller, cx)
    //   -> UniformVirtualListProps,
    // so count goes in at the call site and no reset is needed.
    let (factory_call, count_sync) = match kind {
        VirtualListKind::Heterogeneous => (
            quote! { #factory(#id_expr, &__snap, #cx) },
            quote! {
                // Evaluate the target count *outside* `update` so the
                // closure does not immutably borrow `cx` while the
                // `Entity::update` call already holds a mutable borrow.
                let __target_count = (#count_expr as usize);
                __entity.update(#cx, |__c: &mut #controller_ty, _| {
                    let __current = __c.state().item_count();
                    if __target_count > __current {
                        // Grow via append so the scroll position is
                        // preserved (infinite-loading behaviour).
                        __c.append(__target_count - __current);
                    } else if __target_count < __current {
                        __c.reset(__target_count);
                    }
                });
            },
        ),
        VirtualListKind::Uniform => (
            quote! { #factory(#id_expr, #count_expr as usize, &__snap, #cx) },
            quote! {},
        ),
    };

    let on_range_stmt = match on_range_tokens {
        Some(t) => quote! { __props = __props.on_visible_range_change(#t); },
        None => quote! {},
    };

    // `use_keyed_state` is an inherent method on `gpui::Window`.
    // It persists the controller as an `Entity<T>` keyed by the
    // element id across consecutive frames and auto-observes
    // mutations (so a `.reset()` / `.scroll_to_*` triggers a
    // re-render of the owning view). We clone the controller
    // out of the entity for the per-frame factory call.
    //
    // Alternatively, the caller can supply an external entity via
    // `controller={...}` — useful when buttons outside the list need
    // to drive scroll position or when the controller is shared with
    // business state.
    let entity_init = match &controller_expr {
        Some(expr) => quote! { let __entity = (#expr).clone(); },
        None => quote! {
            let __entity = window.use_keyed_state(
                #id_expr,
                #cx,
                |_window, _cx| #entity_state_init,
            );
        },
    };

    Ok(quote! {
        {
            #entity_init
            #count_sync
            let __snap = __entity.read(#cx).clone();
            let mut __props = #factory_call;
            __props = __props.row(#row_closure);
            #on_range_stmt
            let mut __el = __props.render(#cx);
            #(#style_stmts)*
            ::gpui::IntoElement::into_any_element(__el)
        }
    })
}
