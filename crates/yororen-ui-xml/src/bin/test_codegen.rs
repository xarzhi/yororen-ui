fn main() {
    let contents = std::fs::read_to_string(
        "crates/yororen-ui-demos/gallery_xml/src/ui/gallery.xml",
    )
    .unwrap();
    let ts = yororen_ui_xml::codegen::codegen(
        &contents,
        proc_macro2::Span::call_site(),
        Some(quote::quote! { &mut **cx }),
        Some("crates/yororen-ui-demos/gallery_xml/src/view.rs"),
    )
    .unwrap();
    println!("{}", ts);
}
