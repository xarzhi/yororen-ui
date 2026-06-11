//! `TableRenderer` — visual contract for `Table`.
//!
//! Trait surface is just `compose`. The renderer paints the table
//! chrome: header row, body rows, selected row highlight, borders,
//! and cell sizing. The headless layer owns columns, rows, and
//! selection callbacks.

use std::any::Any;

use gpui::{App, Div, Stateful};

use crate::headless::table::TableProps;

#[derive(Clone, Copy, Debug, Default)]
pub struct TableRenderState {
    pub selected_row: Option<usize>,
}

pub trait TableRenderer: Any + Send + Sync {
    /// Build the full `Stateful<Div>` for the table, including the
    /// header and all data rows.
    fn compose(&self, props: &TableProps, cx: &App) -> Stateful<Div>;
}
