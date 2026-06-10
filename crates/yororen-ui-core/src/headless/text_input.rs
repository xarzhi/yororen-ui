//! Headless `text_input` — text state, IME, selection, on_change,
//! on_submit, key map. No visual.
//!
//! The headless layer owns the *textual* state machine: value,
//! caret/selection (in UTF-8 bytes), UTF-16 mirror for the IME
//! pipeline, scroll position, last shaped line, cursor blink.
//!
//! ## Why this is a `gpui::Entity<TextInputState>`
//!
//! `gpui-ce 0.3` requires the `EntityInputHandler` trait to be
//! implemented on a state held in an `Entity<T>`, so the platform
//! IME / clipboard layer can `update` it via `Window::handle_input`
//! without going through render closures. The renderer's
//! `default_render` mints the entity via `window.use_keyed_state`
//! and `paint` calls `window.handle_input(&focus_handle,
//! ElementInputHandler::new(bounds, entity.clone()), cx)` to
//! register the IME.
//!
//! ## Key dispatch
//!
//! Key dispatch lives in the renderer (the wrapper `div` has
//! `.key_context("UITextInput")` and a stack of `.on_action(...)`
//! calls). `init()` registers the keymap against the
//! `"UITextInput"` context (idempotent via `OnceLock`); apps that
//! want a different keymap simply don't call it.

use std::ops::Range;
use std::sync::{Arc, OnceLock};

use gpui::{
    App, Bounds, Context, EntityInputHandler, FocusHandle, Focusable, Hsla, IntoElement,
    KeyBinding, ParentElement, Pixels, ShapedLine, SharedString, StatefulInteractiveElement,
    Styled, UTF16Selection, Window, actions, point,
};

use crate::renderer::text_input::{TextInputRenderState, TextInputRenderer};

pub type TextChangeCallback = Arc<dyn Fn(&str, &mut Window, &mut App) + Send + Sync>;

// =====================================================================
// `actions!` — the 14 keyboard actions scoped to the `UITextInput`
// key context. The renderer wires `.on_action` to each; `init()`
// binds keys to them.
// =====================================================================

actions!(
    ui_text_input,
    [
        Backspace,
        Delete,
        Enter,
        Escape,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

/// Wire the text-input keymap into the running `App`. Idempotent —
/// subsequent calls are no-ops. Apps that want their own keymap
/// (e.g. an editor that uses `secondary-v` for paste globally)
/// simply don't call this.
pub fn init(cx: &mut App) {
    static DONE: OnceLock<()> = OnceLock::new();
    if DONE.set(()).is_err() {
        return;
    }
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("UITextInput")),
        KeyBinding::new("delete", Delete, Some("UITextInput")),
        KeyBinding::new("enter", Enter, Some("UITextInput")),
        KeyBinding::new("escape", Escape, Some("UITextInput")),
        KeyBinding::new("left", Left, Some("UITextInput")),
        KeyBinding::new("right", Right, Some("UITextInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("UITextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("UITextInput")),
        KeyBinding::new("secondary-a", SelectAll, Some("UITextInput")),
        KeyBinding::new("secondary-v", Paste, Some("UITextInput")),
        KeyBinding::new("secondary-c", Copy, Some("UITextInput")),
        KeyBinding::new("secondary-x", Cut, Some("UITextInput")),
        KeyBinding::new("home", Home, Some("UITextInput")),
        KeyBinding::new("end", End, Some("UITextInput")),
        KeyBinding::new(
            "ctrl-secondary-space",
            ShowCharacterPalette,
            Some("UITextInput"),
        ),
    ]);
}

// =====================================================================
// `action_handler!` — for each action the renderer's wrapper div
// chains `.on_action(action_handler!(state, disabled, Backspace,
// backspace))`. The macro expands to a closure of type `Fn(&Action,
// &mut Window, &mut App)` that early-returns when disabled, then
// `Entity::update`s the state.
// =====================================================================

/// Build a `.on_action(...)` closure for one of the 14 actions.
///
/// Usage in the renderer:
/// ```ignore
/// .on_action(action_handler!(state, disabled, Backspace, backspace))
/// ```
#[macro_export]
macro_rules! action_handler {
    ($state:expr, $disabled:expr, $action:ty, $method:ident) => {{
        let state = $state.clone();
        let disabled = $disabled;
        move |action: &$action, window: &mut gpui::Window, cx: &mut gpui::App| {
            if disabled {
                return;
            }
            let _ = state.update(cx, |s, ctx| s.$method(action, window, ctx));
        }
    }};
}

// =====================================================================
// `TextInputState` — the live state. Held in `Entity<TextInputState>`
// so the IME / action pipeline can update it.
// =====================================================================

