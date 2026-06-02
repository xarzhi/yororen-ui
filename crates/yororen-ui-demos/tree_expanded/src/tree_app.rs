//! Tree Expanded Demo App
//!
//! Displays a tree with persistent expansion state.
//!
//! ## How the bug manifests
//! If `collect_expanded` overwrites user state, collapsing a node and then
//! triggering a re-render will cause the node to re-expand.
//!
//! ## How to verify the fix
//! 1. Click the disclosure arrow next to "Child A" to collapse it.
//! 2. Click "Force re-render" — Child A should stay collapsed.
//! 3. Click "Reset with new nodes" — labels update (new generation) but
//!    your expansion choices should be preserved.

use gpui::{Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px};

use yororen_ui::component::{TreeState, button, label, tree};
use yororen_ui::theme::ActiveTheme;

use crate::state::build_nodes;

pub struct TreeExpandedApp {
    nodes: Vec<yororen_ui::component::TreeNode>,
    last_generation: u32,
    render_count: u32,
}

impl TreeExpandedApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            nodes: build_nodes(0),
            last_generation: 0,
            render_count: 0,
        }
    }

    fn refresh_nodes(&mut self, generation: u32) {
        self.nodes = build_nodes(generation);
        self.last_generation = generation;
    }
}

impl Render for TreeExpandedApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let title = div()
            .text_xl()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.content.primary)
            .child("Tree Expanded State Demo");

        let description = div()
            .text_sm()
            .text_color(theme.content.secondary)
            .child("Collapse a node, then trigger re-render. Your expansion choices should be preserved.");

        // Action buttons
        let rerender_btn = button("force-rerender")
            .child("Force re-render")
            .on_click(|_ev, window, _cx| {
                window.refresh();
            });

        let reset_btn = button("reset-nodes")
            .child("Reset with new nodes")
            .on_click({
                let generation = cx
                    .global::<crate::state::TreeDemoState>()
                    .generation
                    .clone();
                move |_ev, window, cx| {
                    generation.update(cx, |g, _| *g += 1);
                    window.refresh();
                }
            });

        let btn_row = div()
            .flex()
            .gap(px(12.))
            .child(rerender_btn)
            .child(reset_btn);

        // Read current generation from global state
        let current_gen = *cx
            .global::<crate::state::TreeDemoState>()
            .generation
            .read(cx);

        // Refresh nodes if generation changed
        if self.last_generation != current_gen {
            self.refresh_nodes(current_gen);
        }

        self.render_count += 1;

        let status = div()
            .text_sm()
            .text_color(theme.content.secondary)
            .child(format!(
                "Generation: {} | Render count: {}",
                current_gen, self.render_count
            ));

        // Build tree with internal keyed state so expansion persists
        let tree_view = tree(TreeState::new(), &self.nodes).id("demo:tree");

        // Tips panel
        let tips = div()
            .mt(px(16.))
            .p(px(16.))
            .rounded_md()
            .bg(theme.surface.raised)
            .border_1()
            .border_color(theme.border.default)
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(label("How to verify the fix").strong(true))
            .child(
                label("1. Click the arrow next to 'Child A' to collapse it.")
                    .muted(true)
                    .wrap(),
            )
            .child(
                label("2. Click 'Force re-render'. Child A should remain collapsed.")
                    .muted(true)
                    .wrap(),
            )
            .child(
                label(
                    "3. Click 'Reset with new nodes'. Labels change (new generation) but expansion stays.",
                )
                .muted(true)
                .wrap(),
            )
            .child(
                label(
                    "4. If a node unexpectedly re-expands, collect_expanded is overwriting user state.",
                )
                .muted(true)
                .wrap(),
            );

        div()
            .size_full()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .p(px(24.))
            .gap(px(12.))
            .child(title)
            .child(description)
            .child(status)
            .child(btn_row)
            .child(tree_view)
            .child(tips)
    }
}
