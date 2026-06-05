//! Headless `table` — columns + rows + optional selection. The
//! visual lives in the renderer.

use std::sync::Arc;

use gpui::{App, Div, ElementId, SharedString, Stateful};

pub type TableCellValue = SharedString;

#[derive(Clone, Debug)]
pub struct TableColumn {
    pub id: SharedString,
    pub header: SharedString,
    pub width_px: Option<f32>,
}

impl TableColumn {
    pub fn new(id: impl Into<SharedString>, header: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width_px: None,
        }
    }
    pub fn width(mut self, px: f32) -> Self {
        self.width_px = Some(px);
        self
    }
}

pub type TableRow = Vec<TableCellValue>;

pub type TableSelectCallback =
    Arc<dyn Fn(usize, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone)]
pub struct TableProps {
    pub id: ElementId,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub selected_row: Option<usize>,
    pub on_select_row: Option<TableSelectCallback>,
}

pub fn table(id: impl Into<ElementId>, _cx: &mut App) -> TableProps {
    TableProps {
        id: id.into(),
        columns: Vec::new(),
        rows: Vec::new(),
        selected_row: None,
        on_select_row: None,
    }
}

impl TableProps {
    pub fn column(mut self, c: TableColumn) -> Self {
        self.columns.push(c);
        self
    }
    pub fn columns(mut self, cs: impl IntoIterator<Item = TableColumn>) -> Self {
        self.columns.extend(cs);
        self
    }
    pub fn row(mut self, r: TableRow) -> Self {
        self.rows.push(r);
        self
    }
    pub fn rows(mut self, rs: impl IntoIterator<Item = TableRow>) -> Self {
        self.rows.extend(rs);
        self
    }
    pub fn selected(mut self, i: usize) -> Self {
        self.selected_row = Some(i);
        self
    }
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(usize, &mut gpui::Window, &mut gpui::App),
    {
        self.on_select_row = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