/// The text state, caret, selection, scroll, and IME bookkeeping.
pub struct TextInputState {
    /// Text content (UTF-8 byte string). `String` not `SharedString`
    /// because every keystroke rewrites it; interning wouldn't help.
    pub value: String,
    /// Caret position, in UTF-8 bytes. `0..=value.len()`.
    pub caret: usize,
    /// Selection start, in UTF-8 bytes. `selection_start <=
    /// selection_end`. When equal to `selection_end`, the selection
    /// is empty (just a caret at `selection_end`).
    pub selection_start: usize,
    pub selection_end: usize,
    /// Placeholder text shown when `value` is empty. The renderer
    /// paints it in the hint color.
    pub placeholder: SharedString,
    /// Horizontal scroll offset (pixels) when the text is wider
    /// than the visible width. Updated by the renderer's
    /// `prepaint`; read by `bounds_for_range` and
    /// `character_index_for_point`.
    pub scroll_x: Pixels,
    /// The `ShapedLine` produced by the renderer's `prepaint`.
    /// Cached here so the IME `bounds_for_range` /
    /// `character_index_for_point` can answer without re-shaping.
    pub last_layout: Option<ShapedLine>,
    /// The bounds of the text area, in window coordinates. Cached
    /// by the renderer's `paint` for the same reason.
    pub last_bounds: Option<Bounds<Pixels>>,
    /// Per-line shaped layouts from the renderer's last paint.
    /// **Empty** for single-line inputs (text_input, search_input,
    /// password_input, etc.). Populated by `TextAreaElement::paint`
    /// for multi-line rendering. When non-empty, the IME's
    /// `bounds_for_range` / `character_index_for_point` use this
    /// for multi-row lookups (find the row from Y, then the
    /// column from X).
    pub last_line_layouts: Vec<ShapedLine>,
    /// Byte range in `value` for each row in `last_line_layouts`.
    /// Length matches `last_line_layouts`. The i-th range is
    /// `line_i_start..line_i_end`; for non-final rows the end
    /// includes the trailing `'\n'` so the next row's start is
    /// `end`. The last row's end is `value.len()` (or the
    /// position right after a trailing `'\n'`).
    pub last_line_byte_ranges: Vec<Range<usize>>,
    /// Line height in pixels, captured by the renderer at paint
    /// time. Used by `character_index_for_point_inner` to find
    /// the row from the click's Y coordinate.
    pub last_line_height: Option<Pixels>,
    /// `true` while the user is drag-selecting with the mouse.
    /// Toggled by `MouseDown` / `MouseUp` handlers.
    pub is_selecting: bool,
    /// Whether the caret quad is currently painted. Toggled by
    /// the cursor-blink timer in the renderer.
    pub cursor_visible: bool,
    /// Monotonically increasing epoch for the cursor-blink task.
    /// Each focus-in bumps this; the running task checks it on
    /// each tick and exits if the epoch has changed (so we never
    /// have two blink tasks racing).
    pub cursor_blink_epoch: usize,
    /// Active IME composition range, in **UTF-8 bytes**. `Some`
    /// when the platform IME is composing (e.g. typing Chinese
    /// pinyin) — the renderer uses this to highlight the marked
    /// span. The platform calls
    /// `replace_and_mark_text_in_range` to start / update a
    /// composition and `replace_text_in_range` (with this
    /// range) to commit it.
    pub marked_range: Option<Range<usize>>,
    /// Hard cap on `value.len()`. `None` = unlimited. The
    /// `replace_text_in_range_bytes` method (used by every
    /// action method and the platform's `EntityInputHandler`)
    /// truncates to this cap so the value can never exceed it.
    pub max_length: Option<usize>,
    /// Fired whenever the value changes. The renderer sets this
    /// once in `default_render`; the action methods invoke it
    /// after a successful mutation. Closure receives the new
    /// value (UTF-8) plus a `Window` + `App` (the renderer's
    /// `on_change` signature).
    pub on_change: Option<TextChangeCallback>,
    /// Fired on Enter. The renderer sets this once and the
    /// `Enter` action handler invokes it.
    pub on_submit: Option<TextChangeCallback>,
    /// When `true`, `paste` keeps `'\n'` in the pasted text.
    /// `false` (the default) replaces `'\n'` with `' '` — the
    /// single-line input convention. `text_area`'s renderer
    /// sets this to `true` so multi-line paste works.
    pub paste_newlines: bool,
    /// Focus handle minted in `new`. Private — external callers
    /// go through `Focusable::focus_handle`.
    focus_handle: FocusHandle,
}

