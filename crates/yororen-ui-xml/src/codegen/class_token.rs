//! Token-stream conversion for `LayoutClass`.
//!
//! Shared between the `classes!` proc-macro
//! (`yororen-ui-xml-macro`) and the XML `class` attribute
//! codegen (`yororen-ui-xml::codegen::leaf`). Lives in
//! `yororen-ui-xml` (not the macro crate) so the leaf
//! codegen can reach it without a circular dependency.

use proc_macro2::TokenStream;
use quote::quote;
use yororen_ui_core::headless::layout::{
    Inset, LayoutClass, Spacing,
};

/// Convert a `LayoutClass` reference into a `TokenStream`
/// that constructs the value, fully qualified with the
/// provided `layout_path` (e.g. `::yororen_ui::headless::layout::LayoutClass`).
pub fn layout_class_to_tokens(c: &LayoutClass, layout_path: &TokenStream) -> TokenStream {
    match c {
        LayoutClass::Flex => quote! { #layout_path::Flex },
        LayoutClass::FlexCol => quote! { #layout_path::FlexCol },
        LayoutClass::FlexRow => quote! { #layout_path::FlexRow },
        LayoutClass::FlexWrap => quote! { #layout_path::FlexWrap },
        LayoutClass::Flex1 => quote! { #layout_path::Flex1 },
        LayoutClass::ItemsStart => quote! { #layout_path::ItemsStart },
        LayoutClass::ItemsEnd => quote! { #layout_path::ItemsEnd },
        LayoutClass::ItemsCenter => quote! { #layout_path::ItemsCenter },
        LayoutClass::ItemsBaseline => quote! { #layout_path::ItemsBaseline },
        LayoutClass::ItemsStretch => quote! { #layout_path::ItemsStretch },
        LayoutClass::JustifyStart => quote! { #layout_path::JustifyStart },
        LayoutClass::JustifyEnd => quote! { #layout_path::JustifyEnd },
        LayoutClass::JustifyCenter => quote! { #layout_path::JustifyCenter },
        LayoutClass::JustifyBetween => quote! { #layout_path::JustifyBetween },
        LayoutClass::JustifyAround => quote! { #layout_path::JustifyAround },
        LayoutClass::JustifyEvenly => quote! { #layout_path::JustifyEvenly },
        LayoutClass::Gap(s) => wrap_variant(spacing_to_tokens(s), layout_path, "Gap"),
        LayoutClass::P(i) => wrap_variant(inset_to_tokens(i), layout_path, "P"),
        LayoutClass::Px(s) => wrap_variant(spacing_to_tokens(s), layout_path, "Px"),
        LayoutClass::Py(s) => wrap_variant(spacing_to_tokens(s), layout_path, "Py"),
        LayoutClass::M(i) => wrap_variant(inset_to_tokens(i), layout_path, "M"),
        LayoutClass::Mx(i) => wrap_variant(inset_to_tokens(i), layout_path, "Mx"),
        LayoutClass::My(i) => wrap_variant(inset_to_tokens(i), layout_path, "My"),
        LayoutClass::WFull => quote! { #layout_path::WFull },
        LayoutClass::HFull => quote! { #layout_path::HFull },
        LayoutClass::SizeFull => quote! { #layout_path::SizeFull },
        LayoutClass::Relative => quote! { #layout_path::Relative },
        LayoutClass::Absolute => quote! { #layout_path::Absolute },
        LayoutClass::Top0 => quote! { #layout_path::Top0 },
        LayoutClass::Right0 => quote! { #layout_path::Right0 },
        LayoutClass::Bottom0 => quote! { #layout_path::Bottom0 },
        LayoutClass::Left0 => quote! { #layout_path::Left0 },
        LayoutClass::Inset0 => quote! { #layout_path::Inset0 },
        LayoutClass::OverflowHidden => quote! { #layout_path::OverflowHidden },
        LayoutClass::OverflowScroll => quote! { #layout_path::OverflowScroll },
        LayoutClass::Border => quote! { #layout_path::Border },
        LayoutClass::Border1 => quote! { #layout_path::Border1 },
        LayoutClass::Rounded => quote! { #layout_path::Rounded },
        LayoutClass::RoundedMd => quote! { #layout_path::RoundedMd },
        LayoutClass::RoundedLg => quote! { #layout_path::RoundedLg },
        LayoutClass::ShadowMd => quote! { #layout_path::ShadowMd },
        LayoutClass::ShadowLg => quote! { #layout_path::ShadowLg },
    }
}

fn wrap_variant(inner: TokenStream, layout_path: &TokenStream, variant: &str) -> TokenStream {
    let v = proc_macro2::Ident::new(variant, proc_macro2::Span::call_site());
    quote! { #layout_path::#v(#inner) }
}

fn spacing_to_tokens(s: &Spacing) -> TokenStream {
    let p = quote! { ::yororen_ui::headless::layout::Spacing };
    match s {
        Spacing::Xs => quote! { #p::Xs },
        Spacing::Sm => quote! { #p::Sm },
        Spacing::Md => quote! { #p::Md },
        Spacing::Lg => quote! { #p::Lg },
        Spacing::Xl => quote! { #p::Xl },
        Spacing::Xxl => quote! { #p::Xxl },
        Spacing::Px(v) => {
            let lit = proc_macro2::Literal::f32_suffixed(*v);
            quote! { #p::Px(#lit) }
        }
        Spacing::Rem(v) => {
            let lit = proc_macro2::Literal::f32_suffixed(*v);
            quote! { #p::Rem(#lit) }
        }
    }
}

fn inset_to_tokens(i: &Inset) -> TokenStream {
    let p = quote! { ::yororen_ui::headless::layout::Inset };
    match i {
        Inset::Xs => quote! { #p::Xs },
        Inset::Sm => quote! { #p::Sm },
        Inset::Md => quote! { #p::Md },
        Inset::Lg => quote! { #p::Lg },
        Inset::Xl => quote! { #p::Xl },
        Inset::Px(v) => {
            let lit = proc_macro2::Literal::f32_suffixed(*v);
            quote! { #p::Px(#lit) }
        }
    }
}
