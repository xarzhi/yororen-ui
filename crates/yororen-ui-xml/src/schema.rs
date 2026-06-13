//! Schema for the built-in XML tags accepted by the `xml!` macro.
//!
//! Each entry maps an XML tag to the headless component it
//! represents plus the prop/event vocabulary the codegen is
//! allowed to emit. The validator in [`crate::codegen`]
//! cross-checks every `AstElement` against this table and
//! reports a friendly `unknown tag` / `unknown attribute`
//! error if a user typo'd something.
//!
//! Adding a new built-in is a 3-step change:
//!
//! 1. Add a [`ComponentDef`] entry here.
//! 2. Add a codegen arm in [`crate::codegen`] for the new tag.
//! 3. Add a doc test under `xml! { ... }` in the
//!    `counter-xml-demo` crate.
//!
//! ## Container vs. leaf
//!
//! - **Container** (e.g. `Column`, `Row`, `Div`) starts from
//!   `gpui::div()`, applies style attrs (the gpui `Styled`
//!   trait's setter methods), and takes children via
//!   `.child(any_element)`.
//!
//! - **Leaf** (e.g. `Label`, `Button`) calls a headless factory
//!   (e.g. `yororen_ui_core::headless::button::button`),
//!   applies prop setters, and either `.render(cx)`s (for
//!   default styling) or stays as a pure `Props` value the
//!   caller composes via `.apply(div())`.

use proc_macro2::Span;