impl TextInputState {
    /// Mint a new state. Called by the renderer via
    /// `window.use_keyed_state(id, cx, |_, cx| TextInputState::new(cx))`.
    pub fn new(cx: &mut App) -> Self {
        Self {
            value: String::new(),
            caret: 0,
            selection_start: 0,
            selection_end: 0,
            placeholder: SharedString::new_static(""),
            scroll_x: Pixels::ZERO,
            last_layout: None,
            last_bounds: None,
            last_line_layouts: Vec::new(),
            last_line_byte_ranges: Vec::new(),
            last_line_height: None,
            is_selecting: false,
            cursor_visible: true,
            cursor_blink_epoch: 0,
            marked_range: None,
            max_length: None,
            on_change: None,
            on_submit: None,
            paste_newlines: false,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Public focus handle accessor (mirrors the v0.2 pattern; the
    /// `focus_handle` field is private because we implement the
    /// `Focusable` trait which exposes the same name).
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    // -- Pure data accessors -----------------------------------------

    /// Currently selected byte range. Empty when there's no
    /// selection (just a caret).
    pub fn selected_range(&self) -> Range<usize> {
        self.selection_start.min(self.selection_end)..self.selection_start.max(self.selection_end)
    }

    /// `true` if a non-empty selection is active.
    pub fn has_selection(&self) -> bool {
        self.selection_start != self.selection_end
    }

    /// Set `value` programmatically (e.g. clear button, initial
    /// value, load from disk). Resets the caret and selection.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.caret = self.value.len();
        self.selection_start = 0;
        self.selection_end = 0;
        self.scroll_x = Pixels::ZERO;
    }

    /// Convenience for external callers (e.g. on_submit) — clones
    /// the current value.
    pub fn content(&self) -> String {
        self.value.clone()
    }

    // -- UTF-8 ↔ UTF-16 helpers (the IME pipeline talks UTF-16) -----

    /// `byte_offset` (UTF-8) → `usize` UTF-16 code units.
    pub fn offset_to_utf16(&self, byte_offset: usize) -> usize {
        let mut count = 0usize;
        for (i, c) in self.value.char_indices() {
            if i >= byte_offset {
                return count;
            }
            count += c.len_utf16();
        }
        count
    }

    /// `utf16_offset` → UTF-8 byte offset.
    pub fn utf16_to_offset(&self, utf16_offset: usize) -> usize {
        let mut count = 0usize;
        for (i, c) in self.value.char_indices() {
            if count >= utf16_offset {
                return i;
            }
            count += c.len_utf16();
        }
        self.value.len()
    }

    /// Convert a UTF-16 range to a UTF-8 byte range. `None` if
    /// either end fails to align to a char boundary.
    pub fn range_to_utf16(&self, byte_range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(byte_range.start)..self.offset_to_utf16(byte_range.end)
    }

    /// Inverse.
    pub fn range_from_utf16(&self, utf16_range: &Range<usize>) -> Range<usize> {
        self.utf16_to_offset(utf16_range.start)..self.utf16_to_offset(utf16_range.end)
    }

    /// `text_for_range` body — return the substring for a UTF-16
    /// range, plus the *adjusted* UTF-8 range the platform may
    /// have requested (the platform passes `range_utf16` and we
    /// answer with what we actually returned).
    pub fn text_for_range_utf16(&self, range_utf16: Range<usize>) -> (String, Range<usize>) {
        let start = self.utf16_to_offset(range_utf16.start);
        let end = self.utf16_to_offset(range_utf16.end);
        let text = self.value.get(start..end).unwrap_or("").to_string();
        (text, start..end)
    }

    // -- Char-boundary walking (UTF-8 safe) --------------------------

    /// `byte_offset` → previous char boundary (UTF-8 safe).
    pub fn prev_boundary(&self, byte_offset: usize) -> usize {
        if byte_offset == 0 {
            return 0;
        }
        let bytes = self.value.as_bytes();
        let mut i = byte_offset - 1;
        while i > 0 && (bytes[i] & 0b1100_0000) == 0b1000_0000 {
            i -= 1;
        }
        i
    }

    /// `byte_offset` → next char boundary (UTF-8 safe).
    pub fn next_boundary(&self, byte_offset: usize) -> usize {
        let len = self.value.len();
        if byte_offset >= len {
            return len;
        }
        let bytes = self.value.as_bytes();
        let mut i = byte_offset + 1;
        while i < len && (bytes[i] & 0b1100_0000) == 0b1000_0000 {
            i += 1;
        }
        i
    }

    // -- Selection / caret mutation ----------------------------------

    /// Insert `text` at the current caret and advance the caret.
    /// Used by Enter-on-text-area (inserts '\n') and any
    /// caller that wants to type a single chunk without going
    /// through the IME / action pipeline.
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.replace_text_in_range_bytes(None, text);
    }

    /// Collapse selection to a single caret position.
    pub fn move_to(&mut self, offset: usize) {
        let clamped = offset.min(self.value.len());
        self.caret = clamped;
        self.selection_start = clamped;
        self.selection_end = clamped;
    }

    /// Extend the selection to a new offset (keeps `selection_start`
    /// as the anchor and grows `selection_end`).
    pub fn select_to(&mut self, offset: usize) {
        let clamped = offset.min(self.value.len());
        self.caret = clamped;
        self.selection_end = clamped;
    }

    /// Replace the range `[start..end)` (UTF-8 bytes) with
    /// `new_text`. Updates caret, selection, and (if `fire_on_change`
    /// is true) the `on_change` callback is fired by the caller
    /// (the renderer's key handler or the `replace_text_in_range`
    /// platform pipeline).
    pub fn replace_text(&mut self, start: usize, end: usize, new_text: &str) {
        let start = start.min(self.value.len());
        let end = end.max(start).min(self.value.len());
        self.value.replace_range(start..end, new_text);
        let new_caret = start + new_text.len();
        self.caret = new_caret;
        self.selection_start = new_caret;
        self.selection_end = new_caret;
    }

    /// Apply a `Range<usize>` (UTF-8) replacement. Used by the
    /// platform IME / clipboard via `EntityInputHandler`. A
    /// commit (a non-`replace_and_mark_text_in_range` call
    /// that targets the marked range) implicitly clears the
    /// composition.
    pub fn replace_text_in_range_bytes(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
    ) -> bool {
        let before = self.value.clone();
        // Lookup order: IME-sent range → active marked range
        // (IME commit with no `replacementRange` still needs to
        // find the pinyin) → active selection. The marked
        // range wins over the selection because the caret sits
        // at the end of the marked text — falling back to the
        // selection would insert *after* the pinyin.
        let resolved = range.or_else(|| self.marked_range.clone()).or_else(|| {
            if !self.selected_range().is_empty() {
                Some(self.selected_range())
            } else {
                None
            }
        });
        // Decide the effective new text up front (honouring
        // `max_length`). This avoids the "apply, then truncate"
        // path which would leave the caret past the end.
        let effective = if let Some(cap) = self.max_length {
            let existing_len = match &resolved {
                Some(r) => self.value.len() - (r.end - r.start),
                None => self.value.len(),
            };
            let room = cap.saturating_sub(existing_len);
            &new_text[..new_text.len().min(room)]
        } else {
            new_text
        };
        match &resolved {
            Some(r) => self.replace_text(r.start, r.end, effective),
            None => self.replace_text(self.caret, self.caret, effective),
        }
        // A `replace_text_in_range` outside of an active
        // composition context ends any pending IME mark. The
        // platform's commit call always lands here, so this is
        // what actually clears the pinyin.
        self.marked_range = None;
        self.value != before
    }

    /// Compose (IME marked text). Replaces the active selection
    /// with `new_text` and records the marked range as the
    /// **span of the just-inserted text** (not the IME's
    /// `new_selected_range`, which is the cursor selection
    /// *within* the marked text). This matches v0.2's
    /// `TextEdit::replace_and_mark_text_in_range` semantics:
    /// after a `setMarkedText("ni", …, replacementRange)` call,
    /// the marked range is `replacementRange.start..start +
    /// new_text.len()`. The eventual commit
    /// (`insertText(text, replacementRange)`) sends
    /// `replacementRange` = the marked range, so the commit
    /// replaces the pinyin with the final text.
    pub fn replace_and_mark_text_in_range_bytes(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
    ) {
        // The actual range we replace follows the v0.2 lookup,
        // with one extra priority: an active IME composition
        // (marked range) wins over the active selection. The
        // IME on macOS sends `replacementRange = NSNotFound`
        // (None) when updating the *existing* composition —
        // in that case the new text must replace the marked
        // range, not be inserted at the caret (the caret is
        // already at the end of the marked text, so falling
        // back to the active selection would *append* the
        // new pinyin after the existing composition and
        // produce "sas" / "sa's" artefacts).
        let range = range
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range());
        let range_start = range.start.min(self.value.len());
        let range_end = range.end.max(range_start).min(self.value.len());

        // Apply the replacement. The just-inserted text
        // occupies `[range_start, range_start + new_text.len())`
        // in byte space — that's the new marked range.
        self.value.replace_range(range_start..range_end, new_text);
        let marked_start = range_start;
        let marked_end = range_start + new_text.len();
        self.caret = marked_end;
        if !new_text.is_empty() {
            self.marked_range = Some(marked_start..marked_end);
        } else {
            // Empty marked text (e.g. the IME sent an empty
            // composition to clear) — no composition.
            self.marked_range = None;
        }

        // The IME also tells us the cursor selection *within*
        // the new marked text. Translate it from UTF-16 to
        // bytes and offset by the marked start. If the IME
        // didn't provide one, the caret sits at the end of
        // the marked text.
        if let Some(sel_utf16) = new_selected_range {
            let start_in_marked = self
                .utf16_to_offset(sel_utf16.start)
                .saturating_sub(marked_start);
            let end_in_marked = self
                .utf16_to_offset(sel_utf16.end)
                .saturating_sub(marked_start);
            let sel_start = (marked_start + start_in_marked).min(marked_end);
            let sel_end = (marked_start + end_in_marked).min(marked_end);
            self.selection_start = sel_start;
            self.selection_end = sel_end;
        } else {
            self.selection_start = marked_end;
            self.selection_end = marked_end;
        }
    }

