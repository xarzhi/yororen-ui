//! Section 8 — Lists, Tables, Trees, Forms, Misc. Each
//! component is wrapped in a labelled `cell` so the user can
//! identify every instance.

use std::collections::BTreeSet;

use gpui::{Context, Div, ParentElement, Styled, div, px};

use yororen_ui::headless::form::form;
use yororen_ui::headless::form_field::form_field;
use yororen_ui::headless::label::label;
use yororen_ui::headless::list_item::list_item;
use yororen_ui::headless::radio_group::radio_group;
use yororen_ui::headless::spacer::spacer;
use yororen_ui::headless::table::TableColumn;
use yororen_ui::headless::table::table;
use yororen_ui::headless::tree::TreeData;
use yororen_ui::headless::tree::node_id;
use yororen_ui::headless::tree::tree;
use yororen_ui::headless::tree_item::tree_item;
use yororen_ui::headless::virtual_list::virtual_list;
use yororen_ui::i18n::Translate;

use crate::sections::cell;
use crate::state::GalleryApp;

const LIST_ITEMS: &[(&str, &str)] = &[
    ("li-1", "First item"),
    ("li-2", "Second item"),
    ("li-3", "Third item"),
];

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    // --- list_item: 3 selectable rows ---
    let mut list_col = div().flex().flex_col().gap(px(4.)).w(px(220.));
    for (i, (id, title)) in LIST_ITEMS.iter().enumerate() {
        list_col = list_col.child(
            list_item(*id, *title, cx)
                .selected(app.selected_list_item == Some(i))
                .render(cx),
        );
    }
    let list_wrapped = cell("list_item (3 rows; selected via props)", list_col, cx);

    // --- form + form_field ---
    let entity_form = cx.entity().clone();
    let form_el = form("lists-form", cx)
        .value("email", app.form_email_value.clone())
        .on_submit(move |vals, _w, cx| {
            entity_form.update(cx, |s, _cx| {
                s.form_submit_count += 1;
                if let Some(e) = vals.get("email") {
                    s.form_email_value = e.to_string();
                    s.form_email_error = if e.contains('@') {
                        None
                    } else {
                        Some("must contain @".to_string())
                    };
                }
            });
        })
        .render(cx)
        .w(px(300.))
        .child(
            form_field("lists-ff-email", "email", cx)
                .label(cx.t("form.email"))
                .required(true)
                .render(cx)
                .child(
                    div()
                        .p(px(6.))
                        .border_1()
                        .rounded(px(4.))
                        .child(
                            label(
                                "form-email-display",
                                if app.form_email_value.is_empty() {
                                    cx.t("form.email_placeholder").to_string()
                                } else {
                                    app.form_email_value.clone()
                                },
                                cx,
                            )
                            .muted(app.form_email_value.is_empty())
                            .render(cx),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .items_center()
                .child(label("form-submit-count", format!("submitted: {}", app.form_submit_count), cx).muted(true).render(cx))
                .child(label("form-submit-hint", cx.t("form.click_submit"), cx).muted(true).render(cx)),
        );
    let form_wrapped = cell("form + form_field (email validation)", form_el, cx);

    // --- table ---
    let entity_table = cx.entity().clone();
    let table_el = table("lists-table", cx)
        .columns(vec![
            TableColumn::new("name", "Name").width(120.),
            TableColumn::new("age", "Age").width(60.),
            TableColumn::new("city", "City").width(120.),
        ])
        .rows(vec![
            vec!["Alice".into(), "30".into(), "Beijing".into()],
            vec!["Bob".into(), "25".into(), "Shanghai".into()],
            vec!["Carol".into(), "28".into(), "Shenzhen".into()],
        ])
        .selected(app.selected_table_row.unwrap_or(0))
        .on_select(move |i, _w, cx| {
            entity_table.update(cx, |s, _cx| s.selected_table_row = Some(i));
        })
        .render(cx)
        .w(px(320.));
    let table_wrapped = cell("table (3 rows × 3 cols)", table_el, cx);

    // --- tree (with tree_item rows) ---
    let mut tree_data = TreeData::new();
    tree_data.add(None, node_id("root"), "Root");
    tree_data.add(Some(node_id("root")), node_id("child1"), "Child 1");
    tree_data.add(Some(node_id("root")), node_id("child2"), "Child 2");
    tree_data.add(Some(node_id("child1")), node_id("leaf1"), "Leaf 1");
    tree_data.add(Some(node_id("child1")), node_id("leaf2"), "Leaf 2");
    let entity_tree = cx.entity().clone();
    let tree_expanded: BTreeSet<_> = app.tree_expanded.clone();
    let mut tree_el = tree("lists-tree", cx)
        .data(tree_data)
        .render(cx)
        .w(px(220.));
    for (id, label_text, depth) in [
        (node_id("root"), "Root", 0usize),
        (node_id("child1"), "Child 1", 1),
        (node_id("child2"), "Child 2", 1),
    ] {
        let entity_for_row = entity_tree.clone();
        let id_for_row = id.clone();
        let is_expanded = tree_expanded.contains(&id);
        tree_el = tree_el.child(
            tree_item("ti", id.clone(), label_text, cx)
                .depth(depth)
                .has_children(true)
                .expanded(is_expanded)
                .on_toggle(move |_, _, cx| {
                    entity_for_row.update(cx, |s, _cx| {
                        if !s.tree_expanded.remove(&id_for_row) {
                            s.tree_expanded.insert(id_for_row.clone());
                        }
                    });
                })
                .render(cx),
        );
    }
    let tree_wrapped = cell("tree + tree_item (3 rows; click chevron to expand)", tree_el, cx);

    // --- virtual_list ---
    let vl = virtual_list("lists-vl", 1_000, cx)
        .overdraw(20.0)
        .render(cx)
        .w(px(220.))
        .h(px(120.))
        .child(label("vl-blank", "(1000 items; scroll to test)", cx).muted(true).render(cx));
    let vl_wrapped = cell("virtual_list (1000 items)", vl, cx);

    // --- spacer ---
    let sp = spacer("lists-spacer", cx)
        .render(cx)
        .h(px(16.))
        .w_full();
    let sp_wrapped = cell("spacer (16px tall)", sp, cx);

    // --- radio_group empty (also used as a layout shell) ---
    let rg_demo = radio_group("lists-rg", cx)
        .name("rg-2")
        .render(cx)
        .child(label("rg-2-info", "Standalone radio_group (no children)", cx).muted(true).render(cx));
    let rg_wrapped = cell("radio_group (empty shell)", rg_demo, cx);

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(list_wrapped)
        .child(form_wrapped)
        .child(table_wrapped)
        .child(tree_wrapped)
        .child(vl_wrapped)
        .child(sp_wrapped)
        .child(rg_wrapped)
}
