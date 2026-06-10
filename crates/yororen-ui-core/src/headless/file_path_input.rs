//! Headless `file_path_input` — text input specialised for
//! filesystem paths, with a folder-icon and a "browse" button.

use std::sync::Arc;

use gpui::{App, Hsla};

pub type TextChangeCallback = Arc<dyn Fn(&str, &mut gpui::Window, &mut App) + Send + Sync>;
pub type TextBrowseCallback = TextChangeCallback;

#[derive(Clone)]
pub struct FilePathInputProps {
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub value: String,
    pub on_change: Option<TextChangeCallback>,
    pub on_browse: Option<TextBrowseCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

pub fn file_path_input(id: impl Into<gpui::ElementId>) -> FilePathInputProps {
    FilePathInputProps {
        id: id.into(),
        placeholder: "/path/to/file".to_string(),
        disabled: false,
        value: String::new(),
        on_change: None,
        on_browse: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl FilePathInputProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_browse<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut gpui::Window, &mut App),
    {
        self.on_browse = Some(Arc::new(f));
        self
    }
    pub fn has_custom_bg(mut self, v: bool) -> Self {
        self.has_custom_bg = v;
        self
    }
    pub fn has_custom_border(mut self, v: bool) -> Self {
        self.has_custom_border = v;
        self
    }
    pub fn has_custom_focus_border(mut self, v: bool) -> Self {
        self.has_custom_focus_border = v;
        self
    }
    pub fn custom_bg(mut self, c: Hsla) -> Self {
        self.custom_bg = Some(c);
        self.has_custom_bg = true;
        self
    }
    pub fn custom_border(mut self, c: Hsla) -> Self {
        self.custom_border = Some(c);
        self.has_custom_border = true;
        self
    }
    pub fn custom_focus_border(mut self, c: Hsla) -> Self {
        self.custom_focus_border = Some(c);
        self.has_custom_focus_border = true;
        self
    }
    pub fn custom_text_color(mut self, c: Hsla) -> Self {
        self.custom_text_color = Some(c);
        self
    }

    /// Render the file path input using the registered `FilePathInputRenderer`.
    pub fn render(self, cx: &mut gpui::App, window: &mut gpui::Window) -> gpui::AnyElement {
        use crate::headless::icon::{IconSource, icon};
        use crate::headless::text_input::TextInputState;
        use crate::headless::text_input_element::{
            TextInputElement, start_cursor_blink, wire_input_keyboard,
        };
        use crate::renderer::RendererContext;
        use crate::renderer::file_path_input::{FilePathInputRenderState, FilePathInputRenderer};
        use crate::renderer::markers::FilePathInput as FilePathInputMarker;
        use crate::renderer::spec::Edges;
        use crate::theme::ActiveTheme;
        use gpui::{
            AppContext, CursorStyle, InteractiveElement, IntoElement, MouseButton, ParentElement,
            Stateful, StatefulInteractiveElement, Styled, div, px,
        };
        use std::sync::Arc;

        let theme_arc = cx.theme().clone();
        let r: Arc<dyn FilePathInputRenderer> = cx
            .renderer_arc::<FilePathInputMarker, dyn FilePathInputRenderer>()
            .expect("FilePathInputRenderer registered")
            .clone();
        let theme = &*theme_arc;

        let id = self.id.clone();
        let placeholder_str = self.placeholder.clone();
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let on_browse = self.on_browse.clone();

        let state = window.use_keyed_state(self.id.clone(), cx, |_window, cx| {
            TextInputState::new(&mut *cx)
        });
        state.update(cx, |s, _cx| {
            s.placeholder = gpui::SharedString::from(placeholder_str);
            s.on_change = on_change.clone();
        });

        let focus_handle = state.read(cx).focus_handle();
        let focused = focus_handle.is_focused(window);

        let render_state = FilePathInputRenderState {
            disabled,
            focused,
            custom_bg: self.custom_bg,
            custom_border: self.custom_border,
            custom_focus_border: self.custom_focus_border,
            custom_fg: self.custom_text_color,
        };
        let bg = r.bg(&render_state, theme);
        let border_color = if focused {
            r.focus_border(&render_state, theme)
        } else {
            r.border(&render_state, theme)
        };
        let text_color = theme.get_color("content.primary").unwrap_or_default();
        let button_fg = r.button_fg(&render_state, theme);
        let button_bg = r.button_bg(&render_state, theme);
        let button_hover_bg = r.button_hover_bg(&render_state, theme);
        let min_h = r.min_height(&render_state, theme);
        let padding: Edges<gpui::Pixels> = r.padding(&render_state, theme);
        let radius = r.border_radius(&render_state, theme);
        let action_gap = r.action_gap(&render_state, theme);
        let icon_size = r.icon_size(&render_state, theme);

        if focused {
            start_cursor_blink(state.clone(), window, cx);
        } else {
            state.update(cx, |s, _cx| s.cursor_visible = true);
        }

        let inner = TextInputElement {
            state: state.clone(),
            focus_handle: focus_handle.clone(),
            disabled,
            text_color,
            hint_color: theme.get_color("content.tertiary").unwrap_or_default(),
            cursor_color: text_color,
            selection_color: text_color,
            placeholder: state.read(cx).placeholder.clone(),
            value_override: None,
        };

        let base: Stateful<gpui::Div> = div()
            .id(id.clone())
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .items_center()
            .gap(action_gap)
            .text_color(text_color)
            .overflow_hidden()
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            });