/// A schema entry for a single XML tag.
#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub tag: &'static str,
    pub kind: ComponentKind,
    /// Optional doc string surfaced in compiler errors.
    pub doc: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    /// A `Div`-based container. The macro generates
    /// `div().<style>(...)<style>(...)...<child>(child)...`.
    Container(ContainerDef),
    /// A headless component that produces a single element via
    /// `.render(cx)` (or `.apply(div())` for the headless path).
    Leaf(LeafDef),
    /// A pseudo-tag that produces Rust control flow at
    /// codegen time. It never produces an element of its own.
    ControlFlow(ControlFlowDef),
    /// A tag not known to the schema. The codegen emits
    /// a runtime lookup against [`crate::runtime::lookup`]
    /// — useful for user-registered custom components.
    /// Compile-time validation of attributes / events is
    /// skipped for these tags.
    RuntimeLeaf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerDef {
    /// Initial flex / layout methods. They are emitted as
    /// `.flex()`, `.flex_col()`, `.flex_row()`, etc. when the
    /// corresponding XML attr is present.
    ///
    /// `attr` is the XML attribute (e.g. `"col"`), `method`
    /// is the chained method (e.g. `"flex_col"`).
    pub fixed_methods: &'static [(&'static str, &'static str)],
    /// Hint printed when an unknown prop is used on this
    /// container. (`"the gpui Styled trait"`)
    pub style_hint: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafDef {
    /// Rust path to the headless factory function
    /// (e.g. `yororen_ui_core::headless::button::button`).
    pub factory: &'static str,
    /// Factory args **between** the `id` and the optional
    /// `&mut App` / `&mut Context<T>`. Each entry corresponds
    /// to one positional argument; the macro reads the
    /// corresponding XML attribute (named by `attr`) and
    /// splices it into the call.
    ///
    /// `Text` entries fall back to the element's inner text
    /// content when the attribute is absent. `Custom` entries
    /// require the attribute to be present.
    pub extra_args: &'static [ExtraArg],
    /// How the headless props get turned into an element.
    pub render: RenderMode,
    /// Whether the factory takes `&mut App` as the last arg
    /// (most do — the macro passes `cx` here). Set to `false`
    /// for factories that don't need an app context (e.g. the
    /// newer text_input that mints state internally).
    pub needs_app: bool,
    /// Prop vocabulary: (xml_attr, builder_method, value_transform).
    pub props: &'static [PropDef],
    /// Event vocabulary: (xml_attr like `"on_click"`, builder_method).
    pub events: &'static [(&'static str, &'static str)],
    /// Whether the leaf can wrap `.child(text)` after the render
    /// call (true for `Button` so users can do
    /// `<Button>Click me</Button>`).
    pub supports_text_child: bool,
}

/// How the codegen should turn an XML attribute value into a
/// Rust expression for a prop setter call.
///
/// - [`PropValue::String`] — `attr="hello"` becomes
///   `"hello".into()` (works for `SharedString`, `String`,
///   `&str`, etc.).
/// - [`PropValue::Bool`] — `attr="true"` becomes `true`,
///   `attr="false"` becomes `false`. Anything else is an
///   error.
/// - [`PropValue::Variant`] — `attr="primary"` becomes
///   `::yororen_ui::ActionVariantKind::Primary`. Only the
///   three built-in variants are supported for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropValue {
    String,
    Bool,
    Variant,
    /// A zero-arg setter (`fn wrap(self) -> Self`). The
    /// codegen emits `.setter()` when the attribute is
    /// present (regardless of value). Used for flag-style
    /// toggles like `Label.wrap`.
    Flag,
    /// The generator couldn't classify the type — the user
    /// should review. The codegen treats it as `String`
    /// (i.e. `(#raw).to_string()`).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropDef {
    pub name: &'static str,
    pub setter: &'static str,
    pub value: PropValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtraArg {
    pub kind: ExtraArgKind,
    /// The XML attribute that holds the value (for `Custom`).
    /// For `Text`, the macro synthesises a `String` from the
    /// `text` XML attribute (if present) or the inner text
    /// content of the element.
    pub attr: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraArgKind {
    /// Use the literal / brace expression of the `attr` XML
    /// attribute, wrapped in `.into()` to coerce to the
    /// factory's expected string type.
    Custom,
    /// The label's text: prefer the `text` XML attribute,
    /// fall back to the element's inner text content.
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// The default path: factory + props + `.render(cx)` →
    /// `Stateful<Div>` (or `AnyElement` for some inputs).
    /// The macro appends `.into_any_element()` so the result
    /// can be used as a child of a container.
    Default,
    /// The headless path: factory + props + `.apply(div())`
    /// — caller composes the visual entirely.
    /// The macro appends `.into_any_element()`.
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowDef {
    /// `<If condition={expr}>` — emits `if expr { ... }`.
    If,
    /// `<ElseIf condition={expr}>` — emits `else if expr { ... }`.
    ElseIf,
    /// `<Else>` — emits `else { ... }`.
    Else,
    /// `<For each={expr} let:item>{...}</For>` — emits
    /// `expr.into_iter().map(|item| { ... }).collect::<Vec<_>>()`.
    For,
    /// `<Fragment>` — emits a `Vec<AnyElement>` (for now,
    /// used only as a grouping construct).
    Fragment,
    /// `<Include src="path.xml" />` — at compile time,
    /// read the file and splice its children in. The
    /// `src` path is resolved relative to the calling
    /// source file (same convention as `xml_file!`).
    Include,
    /// `<Template name="X">…</Template>` — define a
    /// named block of XML. The body is captured but
    /// only emitted when referenced by a `Slot` match
    /// (or by `<UseTemplate name="X">` — coming soon).
    /// For the MVP, `<Template>` simply emits its
    /// children in place (compile-time inlining).
    Template,
    /// `<Slot/>` — placeholder for the caller's
    /// children. For the MVP, `<Slot/>` is a no-op
    /// (emits nothing); the children of a `Template`
    /// are inlined as-is. A future revision will wire
    /// named slots and caller-side slot-filling.
    Slot,
    /// `<Match on={expr}>` followed by `<Case pattern={...}>…</Case>`
    /// siblings — emits a Rust `match` expression. The
    /// last `<Case _>` (with literal `_` as the pattern
    /// attribute) becomes the wildcard arm.
    Match,
    /// `<Case pattern={...}>…</Case>` — a single arm of
    /// a `<Match>`. `pattern` is the Rust pattern (e.g.
    /// `Status::Loading` or `_` for wildcard).
    Case,
    /// `<State name="count" default="0">…</State>` — declare a
    /// local `Entity<T>` inside the `Render::render` closure.
    /// `default` is a stringified literal that becomes the
    /// initial value via `cx.new(|_| …)`. Children are emitted
    /// in the closure scope where `name` resolves to the
    /// entity.
    State,
}

impl ComponentDef {
    /// Look up a tag by name (case-sensitive).
    pub fn lookup(tag: &str) -> Option<&'static ComponentDef> {
    BUILTINS.iter().find(|c| c.tag == tag)
    }
}

// Helper for tests / callers that need a static list of
// string tags. Implemented as a function instead of a
// `[&'static str]` to dodge the const-context limitations
// of the `BUILTINS` table.
pub const fn builtin_tags() -> &'static [&'static str] {
    &[
        "Button", "Column", "Div", "Else", "ElseIf", "For", "Fragment", "If", "Label", "Row",
        "Stack",
    ]
}

/// All built-in tags. Keep this list sorted alphabetically for
/// easy diffs. The validator iterates it linearly; for the
/// MVP that's fine. (Phase 3 can switch to a `phf` map if
/// the table grows past ~100 entries.)
pub static BUILTINS: &[ComponentDef] = &[
    ComponentDef {
        tag: "Button",
        kind: ComponentKind::Leaf(LeafDef {
            factory: "::yororen_ui::headless::button::button",
            extra_args: &[],
            render: RenderMode::Default,
            needs_app: true,
            props: &[
                PropDef { name: "caption", setter: "caption", value: PropValue::String },
                PropDef { name: "variant", setter: "variant", value: PropValue::Variant },
                PropDef { name: "disabled", setter: "disabled", value: PropValue::Bool },
                PropDef { name: "clickable", setter: "clickable", value: PropValue::Bool },
                // `icon` / `icon_size` are typed `IconSource` /
                // `gpui::Pixels` in the headless. Until we have
                // dedicated XML representations, we accept them
                // as opaque strings and let the user pass them
                // through as Rust expressions.
                PropDef { name: "icon", setter: "icon", value: PropValue::Unknown },
                PropDef { name: "icon_size", setter: "icon_size", value: PropValue::Unknown },
            ],
            events: &[("on_click", "on_click")],
            supports_text_child: true,
        }),
        doc: "Headless `Button` — see `yororen_ui_core::headless::button`.",
    },
    ComponentDef {
        tag: "Case",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Case),
        doc: "Match arm — `<Case pattern={expr}>…</Case>`.",
    },
    ComponentDef {
        tag: "Match",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Match),
        doc: "Pattern match — `<Match on={expr}><Case>…</Case></Match>`.",
    },
    ComponentDef {
        tag: "State",
        kind: ComponentKind::ControlFlow(ControlFlowDef::State),
        doc: "Declare a local `Entity<T>` — `<State name=\"count\" default=\"0\">…</State>`.",
    },
    ComponentDef {
        tag: "Column",
        kind: ComponentKind::Container(ContainerDef {
            fixed_methods: &[("col", "flex_col")],
            style_hint: "the gpui Styled trait (`.flex`, `.items_center`, `.gap_3()`, …)",
        }),
        doc: "Vertical flex container — `div().flex().flex_col()`.",
    },
    ComponentDef {
        tag: "Div",
        kind: ComponentKind::Container(ContainerDef {
            fixed_methods: &[],
            style_hint: "the gpui Styled trait (`.flex`, `.gap_3()`, …)",
        }),
        doc: "Plain `div()` — no flex by default.",
    },
    ComponentDef {
        tag: "Else",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Else),
        doc: "Else branch of an `<If>`.",
    },
    ComponentDef {
        tag: "ElseIf",
        kind: ComponentKind::ControlFlow(ControlFlowDef::ElseIf),
        doc: "Else-if branch of an `<If>`.",
    },
    ComponentDef {
        tag: "For",
        kind: ComponentKind::ControlFlow(ControlFlowDef::For),
        doc: "Iterate a Rust expression: `<For each={items} let:item>…</For>`.",
    },
    ComponentDef {
        tag: "Fragment",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Fragment),
        doc: "Group children without producing an element wrapper.",
    },
    ComponentDef {
        tag: "Include",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Include),
        doc: "Inline another XML file at compile time: `<Include src=\"toolbar.xml\" />`.",
    },
    ComponentDef {
        tag: "Template",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Template),
        doc: "Define a reusable XML template. Use `<Slot/>` to mark insertion points.",
    },
    ComponentDef {
        tag: "Slot",
        kind: ComponentKind::ControlFlow(ControlFlowDef::Slot),
        doc: "Placeholder inside a `<Template>` — replaced by the caller's children at the matching position.",
    },
    ComponentDef {
        tag: "If",
        kind: ComponentKind::ControlFlow(ControlFlowDef::If),
        doc: "Conditional rendering: `<If condition={cond}>…</If>`.",
    },
    ComponentDef {
        tag: "Label",
        kind: ComponentKind::Leaf(LeafDef {
            factory: "::yororen_ui::headless::label::label",
            extra_args: &[ExtraArg {
                kind: ExtraArgKind::Text,
                attr: "text",
            }],
            render: RenderMode::Default,
            needs_app: true,
            props: &[
                PropDef { name: "strong", setter: "strong", value: PropValue::Bool },
                PropDef { name: "muted", setter: "muted", value: PropValue::Bool },
                PropDef { name: "mono", setter: "mono", value: PropValue::Bool },
                PropDef { name: "inherit_color", setter: "inherit_color", value: PropValue::Bool },
                PropDef { name: "ellipsis", setter: "ellipsis", value: PropValue::Bool },
                PropDef { name: "wrap", setter: "wrap", value: PropValue::Bool },
                PropDef { name: "max_lines", setter: "max_lines", value: PropValue::Unknown },
            ],
            events: &[],
            supports_text_child: false,
        }),
        doc: "Headless `Label` — see `yororen_ui_core::headless::label`.",
    },
    ComponentDef {
        tag: "Row",
        kind: ComponentKind::Container(ContainerDef {
            fixed_methods: &[("row", "flex_row")],
            style_hint: "the gpui Styled trait (`.flex`, `.items_center`, `.gap_3()`, …)",
        }),
        doc: "Horizontal flex container — `div().flex().flex_row()`.",
    },
    ComponentDef {
        tag: "Stack",
        kind: ComponentKind::Container(ContainerDef {
            fixed_methods: &[],
            style_hint: "the gpui Styled trait (`.relative()`, `.absolute()`, …)",
        }),
        doc: "Plain `div()` — use `.relative()` / `.absolute()` for stacking.",
    },
];

