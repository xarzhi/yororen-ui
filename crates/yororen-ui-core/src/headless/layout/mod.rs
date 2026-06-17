//! High-level layout primitives — pure gpui builder sugar.
//!
//! These components do **not** have renderer traits. They expand
//! directly to equivalent gpui builder calls, reading spacing
//! and inset values from the active theme's `tokens.spacing`
//! paths.
//!
//! ## Usage
//!
//! ```ignore
//! use yororen_ui::headless::layout::{center, column, row, Spacing, Inset};
//! use yororen_ui::headless::label::label;
//!
//! center("root", cx)
//!     .w_full()
//!     .h_full()
//!     .child(
//!         column("card", cx)
//!             .gap(Spacing::Lg)
//!             .p(Inset::Lg)
//!             .items_center()
//!             .child(label("title", "Hello", cx).render(cx))
//!     )
//!     .render(cx)
//! ```
//!
//! ## No renderer layer
//!
//! Layout components are semantic sugar — `Column` is always
//! `div().flex().flex_col()`, regardless of the active theme.
//! There is nothing for a renderer to customise, so no
//! `ColumnRenderer` trait exists. Spacing / inset values that
//! *do* depend on the theme (`Spacing::Md`, `Inset::Lg`, …)
//! are resolved at `render()` time via [`Theme::get_number`].
//!
//! [`Theme::get_number`]: crate::theme::Theme::get_number

pub mod types;
pub use types::{AlignItems, Gap, Inset, JustifyContent, Length, Spacing};

pub mod center;
pub mod class;
pub mod column;
pub mod expanded;
pub mod row;
pub mod spacer;
pub mod stack;
pub mod wrap;

pub use center::{CenterProps, center};
pub use class::{LayoutClass, apply_all, classes_vec};
pub use column::{ColumnProps, column};
pub use expanded::{ExpandedProps, expanded};
pub use row::{RowProps, row};
pub use spacer::{SpacerProps, spacer};
pub use stack::{StackProps, stack};
pub use wrap::{WrapProps, wrap};