        let focused_div: Stateful<gpui::Div> = base.track_focus(&focus_handle);
        let keyed = wire_input_keyboard(
            focused_div,
            state.clone(),
            focus_handle.clone(),
            disabled,
            None,
        );

        let hover_border = r.hover_border(&render_state, theme);
        let active_border = r.active_border(&render_state, theme);

        let on_browse_clone = on_browse.clone();
        let window_handle = window.window_handle();
        let on_change_for_async = state.read(cx).on_change.clone();
        let state_for_browse = state.clone();

        keyed
            .hover(|s| s.border_color(hover_border))
            .active(|s| s.border_color(active_border))
            .child(
                icon(
                    "file-path-input-leading-icon",
                    IconSource::Builtin("folder".into()),
                    cx,
                )
                .size(icon_size)
                .color(text_color)
                .render(cx),
            )
            .child(div().flex_1().min_w(px(0.)).child(inner))
            .child(
                div()
                    .id("file-path-input-browse")
                    .size(icon_size)
                    .bg(button_bg)
                    .rounded(px(4.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(button_fg)
                    .hover(|s| s.bg(button_hover_bg))
                    .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                        if disabled {
                            return;
                        }
                        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: false,
                            prompt: Some("Select a file".into()),
                        });
                        let on_change_for_async = on_change_for_async.clone();
                        let state_for_change = state_for_browse.clone();
                        let on_browse_cb = on_browse_clone.clone();
                        cx.spawn(async move |async_cx| {
                            let picked = match receiver.await {
                                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                                _ => None,
                            };
                            if let Some(path) = picked {
                                let path_str = path.to_string_lossy().to_string();
                                let state_for_change = state_for_change.clone();
                                let on_browse_for_async = on_browse_cb.clone();
                                let _ =
                                    async_cx.update_window(window_handle, move |_, window, cx| {
                                        state_for_change.update(cx, |s, cx| {
                                            s.value = path_str.clone();
                                            let end = s.value.len();
                                            s.caret = end;
                                            s.selection_start = end;
                                            s.selection_end = end;
                                            cx.notify();
                                        });
                                        if let Some(cb) = on_change_for_async.as_ref() {
                                            cb(&path_str, window, cx);
                                        }
                                        if let Some(cb) = on_browse_for_async.as_ref() {
                                            cb(&path_str, window, cx);
                                        }
                                    });
                            }
                        })
                        .detach();
                    })
                    .child(
                        icon(
                            "file-path-input-browse-icon",
                            IconSource::Builtin("file".into()),
                            cx,
                        )
                        .size(icon_size)
                        .color(button_fg)
                        .render(cx),
                    ),
            )
            .into_any_element()
    }
}