    // -- `EntityInputHandler` body methods (not the trait impl) -----

    /// UTF-8 version of `EntityInputHandler::text_for_range`.
    pub fn text_for_range_inner(&self, range_utf16: Range<usize>) -> (String, Range<usize>) {
        self.text_for_range_utf16(range_utf16)
    }

    /// UTF-8 version of `EntityInputHandler::selected_text_range`.
    pub fn selected_text_range_inner(&self) -> UTF16Selection {
        let byte_range = self.selected_range();
        let start = self.offset_to_utf16(byte_range.start);
        let end = self.offset_to_utf16(byte_range.end);
        // gpui's `UTF16Selection` keeps the head/end (which side
        // is the caret when the selection is non-empty). We treat
        // `selection_end` as the caret head, which matches the
        // rendering direction in `caret` above.
        UTF16Selection {
            range: start..end,
            reversed: false,
        }
    }

    /// UTF-8 version of `EntityInputHandler::bounds_for_range`.
    /// Returns `None` if we haven't been laid out yet
    /// (`last_layout` is `None`).
    pub fn bounds_for_range_inner(
        &self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
    ) -> Option<Bounds<Pixels>> {
        if !self.last_line_layouts.is_empty() {
            // Multi-line path: find every row the range touches
            // and union the pixel bounds. The returned Bounds
            // covers `[min_x, max_x] x [min_y, max_y]`, where
            // the y-range may span multiple rows.
            let range_bytes = self.range_from_utf16(&range_utf16);
            let line_height = self.last_line_height?;
            let mut min_x = Pixels::from(f32::INFINITY);
            let mut max_x = Pixels::from(f32::NEG_INFINITY);
            let mut min_y = Pixels::from(f32::INFINITY);
            let mut max_y = Pixels::from(f32::NEG_INFINITY);
            let mut any = false;
            for (i, line) in self.last_line_layouts.iter().enumerate() {
                let line_range = &self.last_line_byte_ranges[i];
                // The i-th row's text occupies
                // `[line_range.start, line_range.end)` in
                // `value`. The last row's end is the value
                // length; non-last rows include a trailing
                // '\n'. For the *display* of the line itself,
                // we want to clip the selection to
                // `[line_range.start, line_range.end)` (the
                // '\n' is not selectable as a visible glyph
                // on this row).
                let sel_start = range_bytes.start.max(line_range.start).min(line_range.end);
                let sel_end = range_bytes.end.max(line_range.start).min(line_range.end);
                if sel_start < sel_end {
                    any = true;
                    let col_start = sel_start - line_range.start;
                    let col_end = sel_end - line_range.start;
                    let start_x = line.x_for_index(col_start);
                    let end_x = line.x_for_index(col_end);
                    let y_top = (i as f32) * line_height;
                    let y_bottom = y_top + line_height;
                    min_x = min_x.min(start_x.min(end_x));
                    max_x = max_x.max(start_x.max(end_x));
                    min_y = min_y.min(y_top);
                    max_y = max_y.max(y_bottom);
                }
            }
            if !any {
                return None;
            }
            return Some(Bounds::from_corners(
                point(
                    element_bounds.left() + min_x - self.scroll_x,
                    element_bounds.top() + min_y,
                ),
                point(
                    element_bounds.left() + max_x - self.scroll_x,
                    element_bounds.top() + max_y,
                ),
            ));
        }

        let line = self.last_layout.as_ref()?;
        let range_bytes = self.range_from_utf16(&range_utf16);
        let start_x = line.x_for_index(range_bytes.start);
        let end_x = line.x_for_index(range_bytes.end);
        Some(Bounds::from_corners(
            point(
                element_bounds.left() + start_x - self.scroll_x,
                element_bounds.top(),
            ),
            point(
                element_bounds.left() + end_x - self.scroll_x,
                element_bounds.bottom(),
            ),
        ))
    }