/// Reserved XML attribute names consumed by the macro itself
/// (control flow, model alias, etc.). They're not props.
pub const RESERVED_ATTRS: &[&str] = &[
    "id",         // element id (passed as the first factory arg)
    "condition",  // <If condition={...}>
    "each",       // <For each={...}>
    "let",        // <For let:item>
    "model",      // alias for the surrounding ViewModel
    "key",        // reserved for Phase 2 keyed For
    "if",         // reserved for future inline-if
    "slot",       // reserved for Phase 2 templates
];

/// Reserved "fixed" attribute → method pairs that are valid on
/// every container (and don't need to be re-declared in each
/// `ContainerDef::fixed_methods`).
///
/// These are emitted **before** the `ContainerDef::fixed_methods`,
/// so `Column` with `flex col` is `.flex().flex_col()` in that
/// order.
pub const GLOBAL_CONTAINER_METHODS: &[(&str, &str)] = &[
    ("flex", "flex"),
];

/// Whether a string is a known shorthand suffix that we
/// should expand to a literal method call. Examples:
///
/// - `gap_3` → `.gap_3()`
/// - `p_4` → `.p_4()`
/// - `w_full` → `.w_full()`
///
/// Anything not in this table is rejected on containers
/// (with a hint pointing to the gpui `Styled` trait) — we
/// don't try to invent methods at macro time.
pub fn is_known_shorthand_method(name: &str) -> bool {
    matches!(
        name,
        // flex / layout
        "flex" | "flex_col" | "flex_row" | "flex_col_reverse" | "flex_row_reverse"
        | "flex_wrap" | "flex_wrap_reverse" | "flex_nowrap"
        | "flex_1" | "flex_auto" | "flex_initial" | "flex_none"
        | "flex_grow" | "flex_shrink" | "flex_shrink_0"
        // alignment
        | "items_start" | "items_end" | "items_center" | "items_baseline" | "items_stretch"
        | "justify_start" | "justify_end" | "justify_center"
        | "justify_between" | "justify_around" | "justify_evenly"
        // size
        | "w_full" | "h_full" | "size_full"
        | "min_w_full" | "min_h_full" | "max_w_full" | "max_h_full"
        // overflow / display
        | "overflow_hidden" | "overflow_x_hidden" | "overflow_y_hidden"
        | "visible" | "invisible" | "block" | "inline_block" | "inline_flex"
        | "hidden"
        // position
        | "relative" | "absolute"
        // cursor
        | "cursor_default" | "cursor_pointer" | "cursor_text" | "cursor_move"
        | "cursor_not_allowed" | "cursor_grab" | "cursor_grabbing"
        // border
        | "border" | "border_1" | "border_2" | "border_4" | "border_8"
        | "border_t" | "border_b" | "border_l" | "border_r"
        | "border_x" | "border_y"
        | "rounded" | "rounded_sm" | "rounded_md" | "rounded_lg" | "rounded_xl" | "rounded_full"
        | "rounded_none"
        // shadow
        | "shadow_sm" | "shadow_md" | "shadow_lg" | "shadow_xl" | "shadow_none"
        // visibility
        | "opacity_0" | "opacity_25" | "opacity_50" | "opacity_75" | "opacity_100"
    ) || is_spacing_shorthand(name)
}

