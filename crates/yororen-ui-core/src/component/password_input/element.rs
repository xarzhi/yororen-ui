//! Password input element module.

use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, Entity, GlobalElementId, PaintQuad,
    Pixels, ShapedLine, Style, TextRun, UnderlineStyle, fill, point, relative, size,
};

use super::state::PasswordInputState;
use crate::theme::ActiveTheme;

pub struct PasswordLineElement {
    pub input: Entity<PasswordInputState>,
    pub disabled: bool,
}

pub struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_x: Pixels,
}

impl gpui::IntoElement for PasswordLineElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for PasswordLineElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let theme = cx.theme();
        let cursor_thickness: gpui::Pixels = theme.tokens.control.input.cursor_thickness;
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let marked_range = input.marked_range.clone();
        let style = window.text_style();
        let direction = cx.theme().text_direction;
        let is_rtl = direction.is_rtl();

        let display_text = input.display_text();
        let text_color = if content.is_empty() {
            cx.theme().content.tertiary
        } else {
            style.color
        };

        let cursor_display_index = input.display_index_for_content_offset(cursor);
        let selection_display_range = input.display_index_for_content_offset(selected_range.start)
            ..input.display_index_for_content_offset(selected_range.end);
        let marked_display_range = marked_range.as_ref().map(|range| {
            input.display_index_for_content_offset(range.start)
                ..input.display_index_for_content_offset(range.end)
        });

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = marked_display_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: cursor_thickness,
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let raw_cursor_pos = line.x_for_index(cursor_display_index);
        let cursor_pos = if is_rtl {
            line.width - raw_cursor_pos
        } else {
            raw_cursor_pos
        };

        let cursor_width: gpui::Pixels = theme.tokens.control.input.focus_ring_thickness;
        let _ = cursor_width;
        let max_cursor_x = (bounds.size.width - cursor_width).max(Pixels::ZERO);
        let max_scroll_x = (line.width - max_cursor_x).max(Pixels::ZERO);
        let mut scroll_x = input.scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        if cursor_pos < scroll_x {
            scroll_x = cursor_pos;
        } else if cursor_pos > scroll_x + max_cursor_x {
            scroll_x = cursor_pos - max_cursor_x;
        }
        scroll_x = scroll_x.clamp(Pixels::ZERO, max_scroll_x);

        let (selection, cursor) = if selected_range.is_empty() {
            let cursor_paint_x = if is_rtl {
                bounds.right() - cursor_pos + scroll_x
            } else {
                bounds.left() + cursor_pos - scroll_x
            };
            (
                None,
                input.cursor_visible.then(|| {
                    fill(
                        Bounds::new(
                            point(cursor_paint_x, bounds.top()),
                            size(cursor_width, bounds.bottom() - bounds.top()),
                        ),
                        cx.theme().border.focus,
                    )
                }),
            )
        } else {
            let raw_start_x = line.x_for_index(selection_display_range.start);
            let raw_end_x = line.x_for_index(selection_display_range.end);
            let (start_x, end_x) = if is_rtl {
                (line.width - raw_start_x, line.width - raw_end_x)
            } else {
                (raw_start_x, raw_end_x)
            };
            let selection_start_x = if is_rtl {
                bounds.right() - start_x + scroll_x
            } else {
                bounds.left() + start_x - scroll_x
            };
            let selection_end_x = if is_rtl {
                bounds.right() - end_x + scroll_x
            } else {
                bounds.left() + end_x - scroll_x
            };
            (
                Some(fill(
                    Bounds::from_corners(
                        point(selection_start_x.min(selection_end_x), bounds.top()),
                        point(selection_start_x.max(selection_end_x), bounds.bottom()),
                    ),
                    cx.theme().border.focus.alpha(0.25),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
            scroll_x,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        let direction = cx.theme().text_direction;
        let is_rtl = direction.is_rtl();
        if !self.disabled {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }
        let line = prepaint.line.take().expect("line should exist");

        let origin_x = if is_rtl {
            bounds.right() - line.width + prepaint.scroll_x
        } else {
            bounds.left() - prepaint.scroll_x
        };

        line.paint(
            point(origin_x, bounds.top()),
            window.line_height(),
            window,
            cx,
        )
        .expect("paint should succeed");

        if !self.disabled
            && focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.scroll_x = prepaint.scroll_x;
        });
    }
}