    /// UTF-8 version of `EntityInputHandler::character_index_for_point`.
    pub fn character_index_for_point_inner(&self, point: gpui::Point<Pixels>) -> Option<usize> {
        if self.value.is_empty() {
            return Some(0);
        }

        let bounds = self.last_bounds.as_ref()?;
        let local = bounds.localize(&point)?;

        if !self.last_line_layouts.is_empty() {
            // Multi-line: figure out the row from the Y
            // coordinate (clamped to the last row if past the
            // bottom), then the column within that row from X.
            let line_height = self.last_line_height?;
            let row_count = self.last_line_layouts.len();
            let row = ((local.y / line_height).floor() as i64)
                .max(0)
                .min((row_count - 1) as i64) as usize;
            let line = &self.last_line_layouts[row];
            // `index_for_x` returns the closest glyph *byte*
            // index for the given x. Add `scroll_x` back so
            // horizontal scroll is transparent.
            let col_bytes = line
                .index_for_x(local.x + self.scroll_x)
                .unwrap_or(line.len());
            // Translate (row, col) to a byte offset in `value`.
            // The col is a byte offset *within the row's text*;
            // adding the row's start byte gives the absolute
            // offset. The row's last `byte_range.end` is the
            // value's byte just past the last char of the row
            // (for the last row, that's the value length; for
            // other rows, it's the position of the '\n').
            let row_start = self.last_line_byte_ranges[row].start;
            let byte_in_value = row_start + col_bytes;
            return Some(self.offset_to_utf16(byte_in_value));
        }

        let line = self.last_layout.as_ref()?;
        let utf8_index = line
            .index_for_x(local.x + self.scroll_x)
            .unwrap_or(line.len());
        Some(self.offset_to_utf16(utf8_index))
    }
}

// =====================================================================
// `EntityInputHandler for TextInputState` — the IME / clipboard
// pipeline. The 8 trait methods are thin wrappers around the
// `*_inner` methods above; the wrappers also call `cx.notify()`
// to trigger a re-render after a mutation.
// =====================================================================

impl EntityInputHandler for TextInputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let (text, adjusted) = self.text_for_range_inner(range_utf16);
        *adjusted_range = Some(adjusted);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(self.selected_text_range_inner())
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        // Return the UTF-16 of the active IME marked range.
        // The platform uses this to know where the composition
        // is when committing (or to query / replace it).
        self.marked_range.as_ref().map(|r| self.range_to_utf16(r))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // The platform is telling us to drop the composition
        // (e.g. focus lost, or the user pressed Escape mid-IME).
        // Clear the marked range and the selection; keep the
        // value as-is so the partially-typed pinyin is
        // preserved (the caller can decide to revert if it
        // wants).
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let before = self.value.clone();
        // On commit, the IME either passes the marked range
        // explicitly or we use our tracked one. If neither
        // is available, fall back to the active selection.
        let range = range_utf16
            .map(|r| self.range_from_utf16(&r))
            .or_else(|| self.marked_range.clone())
            .or_else(|| {
                if !self.selected_range().is_empty() {
                    Some(self.selected_range())
                } else {
                    None
                }
            });
        self.replace_text_in_range_bytes(range, new_text);
        // A commit clears the composition.
        self.marked_range = None;
        if self.value != before
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let before = self.value.clone();
        let range = range_utf16.map(|r| self.range_from_utf16(&r));
        let new_sel = new_selected_range_utf16.map(|r| self.range_from_utf16(&r));
        self.replace_and_mark_text_in_range_bytes(range, new_text, new_sel);
        if self.value != before
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        self.bounds_for_range_inner(range_utf16, element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        self.character_index_for_point_inner(point)
    }
}

