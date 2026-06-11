//! Section 8 — Lists, Tables, Trees, Forms, Misc. Each
//! component is wrapped in a labelled `cell` so the user can
//! identify every instance.

use std::collections::BTreeSet;

use gpui::{Context, Div, ElementId, ParentElement, Styled, Window, div, px};

use yororen_ui::headless::form::form;
use yororen_ui::headless::form_field::form_field;
use yororen_ui::headless::label::label;
use yororen_ui::headless::list_item::list_item;
use yororen_ui::headless::radio_group::radio_group;
use yororen_ui::headless::spacer::spacer;
use yororen_ui::headless::table::TableColumn;
use yororen_ui::headless::table::table;
use yororen_ui::headless::text_input::text_input;
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

pub fn render(app: &mut GalleryApp, window: &mut Window, cx: &mut Context<GalleryApp>) -> Div {
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

    // --- form + form_field (with a real text_input + submit button) ---
    let entity_form = cx.entity().clone();
    let entity_text = cx.entity().clone();
    let email_input_el = text_input("lists-ff-email-input")
        .placeholder(cx.t("form.email_placeholder"))
        .on_change(move |new: &str, _w, cx| {
            entity_text.update(cx, |s, _cx| s.form_email_value = new.to_string());
        })
        .render(&mut **cx, window);

    let form_field_el = form_field("lists-ff-email", "email", cx)
        .label(cx.t("form.email"))
        .required(true)
        .input(email_input_el)
        .render(cx);

    let form_props = form("lists-form", cx)
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
        .submit("Submit");

    let submit_btn_el = form_props
        .submit_button(&mut **cx)
        .expect("submit_label was set");

    let form_el = form_props
        .render(cx)
        .w(px(300.))
        .child(form_field_el)
        .child(submit_btn_el)
        .child(
            label(
                "form-submit-count",
                format!("submitted: {} | last error: {:?}", app.form_submit_count, app.form_email_error),
                cx,
            )
            .muted(true)
            .render(cx),
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
    let tree_data_for_iter = tree_data.clone();
    let entity_tree = cx.entity().clone();
    let tree_expanded: BTreeSet<_> = app.tree_expanded.clone();
    let tree_selected = app.tree_selected.clone();
    let mut tree_el = tree("lists-tree", cx)
        .data(tree_data)
        .render(cx)
        .w(px(240.));
    // Use the `TreeData::flatten` helper to walk only the
    // currently-visible (expanded) nodes, in depth-first
    // pre-order — mirrors v0.2's `flatten_tree` output.
    let visible = tree_data_for_iter.flatten(&tree_expanded);
    for (id, depth) in visible {
        let has_children = !tree_data_for_iter.children_of(&id).is_empty();
        let label_text = tree_data_for_iter
            .label_of(&id)
            .unwrap_or("")
            .to_string();
        let is_expanded = tree_expanded.contains(&id);
        let is_selected = tree_selected.as_ref() == Some(&id);

        let entity_for_toggle = entity_tree.clone();
        let entity_for_select = entity_tree.clone();
        let toggle_id = id.clone();
        let select_id = id.clone();
        // Unique ElementId per row — gpui de-duplicates by id,
        // so identical ids would collapse all rows into one.
        let row_id: ElementId = format!("lists-tree-row-{}", id.0).into();
        // Double-click toggles: the renderer falls back to
        // `on_toggle` when `on_double_click` is not set, so we
        // don't need to wire it explicitly — but we wire it
        // here to make the behaviour explicit at the call site.
        let entity_for_double = entity_tree.clone();
        let double_id = id.clone();
        tree_el = tree_el.child(
            tree_item(row_id, id.clone(), label_text, cx)
                .depth(depth)
                .has_children(has_children)
                .expanded(is_expanded)
                .selected(is_selected)
                .on_toggle(move |_, _, cx| {
                    let toggle_id = toggle_id.clone();
                    entity_for_toggle.update(cx, |s, _cx| {
                        if !s.tree_expanded.remove(&toggle_id) {
                            s.tree_expanded.insert(toggle_id);
                        }
                    });
                })
                .on_click(move |_, _, cx| {
                    entity_for_select.update(cx, |s, _cx| {
                        s.tree_selected = Some(select_id.clone());
                    });
                })
                .on_double_click(move |_, _, cx| {
                    let double_id = double_id.clone();
                    entity_for_double.update(cx, |s, _cx| {
                        if !s.tree_expanded.remove(&double_id) {
                            s.tree_expanded.insert(double_id);
                        }
                    });
                })
                .render(&mut **cx, window),
        );
    }
    let tree_wrapped = cell("tree + tree_item (3-5 rows; click chevron or double-click row to expand, click row to select)", tree_el, cx);

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