/// Spacing shorthands for `gap`, `p`, `m`, `px`, `py`, `pt`,
/// `pb`, `pl`, `pr`, `mx`, `my`, `mt`, `mb`, `ml`, `mr`, and
/// their `gap_x_` / `gap_y_` siblings. Each numeric / textual
/// suffix maps to a real gpui method.
pub fn is_spacing_shorthand(name: &str) -> bool {
    const NUMERIC: &[&str] = &[
        "0", "0p5", "1", "1p5", "2", "2p5", "3", "3p5", "4", "5", "6", "7", "8", "9", "10", "11",
        "12", "16", "20", "24", "32", "40", "48", "56", "64", "72", "80", "96",
    ];
    const FULL: &[&str] = &["full", "1_2", "1_3", "2_3", "1_4", "3_4", "1_5", "2_5", "3_5", "4_5", "1_6", "5_6", "1_12"];

    let prefix_end = if let Some(stripped) = name.strip_suffix("_full") {
        Some(stripped)
    } else if let Some(idx) = name.rfind('_') {
        let (prefix, suffix) = name.split_at(idx);
        let suffix = &suffix[1..];
        if NUMERIC.contains(&suffix) || FULL.contains(&suffix) {
            Some(prefix)
        } else {
            None
        }
    } else {
        None
    };

    match prefix_end {
        Some(prefix) => matches!(
            prefix,
            "gap" | "gap_x" | "gap_y"
            | "p" | "px" | "py" | "pt" | "pb" | "pl" | "pr"
            | "m" | "mx" | "my" | "mt" | "mb" | "ml" | "mr"
            | "w" | "h" | "size" | "min_w" | "min_h" | "max_w" | "max_h"
            | "inset"
        ),
        None => false,
    }
}

/// Whether `name` is a **prefix** that takes a numeric /
/// textual suffix (e.g. `gap_3`, `p_4`, `w_full`). The full
/// method name is `<name>_<suffix>`. This is used by the
/// codegen to expand `gap="3"` to `.gap_3()`.
pub fn is_spacing_prefix(name: &str) -> bool {
    matches!(
        name,
        "gap" | "gap_x" | "gap_y"
        | "p" | "px" | "py" | "pt" | "pb" | "pl" | "pr"
        | "m" | "mx" | "my" | "mt" | "mb" | "ml" | "mr"
        | "w" | "h" | "size" | "min_w" | "min_h" | "max_w" | "max_h"
        | "inset"
    )
}

#[allow(dead_code)]
pub(crate) fn span_call(_span: Span) -> Span {
    Span::call_site()
}