// =====================================================================
// `Focusable for TextInputState` — the platform uses this to find
// the focus handle for the entity. Without this, `track_focus` and
// `Window::handle_input(&focus_handle, ...)` can't find the
// handle.
// =====================================================================

impl Focusable for TextInputState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// =====================================================================
// Action handlers. Each takes `(&Action, &mut Window, &mut
// Context<Self>)`. The `action_handler!` macro wires them into
// `.on_action` closures in the renderer.
// =====================================================================

impl TextInputState {
    /// The renderer calls this when the user focuses the input
    /// via mouse / tab. (It's the entry point for cursor blink
    /// setup; the actual focus call is `Window::focus(&handle)`,
    /// done by the platform once `track_focus` is registered.)
    pub fn focus_in(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // The cursor-blink task is started by the renderer (which
        // holds a `Window` to spawn the timer). We bump the epoch
        // so any previous task exits.
        self.cursor_blink_epoch = self.cursor_blink_epoch.wrapping_add(1);
        self.cursor_visible = true;
    }

    pub fn left(&mut self, _: &Left, _window: &mut Window, cx: &mut Context<Self>) {
        if self.has_selection() {
            // Collapse to the start of the selection.
            let start = self.selected_range().start;
            self.move_to(start);
        } else {
            self.move_to(self.prev_boundary(self.caret));
        }
        cx.notify();
    }

    pub fn right(&mut self, _: &Right, _window: &mut Window, cx: &mut Context<Self>) {
        if self.has_selection() {
            let end = self.selected_range().end;
            self.move_to(end);
        } else {
            self.move_to(self.next_boundary(self.caret));
        }
        cx.notify();
    }

    pub fn select_left(&mut self, _: &SelectLeft, _window: &mut Window, cx: &mut Context<Self>) {
        let new_end = self.prev_boundary(self.caret);
        self.select_to(new_end);
        cx.notify();
    }

    pub fn select_right(&mut self, _: &SelectRight, _window: &mut Window, cx: &mut Context<Self>) {
        let new_end = self.next_boundary(self.caret);
        self.select_to(new_end);
        cx.notify();
    }

    pub fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0);
        self.select_to(self.value.len());
        cx.notify();
    }

    pub fn home(&mut self, _: &Home, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0);
        cx.notify();
    }

    pub fn end(&mut self, _: &End, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.value.len());
        cx.notify();
    }

    pub fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.value.clone();
        if self.has_selection() {
            let r = self.selected_range();
            self.replace_text(r.start, r.end, "");
        } else if self.caret > 0 {
            let prev = self.prev_boundary(self.caret);
            self.replace_text(prev, self.caret, "");
        }
        // A direct keyboard backspace (not via the IME
        // pipeline) cancels any active composition.
        self.marked_range = None;
        if self.value != before
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
        cx.notify();
    }

    pub fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.value.clone();
        if self.has_selection() {
            let r = self.selected_range();
            self.replace_text(r.start, r.end, "");
        } else if self.caret < self.value.len() {
            let next = self.next_boundary(self.caret);
            self.replace_text(self.caret, next, "");
        }
        self.marked_range = None;
        if self.value != before
            && let Some(cb) = self.on_change.as_ref()
        {
            cb(&self.value, window, cx);
        }
        cx.notify();
    }

    pub fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            // Single-line inputs collapse newlines to spaces so
            // pasted multi-line text doesn't break the layout.
            // Multi-line inputs (text_area) keep them by
            // setting `paste_newlines = true` in the renderer.
            let text = if self.paste_newlines {
                text.to_string()
            } else {
                text.replace('\n', " ")
            };
            let before = self.value.clone();
            self.replace_text_in_range_bytes(None, &text);
            self.marked_range = None;
            if self.value != before
                && let Some(cb) = self.on_change.as_ref()
            {
                cb(&self.value, window, cx);
            }
        }
        cx.notify();
    }

    pub fn copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        if self.has_selection() {
            let r = self.selected_range();
            let text = self.value[r.clone()].to_string();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
    }

    pub fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if self.has_selection() {
            let r = self.selected_range();
            let text = self.value[r.clone()].to_string();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            let before = self.value.clone();
            self.replace_text(r.start, r.end, "");
            if self.value != before
                && let Some(cb) = self.on_change.as_ref()
            {
                cb(&self.value, window, cx);
            }
        }
        cx.notify();
    }

    pub fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    // -- Mouse handlers (used by the renderer's on_mouse_*) -------

    pub fn on_mouse_down(
        &mut self,
        position: gpui::Point<Pixels>,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        // Map the click point to a UTF-16 offset, then to a byte
        // offset, then move the caret. We use the cached
        // `last_layout` / `last_bounds` from the previous paint.
        if let Some(utf16) = self.character_index_for_point_inner(position) {
            let byte = self.utf16_to_offset(utf16);
            self.move_to(byte);
        }
        // Let the platform focus the input.
        window.focus(&self.focus_handle);
    }

    pub fn on_mouse_up(
        &mut self,
        _event: &gpui::MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.is_selecting = false;
    }

    pub fn on_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        if self.is_selecting
            && let Some(utf16) = self.character_index_for_point_inner(event.position)
        {
            let byte = self.utf16_to_offset(utf16);
            self.select_to(byte);
        }
    }
}

