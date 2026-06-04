//! Composite component Root / Trigger / Content APIs.
//!
//! Most users interact with composite components (popover / modal /
//! dropdown_menu / tooltip / select / combo_box) through their
//! builder APIs in [`crate::component`]. The builder is the
//! recommended 99% path.
//!
//! Some users prefer the "Radix-style" split:
//!
//! ```ignore
//! PopoverRoot::new("my-popover")
//!     .open(is_open)
//!     .on_close(|w, cx| set_is_open(false))
//!     .trigger(button("Open"))
//!     .content(div().p_4().child("body"))
//! ```
//!
//! This module ships that pattern for the 6 composites listed in
//! PLAN_v0.4. Each \`XxxRoot\` wraps the existing builder and adds
//! the \`.trigger(...)\` / \`.content(...)\` convenience chain
//! that the original builders already had. The new types are
//! intentionally thin: the visuals / state machine are unchanged.
//!
//! The 6 components covered:
//!
//! - \`PopoverRoot\`
//! - \`ModalRoot\`
//! - \`DropdownMenuRoot\`
//! - \`TooltipRoot\`
//! - \`SelectRoot\`
//! - \`ComboBoxRoot\`

pub mod dropdown_menu;
pub mod modal;
pub mod popover;
pub mod select;
pub mod tooltip;
mod combo_box;

pub use dropdown_menu::DropdownMenuRoot;
pub use modal::ModalRoot;
pub use popover::PopoverRoot;
pub use select::SelectRoot;
pub use tooltip::TooltipRoot;
pub use combo_box::ComboBoxRoot;