// =====================================================================
// `TextInputProps` — pure data carrier. No `state` (the renderer
// mints it via `use_keyed_state`), no `focus_handle` (lives in the
// state), no `apply` (the renderer builds the full element tree).
// =====================================================================

#[derive(Clone)]
pub struct TextInputProps {
    /// Stable element id used by `window.use_keyed_state` to
    /// persist the `TextInputState` across re-renders.
    pub id: gpui::ElementId,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<TextChangeCallback>,
    pub on_submit: Option<TextChangeCallback>,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_text_color: Option<Hsla>,
}

/// Build a fresh `TextInputProps`. Unlike v0.2's `text_input(id)`,
/// the v0.3 factory takes only the `id` — the focus handle and
/// state are minted by the renderer in `default_render`.
///
/// (The headless crate is now purely declarative; visual side
/// effects like `cx.focus_handle()` belong in the renderer.)
pub fn text_input(id: impl Into<gpui::ElementId>) -> TextInputProps {
    TextInputProps {
        id: id.into(),
        placeholder: String::new(),
        disabled: false,
        max_length: None,
        on_change: None,
        on_submit: None,
        has_custom_bg: false,
        has_custom_border: false,
        has_custom_focus_border: false,
        custom_bg: None,
        custom_border: None,
        custom_focus_border: None,
        custom_text_color: None,
    }
}

impl TextInputProps {
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn max_length(mut self, v: usize) -> Self {
        self.max_length = Some(v);
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn on_submit<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&str, &mut Window, &mut App),
    {
        self.on_submit = Some(Arc::new(f));
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

    /// Render the text input using the registered `TextInputRenderer`.
    ///
    /// Data flow is one-way: hand the full `TextInputProps` to
    /// the registered renderer's `compose` and return the
    /// resulting `AnyElement`. No token values are pulled from
    /// the renderer here — every visual decision (bg, border,
    /// padding, radius, hover/active, text colour, the inner
    /// `TextInputElement` placement, the keymap wiring, focus
    /// tracking) lives in the renderer's `compose`.
    pub fn render(self, cx: &mut App, window: &mut Window) -> gpui::AnyElement {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::TextInput as TextInputMarker;
        use crate::renderer::text_input::TextInputRenderer;

        let r: Arc<dyn TextInputRenderer> = cx
            .renderer_arc::<TextInputMarker, dyn TextInputRenderer>()
            .expect("TextInputRenderer registered")
            .clone();

        r.compose(&self, cx, window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a test state without minting a real focus handle.
    /// The tests only exercise data helpers (UTF-16, boundaries,
    /// range ops) and never touch the focus handle. We leak
    /// each test state via `Box::leak` so the `FocusHandle`
    /// field's `Drop` never runs on a null-pointer `FocusMap`.
    /// Memory leak is fine for tests.
    fn test_state(value: &str) -> &'static mut TextInputState {
        let mut state = std::mem::MaybeUninit::<TextInputState>::zeroed();
        let ptr = state.as_mut_ptr();
        unsafe {
            (*ptr).value = value.into();
            (*ptr).caret = 0;
            (*ptr).selection_start = 0;
            (*ptr).selection_end = 0;
            (*ptr).placeholder = SharedString::new_static("");
            (*ptr).scroll_x = Pixels::ZERO;
            (*ptr).is_selecting = false;
            (*ptr).cursor_visible = true;
            (*ptr).cursor_blink_epoch = 0;
            (*ptr).max_length = None;
            (*ptr).on_change = None;
            (*ptr).on_submit = None;
            // focus_handle left zeroed — tests never read it.
            // Leak so the `FocusHandle` field's `Drop` never
            // runs (which would walk into a null FocusMap Arc).
            Box::leak(Box::new(state.assume_init()))
        }
    }

    #[test]
    fn prev_next_boundary_walks_chars() {
        let s = test_state("héllo");
        // h é l l o  (é = 2 bytes)
        // 0 1    3 4 5
        assert_eq!(s.prev_boundary(5), 4); // before 'o' = start of last 'l'
        assert_eq!(s.prev_boundary(4), 3); // before second 'l' = start of first 'l'
        assert_eq!(s.prev_boundary(3), 1); // before first 'l' = start of 'é'
        assert_eq!(s.next_boundary(1), 3); // after 'é' = start of first 'l'
    }

    #[test]
    fn offset_to_utf16_handles_surrogates() {
        // 𝄞 (musical G clef) = U+1D11E = 4 UTF-8 bytes, 2 UTF-16 code units
        let s = test_state("a𝄞b");
        // 'a' = byte 0, '𝄞' = bytes 1..5, 'b' = byte 5
        assert_eq!(s.offset_to_utf16(0), 0);
        assert_eq!(s.offset_to_utf16(1), 1); // before 𝄞
        assert_eq!(s.offset_to_utf16(5), 3); // after 𝄞 (a=1, 𝄞=2)
        assert_eq!(s.offset_to_utf16(6), 4); // after b
    }

    #[test]
    fn text_for_range_returns_substring() {
        let s = test_state("hello");
        let (text, adj) = s.text_for_range_utf16(1..4);
        assert_eq!(text, "ell");
        assert_eq!(adj, 1..4);
    }

    #[test]
    fn replace_text_collapses_selection() {
        let s = &mut *test_state("hello");
        s.caret = 5;
        s.selection_start = 1;
        s.selection_end = 4;
        s.replace_text(1, 4, "XY");
        assert_eq!(s.value, "hXYo");
        assert_eq!(s.caret, 3);
        assert_eq!(s.selection_start, 3);
        assert_eq!(s.selection_end, 3);
    }

    #[test]
    fn backspace_at_zero_is_noop() {
        let s = &mut *test_state("hi");
        s.move_to(0);
        // No selection, caret=0 → replace_text(0,0,"") is a no-op
        s.replace_text(0, 0, "");
        assert_eq!(s.value, "hi");
        assert_eq!(s.caret, 0);
    }

    #[test]
    fn move_to_clamps() {
        let s = &mut *test_state("hi");
        s.move_to(100);
        assert_eq!(s.caret, 2);
    }

    #[test]
    fn selected_range_is_normalised() {
        let s = &mut *test_state("hello");
        s.selection_start = 4;
        s.selection_end = 1;
        assert_eq!(s.selected_range(), 1..4);
    }

    #[test]
    fn set_value_resets_state() {
        let s = &mut *test_state("old");
        s.caret = 2;
        s.selection_start = 0;
        s.selection_end = 2;
        s.set_value("fresh");
        assert_eq!(s.value, "fresh");
        assert_eq!(s.caret, 5);
        assert_eq!(s.selection_start, 0);
        assert_eq!(s.selection_end, 0);
    }

    #[test]
    fn ime_marked_range_round_trip() {
        // Simulate the IME pipeline:
        //   1. `replace_and_mark_text_in_range` marks "ni" as
        //      composition. The marked range is the byte span
        //      of the just-inserted text — `replacementRange`
        //      start..start + new_text.len() (NOT the IME's
        //      `new_selected_range`, which is the cursor
        //      selection *within* the marked text).
        //   2. `marked_text_range` returns the UTF-16 span so
        //      the platform can drive the commit.
        //   3. Commit via `replace_text_in_range` with the
        //      marked range — the pinyin is replaced.
        let s = &mut *test_state("");
        s.caret = 0;
        // IME: insert "ni" at position 0, mark 0..2 as composition.
        s.replace_and_mark_text_in_range_bytes(Some(0..0), "ni", None);
        assert_eq!(s.value, "ni");
        assert_eq!(s.marked_range.as_ref().unwrap().clone(), 0..2);
        // The IME's commit: `insertText(0..2, "你")` →
        // `replace_text_in_range(Some(0..2), "你")`.
        s.replace_text_in_range_bytes(Some(0..2), "你");
        assert_eq!(s.value, "你");
        assert!(s.marked_range.is_none());
    }

    #[test]
    fn ime_marked_range_uses_insertion_span_not_selection() {
        // Regression test: the IME sends a `new_selected_range`
        // that is the cursor position WITHIN the marked text
        // (typically `0..0` when the user just inserted pinyin).
        // The marked range itself must be the byte span of
        // the inserted text — using the IME's selection as
        // the marked range would result in a zero-width mark
        // and the commit would fail to replace the pinyin.
        let s = &mut *test_state("");
        // IME inserts "你好" at 0..0 and tells us the
        // selection within the new text is also 0..0
        // (cursor at the start of the marked text).
        s.replace_and_mark_text_in_range_bytes(Some(0..0), "你好", Some(0..0));
        assert_eq!(s.value, "你好");
        assert_eq!(s.marked_range.as_ref().unwrap().clone(), 0..6);
        assert_eq!(s.selection_start, 0);
        assert_eq!(s.selection_end, 0);
        // Commit replaces the marked span (0..6 bytes).
        s.replace_text_in_range_bytes(Some(0..6), "X");
        assert_eq!(s.value, "X");
    }

    #[test]
    fn ime_extending_composition_replaces_marked_not_appends() {
        // Regression test: when the IME updates an existing
        // composition (e.g. user types "s" then "a" while the
        // IME has "s" marked), it sends `setMarkedText` with
        // `replacementRange = NSNotFound` (None) to mean
        // "update the existing composition". Falling back to
        // the active selection would *append* the new pinyin
        // after the existing one (giving "ssa" or "sas"),
        // because the caret sits at the end of the marked
        // text. The fix: prefer `marked_range` over the
        // active selection when `replacementRange` is None.
        let s = &mut *test_state("");
        // Step 1: user types 's'. IME: setMarkedText("s", …,
        // replacementRange: 0..0). value = "s", marked = 0..1.
        s.replace_and_mark_text_in_range_bytes(Some(0..0), "s", Some(0..0));
        assert_eq!(s.value, "s");
        assert_eq!(s.marked_range.as_ref().unwrap().clone(), 0..1);
        // Step 2: user types 'a'. IME: setMarkedText("sa", …,
        // replacementRange: NSNotFound). The IME expects the
        // existing marked "s" to be REPLACED by "sa", not
        // appended after.
        s.replace_and_mark_text_in_range_bytes(None, "sa", Some(0..0));
        assert_eq!(s.value, "sa");
        assert_eq!(s.marked_range.as_ref().unwrap().clone(), 0..2);
        // Commit.
        s.replace_text_in_range_bytes(Some(0..2), "X");
        assert_eq!(s.value, "X");
    }
}
